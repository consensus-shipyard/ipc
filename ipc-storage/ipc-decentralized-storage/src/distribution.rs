// Copyright 2025 Recall Contributors
// SPDX-License-Identifier: Apache-2.0, MIT

//! Data distribution module for erasure-encoded shard distribution.
//!
//! Orchestrates encoding data into shards via Reed-Solomon erasure coding,
//! deterministically assigning shards to storage nodes, and notifying nodes
//! to pull their assigned shards via Iroh P2P.

use std::collections::HashMap;

use anyhow::{Context, Result};
use erasure_encoding::{
    encode_and_assign, BlobId, DeterministicAssigner, EncodedChunk, EncodingMetadata, NodeId,
    ReedSolomonEncoder,
};
use iroh::NodeAddr;
use iroh_blobs::Hash;
use iroh_manager::BlobsClient;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Maps an erasure-encoding NodeId to an Iroh NodeAddr for P2P connectivity.
/// Populated from on-chain operator info at the encoding epoch.
pub type NodeDirectory = HashMap<NodeId, NodeAddr>;

/// Maps an erasure-encoding NodeId to the node's RPC URL.
pub type NodeRpcDirectory = HashMap<NodeId, String>;

/// Parameters for distributing a blob's shards.
pub struct DistributeParams {
    pub blob_id: BlobId,
    /// The encrypted data to encode and distribute.
    pub data: Vec<u8>,
    /// Number of data shards per chunk (k).
    pub data_shards: usize,
    /// Number of parity shards per chunk (m).
    pub parity_shards: usize,
    /// Ordered node list from on-chain state at encoding epoch.
    pub nodes: Vec<NodeId>,
    /// Mapping from NodeId to Iroh NodeAddr for P2P connectivity.
    pub node_directory: NodeDirectory,
    /// Mapping from NodeId to RPC URL for pull notifications.
    pub node_rpc_directory: NodeRpcDirectory,
}

/// Result of distributing a single shard.
#[derive(Debug)]
pub struct ShardDistributionResult {
    pub chunk_index: usize,
    pub shard_index: usize,
    pub node: NodeId,
    pub iroh_hash: Option<Hash>,
    pub success: bool,
    pub error: Option<String>,
}

/// Result of distributing an entire blob.
#[derive(Debug)]
pub struct DistributionResult {
    pub metadata: EncodingMetadata,
    pub shard_results: Vec<ShardDistributionResult>,
}

impl DistributionResult {
    pub fn all_succeeded(&self) -> bool {
        self.shard_results.iter().all(|r| r.success)
    }

    pub fn failure_count(&self) -> usize {
        self.shard_results.iter().filter(|r| !r.success).count()
    }
}

/// Deterministic shard storage key following DESIGN.md:
///   key = blob_id / chunk_index / shard_index
pub fn shard_key(blob_id: &BlobId, chunk_index: usize, shard_index: usize) -> String {
    let blob_hex = hex::encode(blob_id.0);
    format!("{}/{}/{}", blob_hex, chunk_index, shard_index)
}

/// Request body sent to a storage node's pull endpoint.
#[derive(Debug, Serialize, Deserialize)]
pub struct ShardPullRequest {
    pub blob_id: String,
    pub chunk_index: usize,
    pub shard_index: usize,
    pub shards_per_chunk: usize,
    pub hash: String,
    pub source: NodeAddr,
}

/// Encode data and distribute shards to their assigned nodes.
///
/// 1. Erasure-encode the data into chunks and shards
/// 2. Assign shards to nodes deterministically via DeterministicAssigner
/// 3. Store each shard locally in Iroh under a deterministic tag
/// 4. Notify each target node via RPC to pull the shard from us
pub async fn distribute(
    params: DistributeParams,
    local_blobs: &BlobsClient,
    local_node_addr: &NodeAddr,
) -> Result<DistributionResult> {
    let assigner = DeterministicAssigner::new(params.blob_id, params.nodes.len());

    let (metadata, chunk_iter) = encode_and_assign::<ReedSolomonEncoder, _>(
        &params.data,
        params.data_shards,
        params.parity_shards,
        &params.nodes,
        assigner,
    )?;

    info!(
        "Encoded blob: {} chunks, k={}, m={}, original_len={}, input_data_len={}",
        metadata.num_chunks, metadata.data_shards, metadata.parity_shards, metadata.original_len,
        params.data.len()
    );

    let chunks: Vec<EncodedChunk> = chunk_iter
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("erasure encoding failed")?;

    let shards_per_chunk = params.data_shards + params.parity_shards;
    let mut shard_results = Vec::new();

    for chunk in &chunks {
        info!(
            "Chunk {}: original_data_len={}, num_shards={}",
            chunk.chunk_index, chunk.original_data_len, chunk.shards.len()
        );
        for assigned_shard in &chunk.shards {
            let tag = shard_key(&params.blob_id, chunk.chunk_index, assigned_shard.shard.index);

            info!(
                "Shard {}/{}: data_len={}, assigned_to={:?}",
                chunk.chunk_index, assigned_shard.shard.index,
                assigned_shard.shard.data.len(),
                hex::encode(assigned_shard.node.0)
            );

            // Step 1: Store shard locally
            // add_bytes_named(data, tag_name)
            let store_result = local_blobs
                .add_bytes_named(assigned_shard.shard.data.clone(), tag.clone())
                .await
                .context("failed to store shard locally");

            let hash = match store_result {
                Ok(outcome) => outcome.hash,
                Err(e) => {
                    warn!(
                        "Failed to store shard {}/{} locally: {}",
                        chunk.chunk_index, assigned_shard.shard.index, e
                    );
                    shard_results.push(ShardDistributionResult {
                        chunk_index: chunk.chunk_index,
                        shard_index: assigned_shard.shard.index,
                        node: assigned_shard.node,
                        iroh_hash: None,
                        success: false,
                        error: Some(e.to_string()),
                    });
                    continue;
                }
            };

            // Step 2: Notify the target node to pull the shard from us
            let rpc_url = params.node_rpc_directory.get(&assigned_shard.node);
            let notify_result = match rpc_url {
                Some(url) => {
                    notify_node_to_pull(
                        url,
                        &params.blob_id,
                        chunk.chunk_index,
                        assigned_shard.shard.index,
                        shards_per_chunk,
                        hash,
                        local_node_addr,
                    )
                    .await
                }
                None => Err(anyhow::anyhow!(
                    "No RPC URL for node {:?}",
                    assigned_shard.node
                )),
            };

            match &notify_result {
                Ok(()) => {
                    debug!(
                        "Notified node {:?} to pull shard {}/{} (hash={})",
                        assigned_shard.node, chunk.chunk_index, assigned_shard.shard.index, hash
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to notify node {:?} for shard {}/{}: {}",
                        assigned_shard.node, chunk.chunk_index, assigned_shard.shard.index, e
                    );
                }
            }

            shard_results.push(ShardDistributionResult {
                chunk_index: chunk.chunk_index,
                shard_index: assigned_shard.shard.index,
                node: assigned_shard.node,
                iroh_hash: Some(hash),
                success: notify_result.is_ok(),
                error: notify_result.err().map(|e| e.to_string()),
            });
        }
    }

    Ok(DistributionResult {
        metadata,
        shard_results,
    })
}

/// Notify a storage node via its RPC endpoint to pull a shard from us.
async fn notify_node_to_pull(
    rpc_url: &str,
    blob_id: &BlobId,
    chunk_index: usize,
    shard_index: usize,
    shards_per_chunk: usize,
    hash: Hash,
    source: &NodeAddr,
) -> Result<()> {
    let request = ShardPullRequest {
        blob_id: hex::encode(blob_id.0),
        chunk_index,
        shard_index,
        shards_per_chunk,
        hash: hash.to_string(),
        source: source.clone(),
    };

    let url = format!("{}/v1/shards/pull", rpc_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .context("failed to send pull notification")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "no body".to_string());
        anyhow::bail!("node returned {}: {}", status, body);
    }

    Ok(())
}
