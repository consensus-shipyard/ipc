// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand};
use fendermint_rpc::client::FendermintClient;
use fvm_shared::address::Address;
use std::path::PathBuf;
use url::Url;

pub mod cat;
pub mod credit;
pub mod ls;
pub mod stat;
pub mod bucket;
pub mod cp;
pub mod mv;
pub mod rm;
pub mod sync;

use crate::commands::storage::config::{
    default_storage_client_config_path, resolve_client_config_path, StorageClientConfig,
};
use crate::commands::storage::client::{
    bucket::BucketCommandArgs,
    cat::{CatArgs, CatStorage},
    cp::{CopyArgs, CopyStorage},
    credit::CreditCommandArgs,
    ls::{ListArgs, ListStorage},
    mv::{MoveArgs, MoveStorage},
    rm::{RemoveArgs, RemoveStorage},
    stat::{StatArgs, StatStorage},
    sync::{SyncArgs, SyncStorage},
};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
#[command(name = "client", about = "Storage client operations and config")]
pub struct StorageClientCommandArgs {
    #[command(subcommand)]
    command: StorageClientCommands,
}

#[derive(Debug, Subcommand)]
pub enum StorageClientCommands {
    /// Create and manage storage buckets
    Bucket(BucketCommandArgs),
    /// Buy and query storage credits
    Credit(CreditCommandArgs),
    /// Copy files to/from storage
    Cp(CopyArgs),
    /// List objects in storage
    Ls(ListArgs),
    /// Display file contents from storage
    Cat(CatArgs),
    /// Show object metadata
    Stat(StatArgs),
    /// Remove objects from storage
    Rm(RemoveArgs),
    /// Move/rename objects in storage
    Mv(MoveArgs),
    /// Sync directories with storage
    Sync(SyncArgs),
    /// Initialize storage client config
    Init(StorageClientInitArgs),
    /// Show effective storage client config
    Show(StorageClientShowArgs),
    /// Update storage client config values
    Set(StorageClientSetArgs),
}

impl StorageClientCommandArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            StorageClientCommands::Bucket(args) => args.handle(global).await,
            StorageClientCommands::Credit(args) => args.handle(global).await,
            StorageClientCommands::Cp(args) => CopyStorage::handle(global, args).await,
            StorageClientCommands::Ls(args) => ListStorage::handle(global, args).await,
            StorageClientCommands::Cat(args) => CatStorage::handle(global, args).await,
            StorageClientCommands::Stat(args) => StatStorage::handle(global, args).await,
            StorageClientCommands::Rm(args) => RemoveStorage::handle(global, args).await,
            StorageClientCommands::Mv(args) => MoveStorage::handle(global, args).await,
            StorageClientCommands::Sync(args) => SyncStorage::handle(global, args).await,
            StorageClientCommands::Init(args) => init_client_config(args),
            StorageClientCommands::Show(args) => show_client_config(args),
            StorageClientCommands::Set(args) => set_client_config(args),
        }
    }
}

#[derive(Debug, Args)]
pub struct StorageClientInitArgs {
    /// Output path for storage client config
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Tendermint RPC URL
    #[arg(long, default_value = "http://127.0.0.1:26657")]
    pub rpc_url: String,
    /// Gateway URL
    #[arg(long)]
    pub gateway_url: Option<String>,
    /// Default account address
    #[arg(long)]
    pub address: Option<String>,
}

#[derive(Debug, Args)]
pub struct StorageClientShowArgs {
    /// Storage client config path
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct StorageClientSetArgs {
    /// Storage client config path
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// Tendermint RPC URL
    #[arg(long)]
    pub rpc_url: Option<String>,
    /// Gateway URL
    #[arg(long)]
    pub gateway_url: Option<String>,
    /// Default account address
    #[arg(long)]
    pub address: Option<String>,
}

fn init_client_config(args: &StorageClientInitArgs) -> Result<()> {
    validate_http_url("rpc-url", &args.rpc_url)?;
    if let Some(gateway_url) = &args.gateway_url {
        validate_http_url("gateway-url", gateway_url)?;
    }

    let path = args
        .out
        .clone()
        .unwrap_or_else(default_storage_client_config_path);
    let cfg = StorageClientConfig {
        tendermint_rpc_url: args.rpc_url.clone(),
        gateway_url: args.gateway_url.clone(),
        address: args.address.clone(),
    };
    cfg.save(&path)?;
    println!("Storage client config written to {}", path.display());
    Ok(())
}

fn show_client_config(args: &StorageClientShowArgs) -> Result<()> {
    let path = resolve_client_config_path(args.config.clone());
    let cfg = if path.exists() {
        StorageClientConfig::load(&path)?
    } else {
        StorageClientConfig::default_with_local_rpc()
    };
    println!("{}", serde_yaml::to_string(&cfg)?);
    Ok(())
}

fn set_client_config(args: &StorageClientSetArgs) -> Result<()> {
    let path = resolve_client_config_path(args.config.clone());
    let mut cfg = if path.exists() {
        StorageClientConfig::load(&path)?
    } else {
        StorageClientConfig::default_with_local_rpc()
    };
    if let Some(v) = &args.rpc_url {
        validate_http_url("rpc-url", v)?;
        cfg.tendermint_rpc_url = v.clone();
    }
    if let Some(v) = &args.gateway_url {
        validate_http_url("gateway-url", v)?;
        cfg.gateway_url = Some(v.clone());
    }
    if let Some(v) = &args.address {
        cfg.address = Some(v.clone());
    }
    cfg.save(&path)?;
    println!("Updated storage client config at {}", path.display());
    Ok(())
}

/// Download object data from the gateway, with a hash-based fallback.
///
/// Primary path: `GET /v1/objects/{bucket}/{key}` — gateway resolves the key to a blob hash
/// on-chain then fetches via shard retrieval.
///
/// Fallback: if the primary path returns 404 (common when the blob hasn't been confirmed in
/// the blobs actor yet), we query `ListObjects` directly to retrieve the hash from bucket
/// state, then try `GET /v1/blobs/{hash}` on the gateway.
///
/// If both paths fail, the error explains what was attempted so the caller can diagnose.
pub(crate) async fn download_object_data(
    gateway: &crate::commands::storage::gateway::GatewayClient,
    gateway_url: &str,
    fm_client: &FendermintClient,
    rpc_url: &str,
    bucket_address: Address,
    key: &str,
) -> Result<Vec<u8>> {
    match gateway.download_object(&bucket_address, key, None).await {
        Ok(data) => return Ok(data),
        Err(primary_err) => {
            // The bucket actor's GetObject also verifies blob liveness in the blobs actor.
            // ListObjects reads bucket state directly and is not gated on blob liveness.
            let listed = crate::commands::storage::bucket::list_objects(
                fm_client,
                bucket_address,
                Some(key.to_string()),
                None,
                None,
                16,
            )
            .await
            .with_context(|| {
                format!(
                    "Primary download failed ({primary_err}); also failed to query on-chain \
                     object metadata as fallback (bucket={bucket_address} key={key} rpc={rpc_url})"
                )
            })?;

            let key_bytes = key.as_bytes();
            let obj = listed
                .objects
                .iter()
                .find(|(k, _)| k.as_slice() == key_bytes)
                .map(|(_, o)| o)
                .ok_or_else(|| {
                    anyhow!(
                        "Object not found on-chain (bucket={} key={}). \
                         If you just uploaded it, storage nodes may still be confirming the blob. \
                         Primary gateway error: {}",
                        bucket_address,
                        key,
                        primary_err
                    )
                })?;

            let blob_hash = hex::encode(obj.hash.0);
            gateway
                .download_blob(&blob_hash, None)
                .await
                .with_context(|| {
                    format!(
                        "Object is registered on-chain (hash=0x{blob_hash}) but the gateway \
                         could not retrieve the blob. Storage nodes may still be distributing \
                         shards or the blob may have expired. \
                         Gateway: {gateway_url} Bucket: {bucket_address} Key: {key}"
                    )
                })
        }
    }
}

fn validate_http_url(field_name: &str, value: &str) -> Result<()> {
    let parsed = Url::parse(value)
        .map_err(|e| anyhow::anyhow!("invalid {} '{}': {}", field_name, value, e))?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        scheme => Err(anyhow::anyhow!(
            "invalid {} '{}': unsupported scheme '{}', expected http or https",
            field_name,
            value,
            scheme
        )),
    }
}
