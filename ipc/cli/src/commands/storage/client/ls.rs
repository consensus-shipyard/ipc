// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! List command for storage operations

use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::path::PathBuf;

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
pub struct ListArgs {
    /// Storage path (ipc://bucket_address/prefix)
    #[arg(value_name = "PATH")]
    pub path: String,

    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Show all details
    #[arg(short, long)]
    pub long: bool,

    /// Delimiter for hierarchical listing (default: none, S3-style: "/")
    #[arg(short, long)]
    pub delimiter: Option<String>,

    /// Maximum number of objects to list
    #[arg(long, default_value = "100")]
    pub limit: u64,
}

pub struct ListStorage;

#[async_trait]
impl CommandLineHandler for ListStorage {
    type Arguments = ListArgs;

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let storage_path = path::StoragePath::parse(&args.path)?;

        // Load client/provider config and resolve RPC endpoint.
        let config_path = resolve_client_config_path(args.config.clone());
        let rpc_url = if config_path.exists() {
            if let Ok(client_cfg) = StorageClientConfig::load(&config_path) {
                client_cfg.tendermint_rpc_url
            } else {
                StorageConfig::load(&config_path)?.tendermint_rpc_url
            }
        } else {
            return Err(anyhow!(
                "Storage config not found at {}. Run 'ipc-cli storage client init' (user) or 'ipc-cli storage node init' (provider).",
                config_path.display()
            ));
        };

        // Create FendermintClient
        let fm_client = FendermintClient::new_http(rpc_url.parse()?, None)?;

        // List objects
        let prefix = if storage_path.is_bucket_root() {
            None
        } else {
            Some(storage_path.key.clone())
        };

        let list_result = bucket::list_objects(
            &fm_client,
            storage_path.bucket_address,
            prefix.clone(),
            args.delimiter.clone(),
            None, // start_key
            args.limit,
        )
        .await
        .with_context(|| {
            format!(
                "Failed to list objects (bucket={} prefix={} delimiter={} limit={} rpc={})",
                storage_path.bucket_address,
                prefix.as_deref().unwrap_or("<none>"),
                args.delimiter.as_deref().unwrap_or("<none>"),
                args.limit,
                rpc_url
            )
        })?;

        // Output results
        if args.json {
            print_json(&list_result, &prefix)?;
        } else {
            print_table(&list_result, &prefix, args.long)?;
        }

        Ok(())
    }
}

fn print_json(
    result: &fendermint_actor_bucket::ListObjectsReturn,
    _prefix: &Option<String>,
) -> Result<()> {
    let mut objects = Vec::new();
    for (key, state) in &result.objects {
        let key_str = String::from_utf8_lossy(key);
        objects.push(json!({
            "key": key_str,
            "hash": format!("0x{}", hex::encode(state.hash.0)),
            "size": state.size,
            "expiry": state.expiry,
            "metadata": state.metadata,
        }));
    }

    let mut prefixes = Vec::new();
    for prefix in &result.common_prefixes {
        prefixes.push(String::from_utf8_lossy(prefix).to_string());
    }

    let output = json!({
        "objects": objects,
        "common_prefixes": prefixes,
        "next_key": result.next_key.as_ref().map(|k| String::from_utf8_lossy(k).to_string()),
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn print_table(
    result: &fendermint_actor_bucket::ListObjectsReturn,
    _prefix: &Option<String>,
    long: bool,
) -> Result<()> {
    if result.objects.is_empty() && result.common_prefixes.is_empty() {
        println!("No objects found");
        return Ok(());
    }

    // Print common prefixes (directories) first
    if !result.common_prefixes.is_empty() {
        if long {
            println!("{:<50} {:<10} {:<66}", "KEY", "SIZE", "HASH");
            println!("{}", "-".repeat(130));
        }

        for prefix_bytes in &result.common_prefixes {
            let prefix_str = String::from_utf8_lossy(prefix_bytes);
            if long {
                println!("{:<50} {:<10} {:<66}", prefix_str, "DIR", "-");
            } else {
                println!("{}", prefix_str);
            }
        }
    }

    // Print objects
    if !result.objects.is_empty() {
        if long && result.common_prefixes.is_empty() {
            println!("{:<50} {:<10} {:<66}", "KEY", "SIZE", "HASH");
            println!("{}", "-".repeat(130));
        }

        for (key, state) in &result.objects {
            let key_str = String::from_utf8_lossy(key);
            let hash_str = format!("0x{}", hex::encode(&state.hash.0[..8])); // Truncated hash

            if long {
                println!(
                    "{:<50} {:<10} {:<66}",
                    key_str,
                    format_size(state.size),
                    hash_str
                );
            } else {
                println!("{}", key_str);
            }
        }
    }

    // Print pagination info
    if let Some(next_key) = &result.next_key {
        let next_key_str = String::from_utf8_lossy(next_key);
        println!("\n(More results available, next key: {})", next_key_str);
    }

    println!(
        "\nTotal: {} objects, {} prefixes",
        result.objects.len(),
        result.common_prefixes.len()
    );

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
