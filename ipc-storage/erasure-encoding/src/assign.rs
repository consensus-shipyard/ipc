// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::traits::NodeAssigner;
use crate::types::{AssignedShard, NodeId, Shard};

/// A 32-byte blob identifier used to derive deterministic rotation offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlobId(pub [u8; 32]);

impl BlobId {
    /// Compute `self % divisor` over all 32 bytes (big-endian big-number modulo).
    fn modulo(&self, divisor: usize) -> usize {
        let mut remainder: u64 = 0;
        for &byte in &self.0 {
            remainder = (remainder * 256 + byte as u64) % divisor as u64;
        }
        remainder as usize
    }
}

/// Deterministic node assigner using blob_id as starting rotation offset.
///
/// The blob_id determines where in the node list assignment begins.
/// Each subsequent shard advances by one position. This implements the
/// DESIGN.md formula:
/// ```text
/// rotation_offset = blob_id % num_nodes
/// shard_global    = chunk_index * (k + m) + shard_index
/// node            = nodes[(shard_global + rotation_offset) % num_nodes]
/// ```
pub struct DeterministicAssigner {
    rotation_offset: usize,
    position: usize,
}

impl DeterministicAssigner {
    pub fn new(blob_id: BlobId, num_nodes: usize) -> Self {
        Self {
            rotation_offset: blob_id.modulo(num_nodes),
            position: 0,
        }
    }
}

impl NodeAssigner for DeterministicAssigner {
    type Shard = Shard;

    fn assign(&mut self, shard: Shard, nodes: &[NodeId]) -> AssignedShard {
        let node_index = (self.position + self.rotation_offset) % nodes.len();
        self.position += 1;
        AssignedShard {
            shard,
            node: nodes[node_index],
        }
    }
}

/// Compute which node holds a specific shard without needing encoded data.
///
/// This is the canonical mapping function that both distributor and retriever
/// use to determine shard placement. Any party can recompute this from on-chain
/// parameters alone.
pub fn shard_node(
    blob_id: &BlobId,
    chunk_index: usize,
    shard_index: usize,
    shards_per_chunk: usize,
    nodes: &[NodeId],
) -> NodeId {
    let num_nodes = nodes.len();
    let rotation_offset = blob_id.modulo(num_nodes);
    let shard_global = chunk_index * shards_per_chunk + shard_index;
    let node_index = (shard_global + rotation_offset) % num_nodes;
    nodes[node_index]
}

/// Compute the full shard-to-node mapping for an entire blob.
///
/// Returns `(chunk_index, shard_index, NodeId)` for every shard.
pub fn full_shard_mapping(
    blob_id: &BlobId,
    num_chunks: usize,
    data_shards: usize,
    parity_shards: usize,
    nodes: &[NodeId],
) -> Vec<(usize, usize, NodeId)> {
    let shards_per_chunk = data_shards + parity_shards;
    let mut mapping = Vec::with_capacity(num_chunks * shards_per_chunk);
    for chunk_idx in 0..num_chunks {
        for shard_idx in 0..shards_per_chunk {
            let node = shard_node(blob_id, chunk_idx, shard_idx, shards_per_chunk, nodes);
            mapping.push((chunk_idx, shard_idx, node));
        }
    }
    mapping
}

/// Given a node, return all `(chunk_index, shard_index)` pairs assigned to it.
///
/// Useful for a storage node to know which shards it should expect/hold.
pub fn shards_for_node(
    blob_id: &BlobId,
    num_chunks: usize,
    data_shards: usize,
    parity_shards: usize,
    nodes: &[NodeId],
    target_node: &NodeId,
) -> Vec<(usize, usize)> {
    let shards_per_chunk = data_shards + parity_shards;
    let mut result = Vec::new();
    for chunk_idx in 0..num_chunks {
        for shard_idx in 0..shards_per_chunk {
            let node = shard_node(blob_id, chunk_idx, shard_idx, shards_per_chunk, nodes);
            if node == *target_node {
                result.push((chunk_idx, shard_idx));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{encode_and_assign, DEFAULT_MAX_CHUNK_SIZE};
    use crate::error::Result;
    use crate::reed_solomon::ReedSolomonEncoder;
    use std::collections::HashSet;

    fn make_nodes(n: usize) -> Vec<NodeId> {
        (0..n)
            .map(|i| {
                let mut id = [0u8; 32];
                id[0] = i as u8;
                NodeId(id)
            })
            .collect()
    }

    fn make_blob_id(seed: u8) -> BlobId {
        let mut id = [0u8; 32];
        id[0] = seed;
        BlobId(id)
    }

    #[test]
    fn deterministic_same_inputs_same_outputs() {
        let blob_id = make_blob_id(42);
        let nodes = make_nodes(10);

        let mapping1 = full_shard_mapping(&blob_id, 3, 4, 2, &nodes);
        let mapping2 = full_shard_mapping(&blob_id, 3, 4, 2, &nodes);

        assert_eq!(mapping1, mapping2);
    }

    #[test]
    fn assigner_matches_shard_node() {
        let blob_id = make_blob_id(7);
        let nodes = make_nodes(5);
        let k = 3;
        let m = 2;
        let data = vec![99u8; 1000];

        let (_meta, iter) = encode_and_assign::<ReedSolomonEncoder, _>(
            &data,
            k,
            m,
            &nodes,
            DeterministicAssigner::new(blob_id, nodes.len()),
        )
        .unwrap();
        let chunks: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        for chunk in &chunks {
            for assigned in &chunk.shards {
                let expected = shard_node(
                    &blob_id,
                    chunk.chunk_index,
                    assigned.shard.index,
                    k + m,
                    &nodes,
                );
                assert_eq!(
                    assigned.node, expected,
                    "Mismatch at chunk={} shard={}",
                    chunk.chunk_index, assigned.shard.index
                );
            }
        }
    }

    #[test]
    fn different_blob_ids_different_offsets() {
        let nodes = make_nodes(10);

        let mapping_a = full_shard_mapping(&make_blob_id(1), 1, 4, 2, &nodes);
        let mapping_b = full_shard_mapping(&make_blob_id(2), 1, 4, 2, &nodes);

        let nodes_a: Vec<_> = mapping_a.iter().map(|(_, _, n)| *n).collect();
        let nodes_b: Vec<_> = mapping_b.iter().map(|(_, _, n)| *n).collect();

        assert_ne!(nodes_a, nodes_b);
    }

    #[test]
    fn shards_for_node_covers_all_shards() {
        let blob_id = make_blob_id(99);
        let nodes = make_nodes(5);
        let k = 3;
        let m = 2;
        let num_chunks = 4;
        let total_shards = num_chunks * (k + m);

        let mut all_shards: HashSet<(usize, usize)> = HashSet::new();
        for node in &nodes {
            let shards = shards_for_node(&blob_id, num_chunks, k, m, &nodes, node);
            for s in shards {
                assert!(all_shards.insert(s), "Duplicate shard assignment: {:?}", s);
            }
        }

        assert_eq!(all_shards.len(), total_shards);
    }

    #[test]
    fn hand_calculated_example() {
        // blob_id = [7, 0, 0, ...], big-endian modulo 5:
        // byte 0: (0 * 256 + 7) % 5 = 2, rest are 0 → offset = 2
        let blob_id = make_blob_id(7);
        let nodes = make_nodes(5);
        let shards_per_chunk = 3;

        // chunk 0, shard 0: (0 + 2) % 5 = 2
        assert_eq!(
            shard_node(&blob_id, 0, 0, shards_per_chunk, &nodes),
            nodes[2]
        );
        // chunk 0, shard 1: (1 + 2) % 5 = 3
        assert_eq!(
            shard_node(&blob_id, 0, 1, shards_per_chunk, &nodes),
            nodes[3]
        );
        // chunk 0, shard 2: (2 + 2) % 5 = 4
        assert_eq!(
            shard_node(&blob_id, 0, 2, shards_per_chunk, &nodes),
            nodes[4]
        );
        // chunk 1, shard 0: (3 + 2) % 5 = 0
        assert_eq!(
            shard_node(&blob_id, 1, 0, shards_per_chunk, &nodes),
            nodes[0]
        );
        // chunk 1, shard 1: (4 + 2) % 5 = 1
        assert_eq!(
            shard_node(&blob_id, 1, 1, shards_per_chunk, &nodes),
            nodes[1]
        );
    }

    #[test]
    fn multi_chunk_assigner_consistency() {
        let blob_id = make_blob_id(13);
        let nodes = make_nodes(8);
        let k = 2;
        let m = 1;
        let data = vec![1u8; DEFAULT_MAX_CHUNK_SIZE + 100];

        let (_meta, iter) = encode_and_assign::<ReedSolomonEncoder, _>(
            &data,
            k,
            m,
            &nodes,
            DeterministicAssigner::new(blob_id, nodes.len()),
        )
        .unwrap();
        let chunks: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(chunks.len(), 2);
        for chunk in &chunks {
            for assigned in &chunk.shards {
                let expected = shard_node(
                    &blob_id,
                    chunk.chunk_index,
                    assigned.shard.index,
                    k + m,
                    &nodes,
                );
                assert_eq!(assigned.node, expected);
            }
        }
    }
}
