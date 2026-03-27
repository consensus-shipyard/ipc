// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::error::{ErasureError, Result};
use crate::traits::Decoder;
use crate::types::Shard;

/// Per-chunk recovery input for [`decode_chunks`].
#[derive(Debug, Clone)]
pub struct ChunkRecoveryInput {
    pub chunk_index: usize,
    /// Length of the original (unpadded) data in this chunk.
    pub original_data_len: usize,
    /// Available shards (indices 0..k = data, k..k+m = parity).
    pub shards: Vec<Shard>,
    /// Number of data shards (k) used during encoding.
    pub num_data_shards: usize,
    /// Number of parity shards (m) used during encoding.
    pub num_parity_shards: usize,
}

/// Reconstruct the original data from a set of chunk recovery inputs.
///
/// Each entry in `chunks` describes one chunk's available shards. Chunks are
/// sorted by `chunk_index`, decoded individually, and concatenated. The result
/// is truncated to `original_total_len` to strip padding.
pub fn decode_chunks<D: Decoder<Shard = Shard>>(
    chunks: &mut [ChunkRecoveryInput],
    original_total_len: usize,
) -> Result<Vec<u8>> {
    if chunks.is_empty() {
        return Err(ErasureError::EmptyData);
    }

    chunks.sort_by_key(|c| c.chunk_index);

    let mut output = Vec::with_capacity(original_total_len);

    for chunk in chunks.iter() {
        let mut decoded = D::decode(
            &chunk.shards,
            chunk.num_data_shards,
            chunk.num_parity_shards,
        )?;
        decoded.truncate(chunk.original_data_len);
        output.extend_from_slice(&decoded);
    }

    output.truncate(original_total_len);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{encode_and_assign, RotatingAssigner, DEFAULT_MAX_CHUNK_SIZE};
    use crate::error::Result;
    use crate::reed_solomon::ReedSolomonEncoder;
    use crate::types::NodeId;

    fn make_nodes(n: usize) -> Vec<NodeId> {
        (0..n)
            .map(|i| {
                let mut id = [0u8; 32];
                id[0] = i as u8;
                NodeId(id)
            })
            .collect()
    }

    fn to_recovery_inputs(
        encoded: &[crate::types::EncodedChunk],
        k: usize,
        m: usize,
        drop_indices: &[usize],
    ) -> Vec<ChunkRecoveryInput> {
        encoded
            .iter()
            .map(|ec| {
                let shards: Vec<_> = ec
                    .shards
                    .iter()
                    .filter(|s| !drop_indices.contains(&s.shard.index))
                    .map(|s| s.shard.clone())
                    .collect();
                ChunkRecoveryInput {
                    chunk_index: ec.chunk_index,
                    original_data_len: ec.original_data_len,
                    shards,
                    num_data_shards: k,
                    num_parity_shards: m,
                }
            })
            .collect()
    }

    #[test]
    fn round_trip_single_chunk() {
        let k = 4;
        let m = 2;
        let nodes = make_nodes(6);
        let data: Vec<u8> = (0..1000).map(|i| (i % 251) as u8).collect();

        let (meta, iter) = encode_and_assign::<ReedSolomonEncoder, _>(
            &data,
            k,
            m,
            &nodes,
            RotatingAssigner::new(),
        )
        .unwrap();
        let encoded: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        let mut inputs = to_recovery_inputs(&encoded, k, m, &[]);
        let recovered =
            decode_chunks::<ReedSolomonEncoder>(&mut inputs, meta.original_len).unwrap();
        assert_eq!(recovered, data);
    }

    #[test]
    fn round_trip_with_losses() {
        let k = 4;
        let m = 2;
        let nodes = make_nodes(6);
        let data: Vec<u8> = (0..500).map(|i| (i % 199) as u8).collect();

        let (meta, iter) = encode_and_assign::<ReedSolomonEncoder, _>(
            &data,
            k,
            m,
            &nodes,
            RotatingAssigner::new(),
        )
        .unwrap();
        let encoded: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        // Drop originals 0 and 1.
        let mut inputs = to_recovery_inputs(&encoded, k, m, &[0, 1]);
        let recovered =
            decode_chunks::<ReedSolomonEncoder>(&mut inputs, meta.original_len).unwrap();
        assert_eq!(recovered, data);
    }

    #[test]
    fn round_trip_multi_chunk() {
        let k = 2;
        let m = 1;
        let nodes = make_nodes(3);
        let data: Vec<u8> = (0..(DEFAULT_MAX_CHUNK_SIZE + 500))
            .map(|i| (i % 241) as u8)
            .collect();

        let (meta, iter) = encode_and_assign::<ReedSolomonEncoder, _>(
            &data,
            k,
            m,
            &nodes,
            RotatingAssigner::new(),
        )
        .unwrap();
        let encoded: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();
        assert_eq!(encoded.len(), 2);

        let mut inputs = to_recovery_inputs(&encoded, k, m, &[]);
        let recovered =
            decode_chunks::<ReedSolomonEncoder>(&mut inputs, meta.original_len).unwrap();
        assert_eq!(recovered, data);
    }

    #[test]
    fn padding_correctness_odd_size() {
        let k = 3;
        let m = 2;
        let nodes = make_nodes(5);
        let data: Vec<u8> = (0..77).map(|i| (i * 3 % 256) as u8).collect();

        let (meta, iter) = encode_and_assign::<ReedSolomonEncoder, _>(
            &data,
            k,
            m,
            &nodes,
            RotatingAssigner::new(),
        )
        .unwrap();
        let encoded: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        let mut inputs = to_recovery_inputs(&encoded, k, m, &[]);
        let recovered =
            decode_chunks::<ReedSolomonEncoder>(&mut inputs, meta.original_len).unwrap();
        assert_eq!(recovered, data);
    }
}
