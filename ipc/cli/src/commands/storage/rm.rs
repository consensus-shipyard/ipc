// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Remove command for deleting objects from storage

use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::io::{self, Write};
use std::path::PathBuf;

use fendermint_rpc::client::FendermintClient;

use crate::commands::storage::{bucket, config::StorageConfig, path};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Storage path (ipc://bucket_address/path/to/file)
    #[arg(value_name = "PATH")]
    pub path: String,

    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Recursive delete (for prefix-based deletion)
    #[arg(short, long)]
    pub recursive: bool,

    /// Force deletion without confirmation
    #[arg(short, long)]
    pub force: bool,
}

pub struct RemoveStorage;

impl CommandLineHandler for RemoveStorage {
    type Arguments = RemoveArgs;

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let storage_path = path::StoragePath::parse(&args.path)?;

        if storage_path.is_bucket_root() {
            return Err(anyhow!("Cannot delete entire bucket. Specify a key or prefix."));
        }

        // Handle recursive deletion
        if args.recursive {
            return delete_recursive(&storage_path, args).await;
        }

        // Single file deletion
        delete_file(&storage_path, args).await
    }
}

async fn delete_file(storage_path: &path::StoragePath, args: &RemoveArgs) -> Result<()> {
    // Confirm deletion unless --force
    if !args.force {
        print!("Delete {}? [y/N] ", storage_path.to_uri());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted");
            return Ok(());
        }
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

    // Create FendermintClient and bound client
    let fm_client = FendermintClient::new_http(
        config.tendermint_rpc_url.parse()?,
        None,
    )?;

    let secret_key = fendermint_rpc::message::SignedMessageFactory::read_secret_key(
        &config.secret_key_file
    )?;

    let chain_id = fvm_shared::chainid::ChainID::from(0); // TODO: Get from chain
    let bound_client = fendermint_rpc::tx::BoundClientBuilder::new(fm_client)
        .secret_key(secret_key)
        .chain_id(chain_id)
        .build()
        .await?;

    let mut bound_client = bound_client;

    // Delete object
    println!("Deleting {}...", storage_path.key);

    bucket::delete_object(
        &mut bound_client,
        storage_path.bucket_address,
        storage_path.key.clone(),
    )
    .await
    .context("Failed to delete object")?;

    println!("✓ Deleted: {}", storage_path.key);

    Ok(())
}

async fn delete_recursive(storage_path: &path::StoragePath, args: &RemoveArgs) -> Result<()> {
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

    // List all objects with the prefix
    let fm_client = FendermintClient::new_http(
        config.tendermint_rpc_url.parse()?,
        None,
    )?;

    let prefix = storage_path.key.clone();
    let mut deleted_count = 0;
    let mut start_key = None;

    loop {
        let list_result = bucket::list_objects(
            &fm_client,
            storage_path.bucket_address,
            Some(prefix.clone()),
            None, // no delimiter for recursive
            start_key,
            100, // batch size
        )
        .await
        .context("Failed to list objects")?;

        if list_result.objects.is_empty() {
            break;
        }

        // Confirm deletion unless --force
        if !args.force && deleted_count == 0 {
            println!("Found {} objects to delete with prefix: {}", list_result.objects.len(), prefix);
            print!("Continue? [y/N] ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted");
                return Ok(());
            }
        }

        // Delete each object
        let secret_key = fendermint_rpc::message::SignedMessageFactory::read_secret_key(
            &config.secret_key_file
        )?;

        let chain_id = fvm_shared::chainid::ChainID::from(0);
        let bound_client = fendermint_rpc::tx::BoundClientBuilder::new(fm_client.clone())
            .secret_key(secret_key)
            .chain_id(chain_id)
            .build()
            .await?;

        let mut bound_client = bound_client;

        for (key, _) in &list_result.objects {
            let key_str = String::from_utf8_lossy(key).to_string();
            bucket::delete_object(
                &mut bound_client,
                storage_path.bucket_address,
                key_str.clone(),
            )
            .await
            .with_context(|| format!("Failed to delete object: {}", key_str))?;

            println!("✓ Deleted: {}", key_str);
            deleted_count += 1;
        }

        // Check if there are more pages
        if list_result.next_key.is_none() {
            break;
        }

        start_key = list_result.next_key.map(|k| String::from_utf8_lossy(&k).to_string());
    }

    println!("\nDeleted {} objects", deleted_count);

    Ok(())
}
