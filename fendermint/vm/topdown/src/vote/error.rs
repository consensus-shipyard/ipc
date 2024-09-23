// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::BlockHeight;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("the last finalized block has not been set")]
    Uninitialized,

    #[error("failed to extend chain; height going backwards, current height {0}, got {1}")]
    UnexpectedBlock(BlockHeight, BlockHeight),

    #[error("validator unknown or has no power")]
    UnpoweredValidator,

    #[error("equivocation by validator")]
    Equivocation,

    #[error("validator vote is invalidated")]
    VoteCannotBeValidated,

    #[error("validator cannot sign vote")]
    CannotSignVote,
}
