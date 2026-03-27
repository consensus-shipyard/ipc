// Copyright 2022-2024 Protocol Labs
// Copyright 2025 Recall Contributors
// SPDX-License-Identifier: Apache-2.0, MIT

//! Data retrieval module for erasure-encoded shard fetching and decoding.
//!
//! Derives the shard-to-node mapping from on-chain parameters, fetches
//! at least k shards per chunk from the assigned nodes, and RS-decodes
//! to recover the original data.

use anyhow::{Context, Result};
use erasure_encoding::{
    decode_chunks, shard_node, BlobId, ChunkRecoveryInput, NodeId, ReedSolomonEncoder, Shard,
    DEFAULT_MAX_CHUNK_SIZE,
};
use iroh::NodeAddr;
use iroh_blobs::Hash;
use iroh_manager::BlobsClient;
use tracing::{debug, info, warn};

use std::str::FromStr;

use crate::distribution::{shard_key, NodeDirectory, NodeRpcDirectory};

/// On-chain blob metadata needed for retrieval.
/// Corresponds to the StorageCommitment fields from DESIGN.md.
pub struct BlobRetrievalParams {
    pub blob_id: BlobId,
    pub original_len: usize,
    /// Number of data shards per chunk (k).
    pub data_shards: usize,
    /// Number of parity shards per chunk (m).
    pub parity_shards: usize,
    /// Node list at encoding epoch.
    pub nodes: Vec<NodeId>,
    /// Mapping from NodeId to Iroh NodeAddr.
    pub node_directory: NodeDirectory,
    /// Mapping from NodeId to RPC URL (for shard hash lookups).
    pub node_rpc_directory: NodeRpcDirectory,
}

impl BlobRetrievalParams {
    pub fn num_chunks(&self) -> usize {
        self.original_len.div_ceil(DEFAULT_MAX_CHUNK_SIZE)
    }

    pub fn chunk_data_len(&self, chunk_index: usize) -> usize {
        let start = chunk_index * DEFAULT_MAX_CHUNK_SIZE;
        DEFAULT_MAX_CHUNK_SIZE.min(self.original_len - start)
    }

    pub fn shards_per_chunk(&self) -> usize {
        self.data_shards + self.parity_shards
    }
}

/// Retrieve and decode a blob from the network.
///
/// 1. Derive chunk structure from original_len and MAX_CHUNK_SIZE
/// 2. For each chunk, compute shard→node mapping via shard_node()
/// 3. Fetch shards from assigned nodes (need only k, try all k+m, early-exit)
/// 4. RS-decode each chunk, concatenate, truncate to original_len
pub async fn retrieve(params: &BlobRetrievalParams, local_blobs: &BlobsClient) -> Result<Vec<u8>> {
    let num_chunks = params.num_chunks();
    let shards_per_chunk = params.shards_per_chunk();

    info!(
        "Retrieving blob: {} chunks, k={}, m={}, original_len={}",
        num_chunks, params.data_shards, params.parity_shards, params.original_len
    );

    let mut recovery_inputs = Vec::with_capacity(num_chunks);

    for chunk_idx in 0..num_chunks {
        let chunk_data_len = params.chunk_data_len(chunk_idx);
        let mut fetched_shards = Vec::new();

        for shard_idx in 0..shards_per_chunk {
            let node = shard_node(
                &params.blob_id,
                chunk_idx,
                shard_idx,
                shards_per_chunk,
                &params.nodes,
            );

            let node_addr = params.node_directory.get(&node);
            let node_rpc_url = params.node_rpc_directory.get(&node).map(|s| s.as_str());

            match fetch_shard(
                local_blobs,
                &params.blob_id,
                chunk_idx,
                shard_idx,
                node_addr,
                node_rpc_url,
            )
            .await
            {
                Ok(data) => {
                    fetched_shards.push(Shard {
                        index: shard_idx,
                        data,
                    });
                    if fetched_shards.len() >= params.data_shards {
                        debug!(
                            "Chunk {}: collected {} shards (k={}), sufficient",
                            chunk_idx,
                            fetched_shards.len(),
                            params.data_shards
                        );
                        break;
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch shard {}/{} from node {:?}: {}",
                        chunk_idx, shard_idx, node, e
                    );
                    let remaining = shards_per_chunk - (shard_idx + 1);
                    if fetched_shards.len() + remaining < params.data_shards {
                        anyhow::bail!(
                            "Chunk {}: cannot collect enough shards. Have {}, need {}, {} remaining",
                            chunk_idx,
                            fetched_shards.len(),
                            params.data_shards,
                            remaining
                        );
                    }
                }
            }
        }

        if fetched_shards.len() < params.data_shards {
            anyhow::bail!(
                "Chunk {}: only fetched {} shards, need {}",
                chunk_idx,
                fetched_shards.len(),
                params.data_shards
            );
        }

        recovery_inputs.push(ChunkRecoveryInput {
            chunk_index: chunk_idx,
            original_data_len: chunk_data_len,
            shards: fetched_shards,
            num_data_shards: params.data_shards,
            num_parity_shards: params.parity_shards,
        });
    }

    let recovered = decode_chunks::<ReedSolomonEncoder>(&mut recovery_inputs, params.original_len)?;

    info!(
        "Successfully retrieved and decoded {} bytes",
        recovered.len()
    );
    Ok(recovered)
}

/// Fetch a single shard, first checking local Iroh then downloading from the remote node.
async fn fetch_shard(
    blobs: &BlobsClient,
    blob_id: &BlobId,
    chunk_index: usize,
    shard_index: usize,
    _node_addr: Option<&NodeAddr>,
    node_rpc_url: Option<&str>,
) -> Result<Vec<u8>> {
    let tag = shard_key(blob_id, chunk_index, shard_index);

    // Try local first — shard may already be cached
    if let Ok(hash) = tag_to_hash_lookup(blobs, &tag).await {
        if let Ok(bytes) = blobs.read_to_bytes(hash).await {
            info!(
                "Shard {}/{} found locally (tag={}, hash={}, size={})",
                chunk_index,
                shard_index,
                tag,
                hash,
                bytes.len()
            );
            return Ok(bytes.to_vec());
        }
    }

    // Download from the assigned node
    let rpc_url = node_rpc_url.ok_or_else(|| anyhow::anyhow!("No RPC URL for shard {}", tag))?;

    // Query the node's RPC for the shard's Iroh hash and NodeAddr
    let (hash, source_addr) = query_shard_hash(rpc_url, blob_id, chunk_index, shard_index).await?;

    blobs
        .download_with_opts(
            hash,
            iroh_blobs::rpc::client::blobs::DownloadOptions {
                format: iroh_blobs::BlobFormat::Raw,
                nodes: vec![source_addr],
                tag: iroh_blobs::util::SetTagOption::Named(iroh_blobs::Tag(tag.into())),
                mode: iroh_blobs::rpc::client::blobs::DownloadMode::Queued,
            },
        )
        .await
        .context("failed to start shard download")?
        .finish()
        .await
        .context("shard download did not complete")?;

    let bytes = blobs
        .read_to_bytes(hash)
        .await
        .context("failed to read downloaded shard")?;
    Ok(bytes.to_vec())
}

/// Look up a locally-stored blob hash by its named tag.
pub async fn tag_to_hash_lookup(blobs: &BlobsClient, tag: &str) -> Result<Hash> {
    use futures::StreamExt;
    let iroh_tag = iroh_blobs::Tag(tag.to_string().into());
    let mut tags = blobs.tags().list().await?;
    while let Some(Ok(tag_info)) = tags.next().await {
        if tag_info.name == iroh_tag {
            return Ok(tag_info.hash);
        }
    }
    anyhow::bail!("tag not found: {}", tag)
}

/// Response from the shard hash lookup endpoint.
#[derive(serde::Deserialize)]
struct ShardHashResponse {
    hash: String,
    node_addr: NodeAddr,
}

/// Query a storage node's RPC to get the Iroh hash for a specific shard.
async fn query_shard_hash(
    rpc_url: &str,
    blob_id: &BlobId,
    chunk_index: usize,
    shard_index: usize,
) -> Result<(Hash, NodeAddr)> {
    let blob_id_hex = hex::encode(blob_id.0);
    let url = format!(
        "{}/v1/shards/{}/{}/{}/hash",
        rpc_url.trim_end_matches('/'),
        blob_id_hex,
        chunk_index,
        shard_index
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .context("failed to create HTTP client")?;

    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("failed to query shard hash from {}", url))?;

    if !resp.status().is_success() {
        anyhow::bail!("shard hash lookup failed: HTTP {}", resp.status());
    }

    let response: ShardHashResponse = resp
        .json()
        .await
        .context("failed to parse shard hash response")?;

    let hash = Hash::from_str(&response.hash)
        .map_err(|_| anyhow::anyhow!("invalid hash in shard lookup response"))?;

    Ok((hash, response.node_addr))
}
