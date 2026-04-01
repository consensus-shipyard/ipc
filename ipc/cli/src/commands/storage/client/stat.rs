// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Stat command for displaying object metadata from storage

use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::path::PathBuf;

use fendermint_actor_bucket::ObjectState;
use fendermint_rpc::client::FendermintClient;
use serde_json::json;

use async_trait::async_trait;

use crate::commands::storage::{
    bucket,
    config::{resolve_client_config_path, StorageClientConfig, StorageConfig},
    path,
};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
pub struct StatArgs {
    /// Storage path (ipc://bucket_address/path/to/file)
    #[arg(value_name = "PATH")]
    pub path: String,

    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

pub struct StatStorage;

#[async_trait]
impl CommandLineHandler for StatStorage {
    type Arguments = StatArgs;

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let storage_path = path::StoragePath::parse(&args.path)?;

        if storage_path.is_bucket_root() {
            return Err(anyhow!(
                "Path must include a file key, not just bucket address"
            ));
        }

        let config_path = resolve_client_config_path(args.config.clone());
        let rpc_url = if config_path.exists() {
            if let Ok(client_cfg) = StorageClientConfig::load(&config_path) {
                client_cfg.tendermint_rpc_url
            } else {
                StorageConfig::load(&config_path)?.tendermint_rpc_url
            }
        } else {
            return Err(anyhow!(
                "Storage config not found at {}. Run 'ipc-cli storage client init'.",
                config_path.display()
            ));
        };

        let fm_client = FendermintClient::new_http(rpc_url.parse()?, None)?;

        // Use ListObjects with the exact key as prefix to read object metadata from bucket
        // state. This is more reliable than GetObject, which also verifies blob liveness in
        // the blobs actor and may return None while the blob is still being confirmed.
        let listed = bucket::list_objects(
            &fm_client,
            storage_path.bucket_address,
            Some(storage_path.key.clone()),
            None,
            None,
            16,
        )
        .await
        .with_context(|| {
            format!(
                "Failed to query object metadata (bucket={} key={} rpc={})",
                storage_path.bucket_address, storage_path.key, rpc_url
            )
        })?;

        let key_bytes = storage_path.key.as_bytes();
        let obj = listed
            .objects
            .iter()
            .find(|(k, _)| k.as_slice() == key_bytes)
            .map(|(_, o)| o)
            .ok_or_else(|| anyhow!("Object not found: {}", storage_path.key))?;

        if args.json {
            print_json(&storage_path, obj)?;
        } else {
            print_table(&storage_path, obj)?;
        }

        Ok(())
    }
}

fn print_json(storage_path: &path::StoragePath, obj: &ObjectState) -> Result<()> {
    let output = json!({
        "bucket": storage_path.bucket_address.to_string(),
        "key": storage_path.key,
        "hash": format!("0x{}", hex::encode(obj.hash.0)),
        "size": obj.size,
        "expiry": obj.expiry,
        "metadata": obj.metadata,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn print_table(storage_path: &path::StoragePath, obj: &ObjectState) -> Result<()> {
    println!("Object: {}", storage_path.to_uri());
    println!("  Bucket:   {}", storage_path.bucket_address);
    println!("  Key:      {}", storage_path.key);
    println!("  Hash:     0x{}", hex::encode(obj.hash.0));
    println!(
        "  Size:     {} bytes ({})",
        obj.size,
        format_size(obj.size)
    );
    println!("  Expiry:   block {}", obj.expiry);

    if !obj.metadata.is_empty() {
        println!("  Metadata:");
        for (key, value) in &obj.metadata {
            println!("    {}: {}", key, value);
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
