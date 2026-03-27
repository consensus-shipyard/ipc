// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Move/rename command for storage objects

use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::path::PathBuf;

use async_trait::async_trait;
use fendermint_rpc::client::FendermintClient;
use fendermint_rpc::message::SignedMessageFactory;
use fendermint_rpc::QueryClient;
use fvm_shared::chainid::ChainID;

use crate::commands::storage::{bucket, client::GatewayClient, config::StorageConfig, path};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
pub struct MoveArgs {
    /// Source storage path (ipc://bucket_address/path/to/file)
    #[arg(value_name = "SOURCE")]
    pub source: String,

    /// Destination storage path (ipc://bucket_address/path/to/newfile)
    #[arg(value_name = "DEST")]
    pub dest: String,

    /// Gateway URL (overrides config and env var)
    #[arg(long)]
    pub gateway: Option<String>,

    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,
}

pub struct MoveStorage;

#[async_trait]
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

        let mut config = if config_path.exists() {
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

        // Query chain ID from the network
        let chain_id = bucket::query_chain_id(&fm_client)
            .await
            .context("Failed to query chain ID")?;

        // Create bound client for transactions
        let secret_key = SignedMessageFactory::read_secret_key(
            &config.secret_key_file
        )?;

        let addr = fvm_shared::address::Address::new_secp256k1(
            &secret_key.public_key().serialize(),
        )?;
        let state = fm_client
            .actor_state(&addr, fendermint_vm_message::query::FvmQueryHeight::default())
            .await
            .context("Failed to get actor state")?;
        let sequence = state.value.map(|(_, s)| s.sequence).unwrap_or(0);

        let mf = SignedMessageFactory::new(secret_key, addr, sequence, ChainID::from(chain_id));
        let mut bound_client = fm_client.bind(mf);

        // If moving within the same bucket, we can reuse the blob hash
        if source_path.bucket_address == dest_path.bucket_address {
            println!("Moving within same bucket (reusing blob)...");

            // Get source node ID from gateway
            let gateway_url = config.get_gateway_url(
                args.gateway.as_deref(),
                Some(&config_path),
                true,
            )?;
            let gateway_client = GatewayClient::new(gateway_url)?;
            let node_info = gateway_client.get_node_info().await?;
            let source_node = bucket::hex_to_b256(&node_info.node_id)
                .context("Invalid node ID from gateway")?;

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
