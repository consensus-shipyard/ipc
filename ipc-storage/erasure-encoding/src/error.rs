// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ErasureError>;

#[derive(Debug, Error)]
pub enum ErasureError {
    #[error("input data is empty")]
    EmptyData,

    #[error("invalid data shard count: {0} (must be > 0)")]
    InvalidDataShards(usize),

    #[error("invalid parity shard count: {0} (must be > 0)")]
    InvalidParityShards(usize),

    #[error("not enough nodes: need {needed}, have {available}")]
    NotEnoughNodes { needed: usize, available: usize },

    #[error("shard count mismatch: expected {expected}, got {actual}")]
    ShardSizeMismatch { expected: usize, actual: usize },

    #[error("not enough shards for decoding: need {needed}, have {available}")]
    NotEnoughShards { needed: usize, available: usize },

    #[error("shard index {index} out of range (max {max})")]
    ShardIndexOutOfRange { index: usize, max: usize },

    #[error("duplicate shard index: {0}")]
    DuplicateShardIndex(usize),

    #[error("reed-solomon error: {0}")]
    ReedSolomon(#[from] reed_solomon_simd::Error),
}
