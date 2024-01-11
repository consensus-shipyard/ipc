// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use serde::{Deserialize, Serialize};

type BlockHeight = u64;
type BlockHash = Vec<u8>;

/// All the things that can be voted in a subnet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vote {
    /// The validator considers a certain block final on the parent chain.
    ParentFinality(BlockHeight, BlockHash),
}
