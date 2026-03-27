// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Bucket subcommand for creating and managing storage buckets on-chain.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use clap::{Args, Subcommand};
use num_traits::Zero;
use std::collections::HashMap;
use std::path::PathBuf;

use fendermint_rpc::client::FendermintClient;
use fendermint_rpc::message::{GasParams, SignedMessageFactory};
use fendermint_rpc::tx::{BoundClient, TxClient, TxCommit};
use fendermint_rpc::QueryClient;
use fendermint_vm_actor_interface::adm::{
    self, CreateExternalParams, CreateExternalReturn, Kind, ListMetadataParams,
    Method as AdmMethod,
};
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::chainid::ChainID;
use fvm_shared::econ::TokenAmount;

use crate::commands::storage::config::StorageConfig;
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

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let config_path = args.config.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap()
                .join(".ipc")
                .join("storage_default.yaml")
        });

        let config = if config_path.exists() {
            StorageConfig::load(&config_path)?
        } else {
            return Err(anyhow!(
                "Storage config not found at {}. Run 'ipc-cli storage init' first.",
                config_path.display()
            ));
        };

        let fm_client = FendermintClient::new_http(config.tendermint_rpc_url.parse()?, None)?;

        let chain_id = super::bucket::query_chain_id(&fm_client)
            .await
            .context("Failed to query chain ID")?;

        let secret_key =
            SignedMessageFactory::read_secret_key(&config.secret_key_file)?;
        let addr = Address::new_secp256k1(&secret_key.public_key().serialize())?;

        let state = fm_client
            .actor_state(&addr, FvmQueryHeight::default())
            .await
            .context("Failed to get actor state")?;
        let sequence = state.value.map(|(_, s)| s.sequence).unwrap_or(0);

        let mf = SignedMessageFactory::new(secret_key, addr, sequence, ChainID::from(chain_id));
        let mut bound_client = fm_client.bind(mf);

        // Parse owner address
        let owner = if let Some(ref owner_str) = args.owner {
            crate::require_fil_addr_from_str(owner_str)?
        } else {
            addr
        };

        // Parse metadata
        let metadata = parse_metadata(&args.metadata)?;

        let params = CreateExternalParams {
            owner,
            kind: Kind::Bucket,
            metadata,
        };

        let params_bytes =
            RawBytes::serialize(params).context("Failed to serialize CreateExternalParams")?;

        let gas_params = GasParams {
            gas_limit: 10_000_000_000,
            gas_fee_cap: TokenAmount::from_atto(100),
            gas_premium: TokenAmount::from_atto(100),
        };

        println!("Creating bucket...");

        let res = TxClient::<TxCommit>::transaction(
            &mut bound_client,
            adm::ADM_ACTOR_ADDR,
            AdmMethod::CreateExternal as u64,
            params_bytes,
            TokenAmount::zero(),
            gas_params,
        )
        .await
        .context("Failed to send CreateExternal transaction")?;

        if res.response.check_tx.code.is_err() {
            return Err(anyhow!(
                "CreateExternal check_tx failed: {}",
                res.response.check_tx.log
            ));
        }

        if res.response.deliver_tx.code.is_err() {
            return Err(anyhow!(
                "CreateExternal deliver_tx failed: {}",
                res.response.deliver_tx.log
            ));
        }

        // Decode the return value
        let return_data = fendermint_rpc::response::decode_data(&res.response.deliver_tx.data)
            .context("Failed to decode response data")?;
        let result: CreateExternalReturn = fvm_ipld_encoding::from_slice(&return_data)
            .context("Failed to decode CreateExternalReturn")?;

        println!("Bucket created successfully!");
        println!("  Actor ID: {}", result.actor_id);
        if let Some(ref robust) = result.robust_address {
            println!("  Address:  {}", robust);
        }
        println!("  Owner:    {}", owner);

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
        let config_path = args.config.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap()
                .join(".ipc")
                .join("storage_default.yaml")
        });

        let config = if config_path.exists() {
            StorageConfig::load(&config_path)?
        } else {
            return Err(anyhow!(
                "Storage config not found at {}. Run 'ipc-cli storage init' first.",
                config_path.display()
            ));
        };

        let fm_client = FendermintClient::new_http(config.tendermint_rpc_url.parse()?, None)?;

        // Determine the owner address
        let owner = if let Some(ref owner_str) = args.owner {
            crate::require_fil_addr_from_str(owner_str)?
        } else {
            let secret_key =
                SignedMessageFactory::read_secret_key(&config.secret_key_file)?;
            Address::new_secp256k1(&secret_key.public_key().serialize())?
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
            .context("Failed to query ListMetadata")?;

        if response.value.code.is_err() {
            return Err(anyhow!(
                "ListMetadata query failed: {}",
                response.value.info
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
