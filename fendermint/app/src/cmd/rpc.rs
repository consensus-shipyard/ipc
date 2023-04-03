// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use fendermint_vm_interpreter::fvm::FvmQuery;
use fendermint_vm_message::query::ActorState;
use fvm_shared::ActorID;
use serde_json::json;
use tendermint::block::Height;
use tendermint_rpc::{endpoint::abci_query::AbciQuery, v0_37::Client, HttpClient, Scheme, Url};

use crate::cmd;
use crate::{
    cmd::to_b64,
    options::rpc::{RpcArgs, RpcCommands, RpcQueryCommands},
};
use anyhow::anyhow;

// TODO: We should probably make a client interface for the operations we commonly do.

cmd! {
  RpcArgs(self) {
    let client = http_client(self.url.clone(), self.proxy_url.clone())?;
    match &self.command {
      RpcCommands::Query { height, command } => {
        let height = Height::try_from(*height)?;
        query(client, height, command).await
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
