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
use fendermint_vm_actor_interface::eam::EthAddress;
use fvm_shared::address::Address;
use fvm_shared::chainid::ChainID;

use crate::commands::storage::{
    bucket,
    client_context::resolve_write_context,
    config::{resolve_client_gateway_url},
    gateway::GatewayClient,
    path,
};
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

    async fn handle(global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let source_path = path::StoragePath::parse(&args.source)?;
        let dest_path = path::StoragePath::parse(&args.dest)?;

        if source_path.is_bucket_root() || dest_path.is_bucket_root() {
            return Err(anyhow!(
                "Paths must include file keys, not just bucket addresses"
            ));
        }

        println!("Moving {} -> {}", source_path.to_uri(), dest_path.to_uri());

        let write_ctx = resolve_write_context(global, args.config.clone())?;
        let rpc_url = write_ctx.rpc_url;
        let secret_key = write_ctx.secret_key;

        // Create clients
        let fm_client = FendermintClient::new_http(rpc_url.parse()?, None)?;

        // Get source object metadata via list_objects (reliable — no liveness check)
        let listed = bucket::list_objects(
            &fm_client,
            source_path.bucket_address,
            Some(source_path.key.clone()),
            None,
            None,
            16,
        )
        .await
        .with_context(|| {
            format!(
                "Failed to query source object (bucket={} key={} rpc={})",
                source_path.bucket_address, source_path.key, rpc_url
            )
        })?;

        let key_bytes = source_path.key.as_bytes();
        let source_object = listed
            .objects
            .iter()
            .find(|(k, _)| k.as_slice() == key_bytes)
            .map(|(_, o)| o)
            .ok_or_else(|| anyhow!("Source object not found: {}", source_path.key))?;

        // Query chain ID from the network
        let chain_id = bucket::query_chain_id(&fm_client)
            .await
            .with_context(|| format!("Failed to query chain ID from rpc={}", rpc_url))?;

        let pub_key = secret_key.public_key();
        let eth_addr = EthAddress::new_secp256k1(&pub_key.serialize())
            .context("failed to derive delegated address")?;
        let addr =
            Address::new_delegated(10, &eth_addr.0).context("failed to construct f410 address")?;
        let state = fm_client
            .actor_state(
                &addr,
                fendermint_vm_message::query::FvmQueryHeight::default(),
            )
            .await
            .with_context(|| format!("Failed to get actor state for sender {} via rpc={}", addr, rpc_url))?;
        let sequence = state.value.map(|(_, s)| s.sequence).ok_or_else(|| {
            anyhow!(
                "sender actor {} does not exist on-chain at {}. Fund/initialize this delegated \
                 address first.",
                addr,
                rpc_url
            )
        })?;

        // Check if the destination already exists before binding (fm_client is consumed by bind).
        // mv semantics are to replace the destination, but addObject has no overwrite flag in the
        // EVM interface, so we delete the destination first when it exists.
        let dest_exists = if source_path.bucket_address == dest_path.bucket_address {
            let dest_listed = bucket::list_objects(
                &fm_client,
                dest_path.bucket_address,
                Some(dest_path.key.clone()),
                None,
                None,
                2,
            )
            .await
            .context("Failed to check destination existence")?;
            let dest_key_bytes = dest_path.key.as_bytes();
            dest_listed
                .objects
                .iter()
                .any(|(k, _)| k.as_slice() == dest_key_bytes)
        } else {
            false
        };

        let mf = SignedMessageFactory::new(secret_key, addr, sequence, ChainID::from(chain_id));
        let mut bound_client = fm_client.bind(mf);

        // If moving within the same bucket, we can reuse the blob hash
        if source_path.bucket_address == dest_path.bucket_address {
            println!("Moving within same bucket (reusing blob)...");

            if dest_exists {
                bucket::delete_object(
                    &mut bound_client,
                    dest_path.bucket_address,
                    dest_path.key.clone(),
                )
                .await
                .context("Failed to clear existing destination object")?;
            }

            // Get source node ID from gateway
            let gateway_url =
                resolve_client_gateway_url(args.gateway.as_deref(), args.config.clone(), true)?;
            let gateway_client = GatewayClient::new(gateway_url.clone())?;
            let node_info = gateway_client.get_node_info().await.with_context(|| {
                format!(
                    "Failed to fetch gateway node info from {} (needed for move source node id)",
                    gateway_url
                )
            })?;
            let source_node =
                bucket::hash_to_b256(&node_info.node_id).context("Invalid node ID from gateway")?;

            bucket::add_object(
                &mut bound_client,
                dest_path.bucket_address,
                source_node,
                dest_path.key.clone(),
                source_object.hash,
                source_object.hash, // recovery_hash: reuse blob hash (ObjectState has no separate recovery_hash)
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
