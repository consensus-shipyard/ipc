// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::fmt::Debug;

use crate::error::Result;
use crate::types::{AssignedShard, NodeId};

/// Assigns a shard to a storage node.
pub trait NodeAssigner {
    type Shard: Debug;

    fn assign(&mut self, shard: Self::Shard, nodes: &[NodeId]) -> AssignedShard;
}

/// Splits raw data into `num_data_chunks` padded shards, encodes, and returns all shards.
pub trait Encoder {
    type Shard: Debug;

    fn encode(
        data: &[u8],
        num_data_chunks: usize,
        num_parity_chunks: usize,
    ) -> Result<impl Iterator<Item = Self::Shard>>;
}

/// Recovers missing original shards and returns reconstructed raw data.
pub trait Decoder {
    type Shard: Debug;

    fn decode(
        shards: &[Self::Shard],
        num_data_chunks: usize,
        num_parity_chunks: usize,
    ) -> Result<Vec<u8>>;
}
