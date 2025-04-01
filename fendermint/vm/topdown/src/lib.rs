// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod cache;
mod error;
pub mod finality;
pub mod sync;

pub mod proxy;

pub mod observe;

use ethers::utils::hex;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::Duration;

pub use crate::cache::{SequentialAppendError, SequentialKeyCache, ValueIter};
pub use crate::error::Error;

pub type BlockHeight = u64;
pub type Bytes = Vec<u8>;
pub type BlockHash = Bytes;

/// The null round error message
pub(crate) const NULL_ROUND_ERR_MSG: &str = "requested epoch was a null round";
pub(crate) const DEFAULT_MAX_CACHE_BLOCK: BlockHeight = 500;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// The number of blocks to delay before reporting a height as final on the parent chain.
    /// To propose a certain number of epochs delayed from the latest height, we see to be
    /// conservative and avoid other from rejecting the proposal because they don't see the
    /// height as final yet.
    pub chain_head_delay: BlockHeight,
    /// Parent syncing cron period, in seconds
    pub polling_interval: Duration,
    /// Parent voting cron period, in seconds
    pub vote_interval: Duration,
    /// Max number of blocks that should be stored in cache
    pub max_cache_blocks: Option<BlockHeight>,
}

impl Config {
    pub fn new(
        chain_head_delay: BlockHeight,
        polling_interval: Duration,
        vote_interval: Duration,
    ) -> Self {
        Self {
            chain_head_delay,
            polling_interval,
            vote_interval,
            max_cache_blocks: None,
        }
    }

    pub fn with_max_cache_blocks(&mut self, blocks: BlockHeight) {
        self.max_cache_blocks = Some(blocks);
    }

    pub fn max_cache_blocks(&self) -> BlockHeight {
        self.max_cache_blocks.unwrap_or(DEFAULT_MAX_CACHE_BLOCK)
    }
}

/// The finality view for IPC parent at certain height.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParentState {
    /// The latest chain height
    pub height: BlockHeight,
    /// The block hash. For FVM, it is a Cid. For Evm, it is bytes32 as one can now potentially
    /// deploy a subnet on EVM.
    pub block_hash: BlockHash,
}

impl ParentState {
    pub fn new(height: ChainEpoch, hash: BlockHash) -> Self {
        Self {
            height: height as BlockHeight,
            block_hash: hash,
        }
    }
}

impl Display for ParentState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IPCParentFinality(height: {}, block_hash: {})",
            self.height,
            hex::encode(&self.block_hash)
        )
    }
}

/// checks if the error is a filecoin null round error
pub(crate) fn is_null_round_str(s: &str) -> bool {
    s.contains(NULL_ROUND_ERR_MSG)
}
