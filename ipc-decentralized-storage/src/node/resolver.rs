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
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use iroh_blobs::Hash;
use iroh_manager::IrohNode;
use tracing::{debug, error, info, warn};

use super::store::Store;
use super::SignatureStorage;

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

/// Delete a blob and its associated content from Iroh storage
async fn delete_blob_from_iroh(iroh: &IrohNode, hash: Hash) -> Result<bool> {
    use iroh_blobs::hashseq::HashSeq;

    // First, try to read the hash sequence to get all associated hashes
    let hash_seq_bytes = match iroh.blobs_client().read_to_bytes(hash).await {
        Ok(bytes) => bytes,
        Err(_) => {
            // Blob not found, nothing to delete
            return Ok(false);
        }
    };

    // Parse the hash sequence
    let content_hashes: Vec<Hash> = match HashSeq::try_from(hash_seq_bytes) {
        Ok(seq) => seq.iter().collect(),
        Err(e) => {
            warn!("Failed to parse hash sequence for {}: {}", hash, e);
            // Still try to delete the main hash
            vec![]
        }
    };

    // Delete the hash sequence blob tag
    let seq_tag = iroh_blobs::Tag(format!("blob-seq-{}", hash).into());
    let _ = iroh.blobs_client().tags().delete(seq_tag).await;

    // Delete content blob tags
    for content_hash in &content_hashes {
        let content_tag = iroh_blobs::Tag(format!("blob-{}-{}", hash, content_hash).into());
        let _ = iroh.blobs_client().tags().delete(content_tag).await;
    }

    Ok(true)
}

/// Resolve a blob by downloading it from one of its sources
///
/// Downloads the hash sequence and all blobs referenced within it (including original content).
/// Returns Ok(()) if the blob was successfully downloaded, Err otherwise.
pub async fn resolve_blob(
    iroh: IrohNode,
    hash: Hash,
    size: u64,
    sources: std::collections::HashSet<(
        fvm_shared::address::Address,
        fendermint_actor_blobs_shared::blobs::SubscriptionId,
        iroh::NodeId,
    )>,
    bls_private_key: BlsPrivateKey,
    signatures: SignatureStorage,
) -> Result<()> {
    use iroh_blobs::hashseq::HashSeq;

    info!("Resolving blob: {} (size: {})", hash, size);
    debug!("Sources: {} available", sources.len());

    // Try each source until one succeeds
    for (_subscriber, _id, source_node_id) in sources {
        debug!("Attempting download from source: {}", source_node_id);

        // Create a NodeAddr from the source
        let source_addr = iroh::NodeAddr::new(source_node_id);

        // Step 1: Download the hash sequence blob
        match iroh
            .blobs_client()
            .download_with_opts(
                hash,
                iroh_blobs::rpc::client::blobs::DownloadOptions {
                    format: iroh_blobs::BlobFormat::Raw,
                    nodes: vec![source_addr.clone()],
                    tag: iroh_blobs::util::SetTagOption::Named(iroh_blobs::Tag(
                        format!("blob-seq-{}", hash).into(),
                    )),
                    mode: iroh_blobs::rpc::client::blobs::DownloadMode::Queued,
                },
            )
            .await
        {
            Ok(progress) => {
                match progress.finish().await {
                    Ok(outcome) => {
                        let downloaded_size = outcome.local_size + outcome.downloaded_size;
                        info!(
                            "Downloaded hash sequence {} (downloaded: {} bytes, local: {} bytes)",
                            hash, outcome.downloaded_size, outcome.local_size
                        );

                        // Step 2: Read and parse the hash sequence to get all referenced blobs
                        let hash_seq_bytes = match iroh.blobs_client().read_to_bytes(hash).await {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                warn!("Failed to read hash sequence {}: {}", hash, e);
                                continue;
                            }
                        };

                        let hash_seq = match HashSeq::try_from(hash_seq_bytes) {
                            Ok(seq) => seq,
                            Err(e) => {
                                warn!("Failed to parse hash sequence {}: {}", hash, e);
                                continue;
                            }
                        };

                        let content_hashes: Vec<Hash> = hash_seq.iter().collect();
                        info!(
                            "Hash sequence {} contains {} blobs to download",
                            hash,
                            content_hashes.len()
                        );

                        // Step 3: Download all blobs in the hash sequence
                        let mut all_downloaded = true;
                        for (idx, content_hash) in content_hashes.iter().enumerate() {
                            let blob_type = if idx == 0 {
                                "original content"
                            } else if idx == 1 {
                                "metadata"
                            } else {
                                "parity"
                            };

                            debug!(
                                "Downloading {} blob {} ({}/{}): {}",
                                blob_type,
                                content_hash,
                                idx + 1,
                                content_hashes.len(),
                                content_hash
                            );

                            match iroh
                                .blobs_client()
                                .download_with_opts(
                                    *content_hash,
                                    iroh_blobs::rpc::client::blobs::DownloadOptions {
                                        format: iroh_blobs::BlobFormat::Raw,
                                        nodes: vec![source_addr.clone()],
                                        tag: iroh_blobs::util::SetTagOption::Named(
                                            iroh_blobs::Tag(
                                                format!("blob-{}-{}", hash, content_hash).into(),
                                            ),
                                        ),
                                        mode: iroh_blobs::rpc::client::blobs::DownloadMode::Queued,
                                    },
                                )
                                .await
                            {
                                Ok(content_progress) => match content_progress.finish().await {
                                    Ok(content_outcome) => {
                                        debug!(
                                                "Downloaded {} blob {} (downloaded: {} bytes, local: {} bytes)",
                                                blob_type,
                                                content_hash,
                                                content_outcome.downloaded_size,
                                                content_outcome.local_size
                                            );
                                    }
                                    Err(e) => {
                                        warn!(
                                            "Failed to complete {} blob {} download: {}",
                                            blob_type, content_hash, e
                                        );
                                        all_downloaded = false;
                                    }
                                },
                                Err(e) => {
                                    warn!(
                                        "Failed to start {} blob {} download: {}",
                                        blob_type, content_hash, e
                                    );
                                    all_downloaded = false;
                                }
                            }
                        }

                        if !all_downloaded {
                            warn!(
                                "Not all content blobs downloaded for {}, trying next source",
                                hash
                            );
                            continue;
                        }

                        info!(
                            "Successfully resolved blob {} with all {} content blobs (expected original size: {} bytes)",
                            hash, content_hashes.len(), size
                        );

                        // Generate BLS signature for the blob hash
                        let hash_bytes = hash.as_bytes();
                        let signature = bls_private_key.sign(hash_bytes);
                        let signature_bytes = signature.as_bytes();

                        // Store signature in memory
                        {
                            let mut sigs = signatures.write().unwrap();
                            sigs.insert(hash, signature_bytes.clone());
                        }

                        info!("Generated BLS signature for blob {}", hash);
                        debug!("Signature: {}", hex::encode(&signature_bytes));
                        debug!("Hash sequence blob size: {} bytes", downloaded_size);

                        // Blob downloaded successfully
                        // It will now wait for validator signatures before finalization
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("Failed to complete download from {}: {}", source_node_id, e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to start download from {}: {}", source_node_id, e);
            }
        }
    }

    anyhow::bail!("Failed to resolve blob {} from any source", hash)
}
