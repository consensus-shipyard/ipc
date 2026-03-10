// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::error::{ErasureError, Result};
use crate::traits::{Encoder, NodeAssigner};
use crate::types::{AssignedShard, EncodedChunk, EncodingMetadata, NodeId, Shard};

/// Default maximum chunk size (16 MiB) per DESIGN.md.
pub const DEFAULT_MAX_CHUNK_SIZE: usize = 16 * 1024 * 1024;

/// Node assigner that rotates through the node list sequentially.
///
/// Each shard advances the position by one, so consecutive chunks naturally
/// map to different node subsets.
pub struct RotatingAssigner {
    position: usize,
}

impl RotatingAssigner {
    pub fn new() -> Self {
        Self { position: 0 }
    }
}

impl Default for RotatingAssigner {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeAssigner for RotatingAssigner {
    type Shard = Shard;

    fn assign(&mut self, shard: Shard, nodes: &[NodeId]) -> AssignedShard {
        let node = nodes[self.position % nodes.len()];
        self.position += 1;
        AssignedShard { shard, node }
    }
}

/// Encode `data` using erasure coding and assign shards to `nodes`.
///
/// Returns `(metadata, iterator)`. The iterator yields one [`EncodedChunk`] per
/// chunk so that only one chunk's shards are in memory at a time.
///
/// Each chunk is passed to `E::encode` which handles splitting, padding, and
/// encoding internally, yielding all k+m shards. Shards are assigned to nodes
/// via the provided [`NodeAssigner`].
pub fn encode_and_assign<'a, E: Encoder<Shard = Shard>, N: NodeAssigner<Shard = Shard> + 'a>(
    data: &'a [u8],
    data_chunks: usize,
    parity_chunks: usize,
    nodes: &'a [NodeId],
    mut node_assigner: N,
) -> Result<(
    EncodingMetadata,
    impl Iterator<Item = Result<EncodedChunk>> + 'a,
)> {
    if nodes.is_empty() {
        return Err(ErasureError::NotEnoughNodes {
            needed: 1,
            available: 0,
        });
    }
    if data.is_empty() {
        return Err(ErasureError::EmptyData);
    }
    if data_chunks == 0 {
        return Err(ErasureError::InvalidDataShards(data_chunks));
    }
    if parity_chunks == 0 {
        return Err(ErasureError::InvalidParityShards(parity_chunks));
    }

    let num_chunks = data.len().div_ceil(DEFAULT_MAX_CHUNK_SIZE);

    let metadata = EncodingMetadata {
        original_len: data.len(),
        num_chunks,
        data_shards: data_chunks,
        parity_shards: parity_chunks,
    };

    // the total number of shards for each chunk after erasure encoding

    let total_shards = data_chunks + parity_chunks;
    let iter = (0..num_chunks).map(move |chunk_index| {
        let chunk_start = chunk_index * DEFAULT_MAX_CHUNK_SIZE;
        let chunk_end = (chunk_start + DEFAULT_MAX_CHUNK_SIZE).min(data.len());
        let chunk_data = &data[chunk_start..chunk_end];
        let original_data_len = chunk_data.len();

        let all_shards = E::encode(chunk_data, data_chunks, parity_chunks)?;

        let mut assigned: Vec<AssignedShard> = Vec::with_capacity(total_shards);
        for shard in all_shards {
            assigned.push(node_assigner.assign(shard, nodes));
        }

        Ok(EncodedChunk {
            chunk_index,
            original_data_len,
            shards: assigned,
        })
    });

    Ok((metadata, iter))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reed_solomon::ReedSolomonEncoder;

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
    fn encode_and_assign_basic() {
        let k = 4;
        let m = 2;
        let nodes = make_nodes(6);
        let data = vec![42u8; 1000];

        let (meta, iter) = encode_and_assign::<ReedSolomonEncoder, _>(
            &data,
            k,
            m,
            &nodes,
            RotatingAssigner::new(),
        )
        .unwrap();
        let chunks: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(meta.original_len, 1000);
        assert_eq!(meta.num_chunks, 1);
        assert_eq!(meta.data_shards, k);
        assert_eq!(meta.parity_shards, m);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].shards.len(), k + m);
    }

    #[test]
    fn encode_and_assign_multi_chunk() {
        let k = 2;
        let m = 1;
        let nodes = make_nodes(3);
        let data = vec![7u8; DEFAULT_MAX_CHUNK_SIZE + 100];

        let (meta, iter) = encode_and_assign::<ReedSolomonEncoder, _>(
            &data,
            k,
            m,
            &nodes,
            RotatingAssigner::new(),
        )
        .unwrap();
        let chunks: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(meta.num_chunks, 2);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].original_data_len, DEFAULT_MAX_CHUNK_SIZE);
        assert_eq!(chunks[1].original_data_len, 100);
    }

    #[test]
    fn error_empty() {
        let nodes = make_nodes(3);
        let result =
            encode_and_assign::<ReedSolomonEncoder, _>(&[], 2, 1, &nodes, RotatingAssigner::new());
        assert!(matches!(result, Err(ErasureError::EmptyData)));
    }

    #[test]
    fn fewer_nodes_than_shards_ok() {
        let k = 4;
        let m = 2;
        let nodes = make_nodes(3);
        let data = vec![42u8; 1000];

        let (_meta, iter) = encode_and_assign::<ReedSolomonEncoder, _>(
            &data,
            k,
            m,
            &nodes,
            RotatingAssigner::new(),
        )
        .unwrap();
        let chunks: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();
        assert_eq!(chunks[0].shards.len(), k + m);
    }

    #[test]
    fn rotating_assigner_distributes_across_chunks() {
        let k = 2;
        let m = 1;
        let nodes = make_nodes(6);
        let data = vec![1u8; DEFAULT_MAX_CHUNK_SIZE * 2];

        let (_meta, iter) = encode_and_assign::<ReedSolomonEncoder, _>(
            &data,
            k,
            m,
            &nodes,
            RotatingAssigner::new(),
        )
        .unwrap();
        let chunks: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        // Chunk 0 gets nodes [0,1,2], chunk 1 gets nodes [3,4,5].
        let c0_nodes: Vec<_> = chunks[0].shards.iter().map(|s| s.node).collect();
        let c1_nodes: Vec<_> = chunks[1].shards.iter().map(|s| s.node).collect();
        assert_eq!(c0_nodes, &nodes[0..3]);
        assert_eq!(c1_nodes, &nodes[3..6]);
    }
}
