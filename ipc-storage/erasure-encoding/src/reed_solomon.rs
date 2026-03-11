// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::collections::HashSet;

use crate::error::{ErasureError, Result};
use crate::traits::{Decoder, Encoder};
use crate::types::Shard;

/// Reed-Solomon encoder/decoder backed by `reed-solomon-simd`.
pub struct ReedSolomonEncoder;

/// Split raw data into `k` equal shards with zero-padding.
/// Shard size is rounded up to even (reed-solomon-simd requirement).
fn split_into_shards(data: &[u8], k: usize) -> Vec<Vec<u8>> {
    let mut shard_size = data.len().div_ceil(k);
    if shard_size % 2 != 0 {
        shard_size += 1;
    }
    if shard_size == 0 {
        shard_size = 2;
    }

    let mut shards = Vec::with_capacity(k);
    for i in 0..k {
        let start = i * shard_size;
        let mut shard = vec![0u8; shard_size];
        if start < data.len() {
            let end = (start + shard_size).min(data.len());
            shard[..end - start].copy_from_slice(&data[start..end]);
        }
        shards.push(shard);
    }
    shards
}

impl Encoder for ReedSolomonEncoder {
    type Shard = Shard;

    fn encode(
        data: &[u8],
        num_data_chunks: usize,
        num_parity_chunks: usize,
    ) -> Result<impl Iterator<Item = Shard>> {
        if num_data_chunks == 0 {
            return Err(ErasureError::InvalidDataShards(num_data_chunks));
        }
        if num_parity_chunks == 0 {
            return Err(ErasureError::InvalidParityShards(num_parity_chunks));
        }

        let original_shards = split_into_shards(data, num_data_chunks);
        let parity =
            reed_solomon_simd::encode(num_data_chunks, num_parity_chunks, &original_shards)?;

        // Yield k original shards (index 0..k) then m parity shards (index k..k+m).
        let iter = original_shards
            .into_iter()
            .enumerate()
            .map(|(i, data)| Shard { index: i, data })
            .chain(parity.into_iter().enumerate().map(move |(i, data)| Shard {
                index: num_data_chunks + i,
                data,
            }));

        Ok(iter)
    }
}

impl Decoder for ReedSolomonEncoder {
    type Shard = Shard;

    fn decode(
        shards: &[Shard],
        num_data_chunks: usize,
        num_parity_chunks: usize,
    ) -> Result<Vec<u8>> {
        let mut seen = HashSet::with_capacity(shards.len());
        for s in shards {
            if !seen.insert(s.index) {
                return Err(ErasureError::DuplicateShardIndex(s.index));
            }
        }

        let total_available = seen.len();
        if total_available < num_data_chunks {
            return Err(ErasureError::NotEnoughShards {
                needed: num_data_chunks,
                available: total_available,
            });
        }

        // Split by index: 0..k = original, k..k+m = recovery.
        let original: Vec<_> = shards
            .iter()
            .filter(|s| s.index < num_data_chunks)
            .map(|s| (s.index, s.data.as_slice()))
            .collect();
        let recovery: Vec<_> = shards
            .iter()
            .filter(|s| s.index >= num_data_chunks)
            .map(|s| (s.index - num_data_chunks, s.data.as_slice()))
            .collect();

        let restored = reed_solomon_simd::decode(
            num_data_chunks,
            num_parity_chunks,
            original.iter().copied(),
            recovery.iter().copied(),
        )?;

        // Merge available + restored originals in index order, concatenate.
        let mut all_originals: Vec<Option<&[u8]>> = vec![None; num_data_chunks];
        for &(idx, data) in &original {
            all_originals[idx] = Some(data);
        }
        for (idx, data) in &restored {
            all_originals[*idx] = Some(data.as_slice());
        }

        let mut result = Vec::new();
        for (i, opt) in all_originals.iter().enumerate() {
            let shard = opt.ok_or(ErasureError::NotEnoughShards {
                needed: num_data_chunks,
                available: i,
            })?;
            result.extend_from_slice(shard);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_yields_all_shards() {
        let data = vec![42u8; 256];
        let shards: Vec<_> = ReedSolomonEncoder::encode(&data, 4, 2).unwrap().collect();
        assert_eq!(shards.len(), 6);
        // Indices 0..4 original, 4..6 parity.
        for (i, s) in shards.iter().enumerate() {
            assert_eq!(s.index, i);
        }
        // All shards same size.
        let size = shards[0].data.len();
        assert!(shards.iter().all(|s| s.data.len() == size));
    }

    #[test]
    fn round_trip_full_shards() {
        let data = vec![42u8; 256];
        let shards: Vec<_> = ReedSolomonEncoder::encode(&data, 4, 2).unwrap().collect();

        let recovered = ReedSolomonEncoder::decode(&shards, 4, 2).unwrap();
        assert_eq!(&recovered[..data.len()], &data);
    }

    #[test]
    fn decode_with_losses() {
        let data = vec![42u8; 256];
        let all_shards: Vec<_> = ReedSolomonEncoder::encode(&data, 4, 2).unwrap().collect();

        // Drop originals 0 and 1, keep 2, 3 + both parity.
        let available: Vec<_> = all_shards.into_iter().filter(|s| s.index >= 2).collect();
        assert_eq!(available.len(), 4); // shards 2, 3, 4, 5

        let recovered = ReedSolomonEncoder::decode(&available, 4, 2).unwrap();
        assert_eq!(&recovered[..data.len()], &data);
    }

    #[test]
    fn decode_max_loss() {
        let data = vec![42u8; 512];
        let all_shards: Vec<_> = ReedSolomonEncoder::encode(&data, 4, 3).unwrap().collect();

        // Lose originals 0,1,2, keep index 3 + all 3 parity.
        let available: Vec<_> = all_shards.into_iter().filter(|s| s.index >= 3).collect();
        assert_eq!(available.len(), 4);

        let recovered = ReedSolomonEncoder::decode(&available, 4, 3).unwrap();
        assert_eq!(&recovered[..data.len()], &data);
    }

    #[test]
    fn error_not_enough_shards() {
        let shard = Shard {
            index: 0,
            data: vec![0u8; 64],
        };
        let result = ReedSolomonEncoder::decode(&[shard], 4, 2);
        assert!(matches!(result, Err(ErasureError::NotEnoughShards { .. })));
    }

    #[test]
    fn error_invalid_params() {
        assert!(matches!(
            ReedSolomonEncoder::encode(&[1, 2], 0, 2).map(|i| i.count()),
            Err(ErasureError::InvalidDataShards(0))
        ));
        assert!(matches!(
            ReedSolomonEncoder::encode(&[1, 2], 2, 0).map(|i| i.count()),
            Err(ErasureError::InvalidParityShards(0))
        ));
    }
}
