// Copyright 2025 Recall Contributors
// SPDX-License-Identifier: Apache-2.0, MIT

//! Shard assignment verification for storage nodes.
//!
//! When a storage node receives a shard pull request, it uses this module
//! to verify that the shard is legitimately assigned to it per the
//! deterministic mapping.

use anyhow::{ensure, Result};
use erasure_encoding::{shard_node, BlobId, NodeId};

/// Verify that a shard is correctly assigned to the expected node.
///
/// Called by the storage node when receiving a shard pull request.
/// Returns `Ok(())` if the shard belongs to `expected_node`, error otherwise.
pub fn verify_shard_assignment(
    blob_id: &BlobId,
    chunk_index: usize,
    shard_index: usize,
    shards_per_chunk: usize,
    nodes: &[NodeId],
    expected_node: &NodeId,
) -> Result<()> {
    let assigned = shard_node(blob_id, chunk_index, shard_index, shards_per_chunk, nodes);
    ensure!(
        assigned == *expected_node,
        "Shard {}/{} is assigned to node {:?}, not {:?}",
        chunk_index,
        shard_index,
        assigned,
        expected_node
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nodes(n: usize) -> Vec<NodeId> {
        (0..n)
            .map(|i| {
                let mut id = [0u8; 32];
                id[0] = i as u8;
                NodeId(id)
            })
            .collect()
    }

    #[test]
    fn verify_correct_assignment() {
        let mut blob_bytes = [0u8; 32];
        blob_bytes[0] = 7;
        let blob_id = BlobId(blob_bytes);
        let nodes = make_nodes(5);
        let shards_per_chunk = 3;

        // blob_id [7, 0..] % 5 = 2, so shard (0, 0) → node[2]
        let expected = shard_node(&blob_id, 0, 0, shards_per_chunk, &nodes);
        assert!(verify_shard_assignment(&blob_id, 0, 0, shards_per_chunk, &nodes, &expected).is_ok());
    }

    #[test]
    fn reject_wrong_assignment() {
        let mut blob_bytes = [0u8; 32];
        blob_bytes[0] = 7;
        let blob_id = BlobId(blob_bytes);
        let nodes = make_nodes(5);
        let shards_per_chunk = 3;

        // node[0] is not the correct assignee for shard (0, 0)
        let wrong_node = nodes[0];
        let result = verify_shard_assignment(&blob_id, 0, 0, shards_per_chunk, &nodes, &wrong_node);
        assert!(result.is_err());
    }
}
