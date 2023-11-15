// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{BlockHeight, SequentialAppendError};
use thiserror::Error;

/// The errors for top down checkpointing
#[derive(Error, Debug, Eq, PartialEq, Clone)]
pub enum Error {
    #[error("Incoming items are not order sequentially")]
    NotSequential,
    #[error("The parent view update with block height is not sequential: {0:?}")]
    NonSequentialParentViewInsert(SequentialAppendError),
    #[error("Parent chain reorg detected")]
    ParentChainReorgDetected,
    #[error("Cannot query parent at height {1}: {0}")]
    CannotQueryParent(String, BlockHeight),
    /// This error happens when querying top down messages, the block ahead are all null rounds.
    /// See `parent_views_at_height` for detailed explanation
    #[error("Look ahead limit reached from {0}: {1}")]
    LookAheadLimitReached(BlockHeight, BlockHeight),
}
