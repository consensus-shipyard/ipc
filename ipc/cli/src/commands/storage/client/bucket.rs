// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Bucket subcommand for creating and managing storage buckets on-chain.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use clap::{Args, Subcommand};
use ethers::abi::{encode as abi_encode, Token};
use num_traits::Zero;
use std::collections::HashMap;
use std::path::PathBuf;

use fendermint_rpc::client::FendermintClient;
use fendermint_rpc::message::SignedMessageFactory;
use fendermint_rpc::tx::{TxClient, TxCommit};
use fendermint_rpc::QueryClient;
use fendermint_vm_actor_interface::adm::{self, Kind, ListMetadataParams, Method as AdmMethod};
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_actor_interface::evm;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_ipld_encoding::{BytesSer, RawBytes};
use fvm_shared::address::Address;
use fvm_shared::chainid::ChainID;
use fvm_shared::econ::TokenAmount;

use crate::commands::storage::client_context::{
    resolve_default_owner_from_client_config, resolve_rpc_url, resolve_write_context,
};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
#[command(name = "bucket", about = "Create and manage storage buckets")]
pub struct BucketCommandArgs {
    #[command(subcommand)]
    command: BucketCommands,
}

#[derive(Debug, Subcommand)]
pub enum BucketCommands {
    /// Create a new storage bucket
    Create(CreateBucketArgs),
    /// List buckets owned by an address
    List(ListBucketsArgs),
}

impl BucketCommandArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            BucketCommands::Create(args) => CreateBucket::handle(global, args).await,
            BucketCommands::List(args) => ListBuckets::handle(global, args).await,
        }
    }
}

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

#[derive(Debug, Args)]
pub struct CreateBucketArgs {
    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Optional owner address (defaults to the operator key address)
    #[arg(long)]
    pub owner: Option<String>,

    /// Optional metadata key=value pairs (can be repeated)
    #[arg(long = "metadata", value_name = "KEY=VALUE")]
    pub metadata: Vec<String>,
}

pub struct CreateBucket;

#[async_trait]
impl CommandLineHandler for CreateBucket {
    type Arguments = CreateBucketArgs;

    async fn handle(global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let write_ctx = resolve_write_context(global, args.config.clone())?;
        let rpc_url = write_ctx.rpc_url;
        let secret_key = write_ctx.secret_key;
        let fm_client = FendermintClient::new_http(rpc_url.parse()?, None)?;

        let chain_id = crate::commands::storage::bucket::query_chain_id(&fm_client)
            .await
            .context("Failed to query chain ID")?;

        let pub_key = secret_key.public_key();
        let sender_eth = EthAddress::new_secp256k1(&pub_key.serialize())
            .context("failed to derive delegated sender address from signer key")?;
        let addr = Address::new_delegated(10, &sender_eth.0)
            .context("failed to construct delegated sender address")?;

        // Parse owner address — ADM requires delegated (f410) address
        let owner = if let Some(ref owner_str) = args.owner {
            crate::require_fil_addr_from_str(owner_str)?
        } else {
            let eth_addr = EthAddress::new_secp256k1(&pub_key.serialize())
                .context("failed to derive delegated address")?;
            Address::new_delegated(10, &eth_addr.0).context("failed to construct f410 address")?
        };

        let state = fm_client
            .actor_state(&addr, FvmQueryHeight::default())
            .await
            .context("Failed to get actor state")?;
        let sequence = state.value.map(|(_, s)| s.sequence).ok_or_else(|| {
            anyhow!(
                "sender actor {} does not exist on-chain at {}. Fund/initialize this delegated \
                 address first.",
                addr,
                rpc_url
            )
        })?;

        let mf = SignedMessageFactory::new(secret_key, addr, sequence, ChainID::from(chain_id));
        let mut bound_client = fm_client.bind(mf);

        // Parse metadata
        let metadata = parse_metadata(&args.metadata)?;

        let owner_eth = ipc_api::evm::payload_to_evm_address(owner.payload())
            .context("bucket owner must be an EVM/delegated address")?;
        let (calldata, invoke_params) = if metadata.is_empty() {
            let mut calldata = Vec::with_capacity(4 + 32);
            // createBucket(address)
            calldata.extend_from_slice(&[0xf6, 0xd6, 0xc4, 0x20]);
            calldata.extend_from_slice(&abi_encode(&[Token::Address(owner_eth)]));
            let invoke_params = RawBytes::serialize(BytesSer(&calldata))
                .context("Failed to serialize FEVM calldata for createBucket(address)")?;
            (calldata, invoke_params)
        } else {
            let metadata_tokens: Vec<Token> = metadata
                .iter()
                .map(|(k, v)| Token::Tuple(vec![Token::String(k.clone()), Token::String(v.clone())]))
                .collect();
            let mut calldata = Vec::with_capacity(4 + 128);
            // createBucket(address,(string,string)[])
            calldata.extend_from_slice(&[0xe1, 0x29, 0xed, 0x90]);
            calldata.extend_from_slice(&abi_encode(&[
                Token::Address(owner_eth),
                Token::Array(metadata_tokens),
            ]));
            let invoke_params = RawBytes::serialize(BytesSer(&calldata))
                .context("Failed to serialize FEVM calldata for createBucket(address,metadata)")?;
            (calldata, invoke_params)
        };
        let gas_params = crate::commands::storage::bucket::tx_gas_params(
            &bound_client,
            addr,
            adm::ADM_ACTOR_ADDR,
            evm::Method::InvokeContract as u64,
            invoke_params,
            TokenAmount::zero(),
        )
        .await
        .context("Failed to estimate CreateExternal gas parameters")?;

        println!("Creating bucket...");

        let res = TxClient::<TxCommit>::fevm_invoke(
            &mut bound_client,
            adm::ADM_ACTOR_ADDR,
            calldata.into(),
            TokenAmount::zero(),
            gas_params,
        )
        .await
        .map_err(|e| {
            anyhow!(
                "Failed to send CreateExternal transaction: {} (sender={} owner={} rpc={})",
                e,
                addr,
                owner,
                rpc_url
            )
        })?;

        if res.response.check_tx.code.is_err() {
            let log = &res.response.check_tx.log;
            let info = &res.response.check_tx.info;
            return Err(anyhow!(
                "CreateExternal check_tx failed (code {:?}): log={} info={} sender={} owner={} rpc={}",
                res.response.check_tx.code,
                if log.is_empty() { "<empty>" } else { log },
                if info.is_empty() { "<empty>" } else { info },
                addr,
                owner,
                rpc_url
            ));
        }

        if res.response.deliver_tx.code.is_err() {
            let log = &res.response.deliver_tx.log;
            let info = &res.response.deliver_tx.info;
            return Err(anyhow!(
                "CreateExternal deliver_tx failed (code {:?}): log={} info={} sender={} owner={} rpc={}",
                res.response.deliver_tx.code,
                if log.is_empty() { "<empty>" } else { log },
                if info.is_empty() { "<empty>" } else { info },
                addr,
                owner,
                rpc_url
            ));
        }

        println!("Bucket created successfully!");
        println!("  Owner:       {}", owner);
        println!("  Tx hash:     {}", res.response.hash);
        println!("Run `ipc-cli storage client bucket list` to see the new bucket address.");

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

#[derive(Debug, Args)]
pub struct ListBucketsArgs {
    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Owner address to list buckets for (defaults to operator key address)
    #[arg(long)]
    pub owner: Option<String>,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

pub struct ListBuckets;

#[async_trait]
impl CommandLineHandler for ListBuckets {
    type Arguments = ListBucketsArgs;

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let rpc_url = resolve_rpc_url(args.config.clone())?;
        let fm_client = FendermintClient::new_http(rpc_url.parse()?, None)?;

        // Determine the owner address
        let owner = if let Some(ref owner_str) = args.owner {
            crate::require_fil_addr_from_str(owner_str)?
        } else if let Some(owner_from_cfg) = resolve_default_owner_from_client_config(args.config.clone())? {
            owner_from_cfg
        } else {
            return Err(anyhow!(
                "No default owner configured. Pass --owner, or set `address` in storage client config."
            ));
        };

        let params = ListMetadataParams { owner };
        let params_bytes =
            RawBytes::serialize(params).context("Failed to serialize ListMetadataParams")?;

        let msg = fvm_shared::message::Message {
            version: Default::default(),
            from: fendermint_vm_actor_interface::system::SYSTEM_ACTOR_ADDR,
            to: adm::ADM_ACTOR_ADDR,
            sequence: 0,
            value: TokenAmount::zero(),
            method_num: AdmMethod::ListMetadata as u64,
            params: params_bytes,
            gas_limit: 10_000_000_000,
            gas_fee_cap: TokenAmount::zero(),
            gas_premium: TokenAmount::zero(),
        };

        let response = fm_client
            .call(msg, FvmQueryHeight::default())
            .await
            .with_context(|| {
                format!(
                    "Failed to query ListMetadata (owner={} rpc={})",
                    owner, rpc_url
                )
            })?;

        if response.value.code.is_err() {
            return Err(anyhow!(
                "ListMetadata query failed: {} (owner={} rpc={})",
                response.value.info,
                owner,
                rpc_url
            ));
        }

        let return_data = fendermint_rpc::response::decode_data(&response.value.data)
            .context("Failed to decode response data")?;

        let results: Vec<adm::Metadata> = fvm_ipld_encoding::from_slice(&return_data)
            .context("Failed to decode ListMetadata response")?;

        if args.json {
            let json_items: Vec<serde_json::Value> = results
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "kind": format!("{}", m.kind),
                        "address": m.address.to_string(),
                        "metadata": m.metadata,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_items)?);
        } else {
            let buckets: Vec<&adm::Metadata> = results
                .iter()
                .filter(|m| matches!(m.kind, Kind::Bucket))
                .collect();

            if buckets.is_empty() {
                println!("No buckets found for {}", owner);
            } else {
                println!("{:<50} METADATA", "ADDRESS");
                println!("{}", "-".repeat(80));
                for m in &buckets {
                    let meta_str = if m.metadata.is_empty() {
                        String::from("-")
                    } else {
                        m.metadata
                            .iter()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    println!("{:<50} {}", m.address, meta_str);
                }
                println!("\nTotal: {} buckets", buckets.len());
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_metadata(pairs: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for pair in pairs {
        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| anyhow!("Invalid metadata format '{}', expected KEY=VALUE", pair))?;
        map.insert(key.to_string(), value.to_string());
    }
    Ok(map)
}
