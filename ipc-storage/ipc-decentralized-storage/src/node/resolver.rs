// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Blob resolution and event handling for the storage node
//!
//! This module provides:
//! - Blob resolution by downloading from source nodes
//! - Event polling for blob finalization and deletion using ethers-rs

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use bls_signatures::{PrivateKey as BlsPrivateKey, Serialize as BlsSerialize};
use erasure_encoding::{shard_node, shards_for_node, BlobId, NodeId, DEFAULT_MAX_CHUNK_SIZE};
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use futures::StreamExt;
use iroh::NodeAddr;
use iroh_blobs::Hash;
use iroh_manager::IrohNode;
use std::str::FromStr;
use tracing::{debug, error, info, warn};

use super::store::Store;
use super::SignatureStorage;
use crate::distribution::{shard_key, NodeRpcDirectory};

// Event signatures for blob events (keccak256 of the event signature)
// BlobFinalized(address indexed subscriber, bytes32 hash, bool resolved)
const BLOB_FINALIZED_TOPIC: &str =
    "0x3f5b99de731555264580d7e2f00e46919de0d4f067a01d28aed55632a9068595";
// BlobDeleted(address indexed subscriber, bytes32 hash, uint256 size, uint256 bytesReleased)
const BLOB_DELETED_TOPIC: &str =
    "0x1ebbc934d9a1e5c0c9bcb94c6a7c55bfa2b66fca0a5d8ed66f0b43a5c8e3c0d8";

/// Configuration for the event poller
#[derive(Clone)]
pub struct EventPollerConfig {
    /// Ethereum JSON-RPC URL (Fendermint ETH API endpoint)
    pub eth_rpc_url: String,
    /// Polling interval
    pub poll_interval: Duration,
    /// Blobs actor address to filter events from
    pub blobs_actor_address: Address,
}

/// Events that the poller can detect
#[derive(Debug, Clone)]
pub enum BlobEvent {
    /// A blob has been finalized
    Finalized { hash: Hash },
    /// A blob has been deleted
    Deleted { hash: Hash },
}

/// Poll for blob events (finalized and deleted) using ethers-rs get_logs
///
/// This function polls the chain for new blocks and processes events
/// related to blob finalization and deletion.
pub async fn poll_for_blob_events<S: Store>(
    config: EventPollerConfig,
    signatures: SignatureStorage,
    store: Arc<S>,
    iroh: IrohNode,
) -> Result<()> {
    info!("Starting event poller for BlobFinalized and BlobDeleted events");
    info!("ETH RPC URL: {}", config.eth_rpc_url);
    info!("Poll interval: {:?}", config.poll_interval);
    info!("Blobs actor address: {:?}", config.blobs_actor_address);

    // Create ethers HTTP provider
    let provider = Provider::<Http>::try_from(&config.eth_rpc_url)
        .context("failed to create HTTP provider")?;

    loop {
        if let Err(e) = poll_once(&provider, &config, &signatures, &store, &iroh).await {
            error!("Error during event polling: {}", e);
        }

        tokio::time::sleep(config.poll_interval).await;
    }
}

/// Perform a single poll iteration
async fn poll_once<S: Store>(
    provider: &Provider<Http>,
    config: &EventPollerConfig,
    signatures: &SignatureStorage,
    store: &Arc<S>,
    iroh: &IrohNode,
) -> Result<()> {
    // Get the latest block number
    let latest_block = provider
        .get_block_number()
        .await
        .context("failed to get block number")?;
    let latest_height = latest_block.as_u64();

    // Get the last polled height from store
    let last_polled = store.get_last_polled_height()?.unwrap_or(0);

    if latest_height <= last_polled {
        debug!(
            "No new blocks to process (latest: {}, last polled: {})",
            latest_height, last_polled
        );
        return Ok(());
    }

    let from_block = last_polled + 1;
    debug!("Processing blocks from {} to {}", from_block, latest_height);

    // Build filter for BlobFinalized events
    let finalized_filter = Filter::new()
        .address(config.blobs_actor_address)
        .topic0(BLOB_FINALIZED_TOPIC.parse::<H256>().unwrap())
        .from_block(from_block)
        .to_block(latest_height);

    // Build filter for BlobDeleted events
    let deleted_filter = Filter::new()
        .address(config.blobs_actor_address)
        .topic0(BLOB_DELETED_TOPIC.parse::<H256>().unwrap())
        .from_block(from_block)
        .to_block(latest_height);

    // Query for BlobFinalized events
    let finalized_logs = provider
        .get_logs(&finalized_filter)
        .await
        .context("failed to get BlobFinalized logs")?;

    for log in finalized_logs {
        if let Some(event) = parse_blob_finalized_log(&log) {
            handle_blob_event(event, signatures, iroh).await;
        }
    }

    // Query for BlobDeleted events
    let deleted_logs = provider
        .get_logs(&deleted_filter)
        .await
        .context("failed to get BlobDeleted logs")?;

    for log in deleted_logs {
        if let Some(event) = parse_blob_deleted_log(&log) {
            handle_blob_event(event, signatures, iroh).await;
        }
    }

    // Update the last polled height
    store.set_last_polled_height(latest_height)?;
    debug!("Updated last polled height to {}", latest_height);

    Ok(())
}

/// Parse a BlobFinalized event from a log
/// Event: BlobFinalized(address indexed subscriber, bytes32 hash, bool resolved)
fn parse_blob_finalized_log(log: &Log) -> Option<BlobEvent> {
    // The hash is the second topic (first non-indexed param in data, but hash is in data)
    // Actually, looking at the event signature:
    // event BlobFinalized(address indexed subscriber, bytes32 hash, bool resolved);
    // - subscriber is indexed (topic1)
    // - hash is not indexed (in data)
    // - resolved is not indexed (in data)

    if log.data.len() < 64 {
        debug!("BlobFinalized log data too short: {} bytes", log.data.len());
        return None;
    }

    // First 32 bytes of data is the hash
    let hash_bytes: [u8; 32] = log.data[0..32].try_into().ok()?;
    let hash = Hash::from(hash_bytes);

    Some(BlobEvent::Finalized { hash })
}

/// Parse a BlobDeleted event from a log
/// Event: BlobDeleted(address indexed subscriber, bytes32 hash, uint256 size, uint256 bytesReleased)
fn parse_blob_deleted_log(log: &Log) -> Option<BlobEvent> {
    // - subscriber is indexed (topic1)
    // - hash is not indexed (in data, first 32 bytes)
    // - size is not indexed (in data)
    // - bytesReleased is not indexed (in data)

    if log.data.len() < 96 {
        debug!("BlobDeleted log data too short: {} bytes", log.data.len());
        return None;
    }

    // First 32 bytes of data is the hash
    let hash_bytes: [u8; 32] = log.data[0..32].try_into().ok()?;
    let hash = Hash::from(hash_bytes);

    Some(BlobEvent::Deleted { hash })
}

/// Handle a blob event
async fn handle_blob_event(event: BlobEvent, signatures: &SignatureStorage, iroh: &IrohNode) {
    match event {
        BlobEvent::Finalized { hash } => {
            // Remove signature from memory for finalized blobs
            let mut sigs = signatures.write().unwrap();
            if sigs.remove(&hash).is_some() {
                info!("Removed signature for finalized blob {} from memory", hash);
            } else {
                debug!(
                    "Blob {} was finalized but no signature found in memory",
                    hash
                );
            }
        }
        BlobEvent::Deleted { hash } => {
            // Remove signature from memory
            {
                let mut sigs = signatures.write().unwrap();
                if sigs.remove(&hash).is_some() {
                    info!("Removed signature for deleted blob {} from memory", hash);
                }
            }

            // Optionally delete the blob from Iroh storage
            // Note: This is a best-effort cleanup, failures are logged but not fatal
            match delete_blob_from_iroh(iroh, hash).await {
                Ok(deleted) => {
                    if deleted {
                        info!("Deleted blob {} from Iroh storage", hash);
                    } else {
                        debug!("Blob {} was not found in Iroh storage", hash);
                    }
                }
                Err(e) => {
                    warn!("Failed to delete blob {} from Iroh storage: {}", hash, e);
                }
            }
        }
    }
}

/// Delete a blob's shard data from Iroh storage.
///
/// Iterates all Iroh tags and deletes any whose name starts with the blob's
/// hex prefix, covering all `{blob_hex}/{chunk}/{shard}` tags.
async fn delete_blob_from_iroh(iroh: &IrohNode, hash: Hash) -> Result<bool> {
    let blob_hex = hex::encode(hash.as_bytes());
    let prefix = format!("{}/", blob_hex);

    let mut tags = iroh.blobs_client().tags().list().await?;
    let mut deleted_any = false;

    while let Some(Ok(tag_info)) = tags.next().await {
        let tag_name = std::str::from_utf8(tag_info.name.0.as_ref()).unwrap_or("");
        if tag_name.starts_with(&prefix) {
            debug!("Deleting shard tag: {}", tag_name);
            let _ = iroh.blobs_client().tags().delete(tag_info.name).await;
            deleted_any = true;
        }
    }

    Ok(deleted_any)
}

/// Response from the shard hash lookup endpoint.
#[derive(serde::Deserialize)]
struct ShardHashLookupResponse {
    hash: String,
    node_addr: NodeAddr,
}

/// Resolve a blob by downloading assigned shards from other operators.
///
/// 1. Computes which shards are assigned to this node
/// 2. Checks which are already stored locally
/// 3. For missing shards, queries other operators for the shard hash and downloads via Iroh P2P
/// 4. Signs the blob hash with BLS key once all assigned shards are present
#[allow(clippy::too_many_arguments)]
pub async fn resolve_blob_shards(
    iroh: IrohNode,
    blob_hash: fendermint_actor_blobs_shared::bytes::B256,
    size: u64,
    data_shards: usize,
    parity_shards: usize,
    nodes: Vec<NodeId>,
    node_rpc_directory: NodeRpcDirectory,
    our_node_id: NodeId,
    bls_private_key: BlsPrivateKey,
    signatures: SignatureStorage,
) -> Result<()> {
    let blob_id = BlobId(blob_hash.0);
    let blob_iroh_hash = Hash::from_bytes(blob_hash.0);
    let num_chunks = (size as usize).div_ceil(DEFAULT_MAX_CHUNK_SIZE);
    let shards_per_chunk = data_shards + parity_shards;

    info!(
        "Resolving blob {} shards: {} chunks, k={}, m={}, size={}",
        blob_hash, num_chunks, data_shards, parity_shards, size
    );

    // Compute which shards are assigned to this node
    let assigned = shards_for_node(
        &blob_id,
        num_chunks,
        data_shards,
        parity_shards,
        &nodes,
        &our_node_id,
    );

    info!(
        "Node has {} assigned shards for blob {}",
        assigned.len(),
        blob_hash
    );

    let mut missing_shards = Vec::new();

    // Check which assigned shards are already stored locally
    for &(chunk_idx, shard_idx) in &assigned {
        let tag = shard_key(&blob_id, chunk_idx, shard_idx);
        let iroh_tag = iroh_blobs::Tag(tag.clone().into());

        let found = {
            let mut tags = iroh.blobs_client().tags().list().await?;
            let mut found = false;
            while let Some(Ok(tag_info)) = tags.next().await {
                if tag_info.name == iroh_tag {
                    found = true;
                    break;
                }
            }
            found
        };

        if found {
            debug!("Shard {}/{} already stored locally", chunk_idx, shard_idx);
        } else {
            missing_shards.push((chunk_idx, shard_idx));
        }
    }

    if missing_shards.is_empty() {
        info!("All assigned shards already present for blob {}", blob_hash);
    } else {
        info!(
            "Need to fetch {} missing shards for blob {}",
            missing_shards.len(),
            blob_hash
        );

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("failed to create HTTP client")?;

        for (chunk_idx, shard_idx) in missing_shards {
            // Find which operator holds this shard
            let holder = shard_node(&blob_id, chunk_idx, shard_idx, shards_per_chunk, &nodes);

            let rpc_url = node_rpc_directory.get(&holder).ok_or_else(|| {
                anyhow::anyhow!(
                    "No RPC URL for node {:?} holding shard {}/{}",
                    holder,
                    chunk_idx,
                    shard_idx
                )
            })?;

            let blob_id_hex = hex::encode(blob_id.0);
            let url = format!(
                "{}/v1/shards/{}/{}/{}/hash",
                rpc_url.trim_end_matches('/'),
                blob_id_hex,
                chunk_idx,
                shard_idx
            );

            debug!("Querying shard hash from {}", url);

            let resp = http_client
                .get(&url)
                .send()
                .await
                .with_context(|| format!("failed to query shard hash from {}", url))?;

            if !resp.status().is_success() {
                anyhow::bail!(
                    "Shard hash lookup failed for {}/{}: HTTP {}",
                    chunk_idx,
                    shard_idx,
                    resp.status()
                );
            }

            let lookup: ShardHashLookupResponse = resp
                .json()
                .await
                .context("failed to parse shard hash response")?;

            let shard_hash = Hash::from_str(&lookup.hash)
                .map_err(|_| anyhow::anyhow!("invalid hash in shard lookup response"))?;

            let tag = shard_key(&blob_id, chunk_idx, shard_idx);

            // Download shard via Iroh P2P
            info!(
                "Downloading shard {}/{} (hash={}) from node",
                chunk_idx, shard_idx, shard_hash
            );

            let progress = iroh
                .blobs_client()
                .download_with_opts(
                    shard_hash,
                    iroh_blobs::rpc::client::blobs::DownloadOptions {
                        format: iroh_blobs::BlobFormat::Raw,
                        nodes: vec![lookup.node_addr],
                        tag: iroh_blobs::util::SetTagOption::Named(iroh_blobs::Tag(tag.into())),
                        mode: iroh_blobs::rpc::client::blobs::DownloadMode::Queued,
                    },
                )
                .await
                .with_context(|| {
                    format!("failed to start shard {}/{} download", chunk_idx, shard_idx)
                })?;

            let outcome = progress.finish().await.with_context(|| {
                format!(
                    "shard {}/{} download did not complete",
                    chunk_idx, shard_idx
                )
            })?;

            info!(
                "Downloaded shard {}/{} (downloaded: {} bytes, local: {} bytes)",
                chunk_idx, shard_idx, outcome.downloaded_size, outcome.local_size
            );
        }
    }

    // All assigned shards are now present — sign the blob hash
    let signature = bls_private_key.sign(blob_iroh_hash.as_bytes());
    let signature_bytes = signature.as_bytes();

    {
        let mut sigs = signatures.write().unwrap();
        sigs.insert(blob_iroh_hash, signature_bytes.clone());
    }

    info!(
        "Generated BLS signature for blob {} (all {} assigned shards present)",
        blob_hash,
        assigned.len()
    );

    Ok(())
}
