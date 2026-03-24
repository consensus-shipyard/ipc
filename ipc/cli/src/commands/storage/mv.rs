// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Move/rename command for storage objects

use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::path::PathBuf;

use fendermint_actor_blobs_shared::bytes::B256;
use fendermint_rpc::client::FendermintClient;

use crate::commands::storage::{bucket, config::StorageConfig, path};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
pub struct MoveArgs {
    /// Source storage path (ipc://bucket_address/path/to/file)
    #[arg(value_name = "SOURCE")]
    pub source: String,

    /// Destination storage path (ipc://bucket_address/path/to/newfile)
    #[arg(value_name = "DEST")]
    pub dest: String,

    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,
}

pub struct MoveStorage;

impl CommandLineHandler for MoveStorage {
    type Arguments = MoveArgs;

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let source_path = path::StoragePath::parse(&args.source)?;
        let dest_path = path::StoragePath::parse(&args.dest)?;

        if source_path.is_bucket_root() || dest_path.is_bucket_root() {
            return Err(anyhow!("Paths must include file keys, not just bucket addresses"));
        }

        println!("Moving {} -> {}", source_path.to_uri(), dest_path.to_uri());

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

        // Create clients
        let fm_client = FendermintClient::new_http(
            config.tendermint_rpc_url.parse()?,
            None,
        )?;

        // Get source object metadata
        let mut query_client = fm_client.clone();
        let source_object = bucket::get_object(
            &mut query_client,
            source_path.bucket_address,
            source_path.key.clone(),
        )
        .await
        .context("Failed to get source object")?;

        let source_object = source_object.ok_or_else(|| anyhow!("Source object not found: {}", source_path.key))?;

        // Create bound client for transactions
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

        // If moving within the same bucket, we can reuse the blob hash
        if source_path.bucket_address == dest_path.bucket_address {
            println!("Moving within same bucket (reusing blob)...");

            // Add object at new location with same hash
            // We need to extract source node ID from somewhere - using zero for now
            let source_node = B256([0u8; 32]); // TODO: Get actual source node ID

            bucket::add_object(
                &mut bound_client,
                dest_path.bucket_address,
                source_node,
                dest_path.key.clone(),
                source_object.hash,
                source_object.recovery_hash,
                source_object.size,
                source_object.metadata.clone(),
                4, // data_shards - default
                2, // parity_shards - default
            )
            .await
            .context("Failed to add object at destination")?;

            println!("✓ Added at destination: {}", dest_path.key);

            // Delete from source location
            bucket::delete_object(
                &mut bound_client,
                source_path.bucket_address,
                source_path.key.clone(),
            )
            .await
            .context("Failed to delete source object")?;

            println!("✓ Deleted from source: {}", source_path.key);
        } else {
            // Moving across buckets requires re-upload
            println!("Moving across buckets requires downloading and re-uploading...");
            return Err(anyhow!(
                "Cross-bucket move not yet implemented. Use 'cp' followed by 'rm' instead."
            ));
        }

        println!("✓ Move complete");

        Ok(())
    }
}
