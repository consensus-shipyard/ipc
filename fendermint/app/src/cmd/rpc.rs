// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use fendermint_vm_interpreter::fvm::{FvmMessage, FvmQuery};
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::query::ActorState;
use fendermint_vm_message::signed::SignedMessage;
use fvm_shared::address::Address;
use fvm_shared::crypto::signature::SECP_SIG_LEN;
use fvm_shared::crypto::signature::{Signature, SignatureType};
use fvm_shared::{ActorID, METHOD_SEND};
use libsecp256k1::PublicKey;
use serde::Serialize;
use serde_json::json;
use tendermint::block::Height;
use tendermint_rpc::{endpoint::abci_query::AbciQuery, v0_37::Client, HttpClient, Scheme, Url};

use crate::cmd;
use crate::options::rpc::{BroadcastMode, TransArgs};
use crate::{
    cmd::to_b64,
    options::rpc::{RpcArgs, RpcCommands, RpcQueryCommands},
};
use anyhow::anyhow;

use super::key::read_secret_key;

// TODO: We should probably make a client interface for the operations we commonly do.

cmd! {
  RpcArgs(self) {
    let client = http_client(self.url.clone(), self.proxy_url.clone())?;
    match &self.command {
      RpcCommands::Query { height, command } => {
        let height = Height::try_from(*height)?;
        query(client, height, command).await
      },
      RpcCommands::Transfer { args } => {
        transfer(client, args).await
      }
    }
  }
}

async fn query(
    client: HttpClient,
    height: Height,
    command: &RpcQueryCommands,
) -> anyhow::Result<()> {
    match command {
        RpcQueryCommands::Ipld { cid } => {
            abci_query_print(client, height, FvmQuery::Ipld(*cid), |res| {
                Ok(to_b64(&res.value))
            })
            .await
        }
        RpcQueryCommands::ActorState { address } => {
            abci_query_print(client, height, FvmQuery::ActorState(*address), |res| {
                let state: ActorState =
                    fvm_ipld_encoding::from_slice(&res.value).context("failed to decode state")?;
                let id: ActorID =
                    fvm_ipld_encoding::from_slice(&res.key).context("failed to decode ID")?;

                let out = json! ({
                  "id": id,
                  "state": state,
                });

                // Print JSON as a single line - we can display it nicer with `jq` if needed.
                let json = serde_json::to_string(&out)?;

                Ok(json)
            })
            .await
        }
    }
}

/// Execute token transfer through RPC and print the response to STDOUT as JSON.
async fn transfer(client: HttpClient, args: &TransArgs) -> anyhow::Result<()> {
    let data = transfer_payload(args)?;
    broadcast_and_print(client, data, args.broadcast_mode).await
}

/// Broadcast a transaction to tendermint and print the results to STDOUT as JSON.
async fn broadcast_and_print(
    client: HttpClient,
    data: Vec<u8>,
    mode: BroadcastMode,
) -> anyhow::Result<()> {
    match mode {
        BroadcastMode::Async => print_json(client.broadcast_tx_async(data).await?),
        BroadcastMode::Sync => print_json(client.broadcast_tx_async(data).await?),
        BroadcastMode::Commit => print_json(client.broadcast_tx_commit(data).await?),
    }
}

/// Display some value as JSON.
fn print_json<T: Serialize>(value: T) -> anyhow::Result<()> {
    // Using "jsonline"; use `jq` to format.
    let json = serde_json::to_string(&value)?;
    println!("{}", json);
    Ok(())
}

/// Construct transfer payload.
fn transfer_payload(args: &TransArgs) -> anyhow::Result<Vec<u8>> {
    transaction_payload(args, METHOD_SEND, Default::default())
}

/// Construct transaction payload.
fn transaction_payload(
    args: &TransArgs,
    method_num: u64,
    params: fvm_ipld_encoding::RawBytes,
) -> anyhow::Result<Vec<u8>> {
    let sk = read_secret_key(&args.secret_key)?;
    let pk = PublicKey::from_secret_key(&sk);
    let from = Address::new_secp256k1(&pk.serialize())?;
    let message = FvmMessage {
        version: Default::default(), // TODO: What does this do?
        from,
        to: args.to,
        sequence: args.sequence,
        value: args.value.clone(),
        method_num,
        params,
        gas_limit: args.gas_limit,
        gas_fee_cap: args.gas_fee_cap.clone(),
        gas_premium: args.gas_premium.clone(),
    };
    let cid = SignedMessage::cid(&message)?;
    let signature = Signature {
        sig_type: SignatureType::Secp256k1,
        bytes: secp_sign(&sk, &cid.to_bytes()).to_vec(),
    };
    let signed = SignedMessage { message, signature };
    let chain = ChainMessage::Signed(Box::new(signed));
    let data = fvm_ipld_encoding::to_vec(&chain)?;
    Ok(data)
}

/// Fetch the query result from the server and print something to STDOUT.
async fn abci_query_print<F>(
    client: HttpClient,
    height: Height,
    query: FvmQuery,
    render: F,
) -> anyhow::Result<()>
where
    F: FnOnce(AbciQuery) -> anyhow::Result<String>,
{
    let data = fvm_ipld_encoding::to_vec(&query).context("failed to encode query")?;
    let res: AbciQuery = client.abci_query(None, data, Some(height), false).await?;
    if res.code.is_ok() {
        let output = render(res)?;
        println!("{}", output);
    } else {
        eprintln!("query returned non-zero exit code: {}", res.code.value());
    }
    Ok(())
}

// Retrieve the proxy URL with precedence:
// 1. If supplied, that's the proxy URL used.
// 2. If not supplied, but environment variable HTTP_PROXY or HTTPS_PROXY are
//    supplied, then use the appropriate variable for the URL in question.
//
// Copied from `tendermint_rpc`.
fn get_http_proxy_url(url_scheme: Scheme, proxy_url: Option<Url>) -> anyhow::Result<Option<Url>> {
    match proxy_url {
        Some(u) => Ok(Some(u)),
        None => match url_scheme {
            Scheme::Http => std::env::var("HTTP_PROXY").ok(),
            Scheme::Https => std::env::var("HTTPS_PROXY")
                .ok()
                .or_else(|| std::env::var("HTTP_PROXY").ok()),
            _ => {
                if std::env::var("HTTP_PROXY").is_ok() || std::env::var("HTTPS_PROXY").is_ok() {
                    tracing::warn!(
                        "Ignoring HTTP proxy environment variables for non-HTTP client connection"
                    );
                }
                None
            }
        }
        .map(|u| u.parse::<Url>().map_err(|e| anyhow!(e)))
        .transpose(),
    }
}

fn http_client(url: Url, proxy_url: Option<Url>) -> anyhow::Result<HttpClient> {
    let proxy_url = get_http_proxy_url(url.scheme(), proxy_url)?;
    let client = match proxy_url {
        Some(proxy_url) => {
            tracing::debug!(
                "Using HTTP client with proxy {} to submit request to {}",
                proxy_url,
                url
            );
            HttpClient::new_with_proxy(url, proxy_url)?
        }
        None => {
            tracing::debug!("Using HTTP client to submit request to: {}", url);
            HttpClient::new(url)?
        }
    };
    Ok(client)
}

fn secp_sign(sk: &libsecp256k1::SecretKey, data: &[u8]) -> [u8; SECP_SIG_LEN] {
    let hash: [u8; 32] = blake2b_simd::Params::new()
        .hash_length(32)
        .to_state()
        .update(data)
        .finalize()
        .as_bytes()
        .try_into()
        .unwrap();

    let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&hash), sk);

    let mut signature = [0u8; SECP_SIG_LEN];
    signature[..64].copy_from_slice(&sig.serialize());
    signature[64] = recovery_id.serialize();
    signature
}
