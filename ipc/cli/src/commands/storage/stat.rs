// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Stat command for displaying object metadata from storage

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use clap::Args;
use std::path::PathBuf;

use fendermint_rpc::client::FendermintClient;
use serde_json::json;

use crate::commands::storage::{bucket, config::StorageConfig, path};
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
            return Err(anyhow!("Path must include a file key, not just bucket address"));
        }

        // Load config
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

        // Create FendermintClient
        let mut fm_client = FendermintClient::new_http(
            config.tendermint_rpc_url.parse()?,
            None,
        )?;

        // Get object metadata
        let object = bucket::get_object(
            &mut fm_client,
            storage_path.bucket_address,
            storage_path.key.clone(),
        )
        .await
        .context("Failed to get object")?;

        match object {
            Some(obj) => {
                if args.json {
                    print_json(&storage_path, &obj)?;
                } else {
                    print_table(&storage_path, &obj)?;
                }
            }
            None => {
                return Err(anyhow!("Object not found: {}", storage_path.key));
            }
        }

        Ok(())
    }
}

fn print_json(storage_path: &path::StoragePath, obj: &fendermint_actor_bucket::Object) -> Result<()> {
    let output = json!({
        "bucket": storage_path.bucket_address.to_string(),
        "key": storage_path.key,
        "hash": format!("0x{}", hex::encode(obj.hash.0)),
        "recovery_hash": format!("0x{}", hex::encode(obj.recovery_hash.0)),
        "size": obj.size,
        "expiry": obj.expiry,
        "metadata": obj.metadata,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn print_table(storage_path: &path::StoragePath, obj: &fendermint_actor_bucket::Object) -> Result<()> {
    println!("Object: {}", storage_path.to_uri());
    println!("  Bucket:        {}", storage_path.bucket_address);
    println!("  Key:           {}", storage_path.key);
    println!("  Hash:          0x{}", hex::encode(obj.hash.0));
    println!("  Recovery Hash: 0x{}", hex::encode(obj.recovery_hash.0));
    println!("  Size:          {} bytes ({})", obj.size, format_size(obj.size));
    println!("  Expiry:        block {}", obj.expiry);

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
