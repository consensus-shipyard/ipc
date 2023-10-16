// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::SequentialAppendError;
use thiserror::Error;

/// The errors for top down checkpointing
#[derive(Error, Debug, Eq, PartialEq, Clone)]
pub enum Error {
    #[error("Incoming top down messages are not order by nonce sequentially")]
    NonceNotSequential,
    #[error("The parent view update with block height is not sequential")]
    NonSequentialParentViewInsert(SequentialAppendError),
    #[error("Parent chain reorg detected")]
    ParentChainReorgDetected,
    #[error("Cannot query parent")]
    CannotQueryParent(String),
}
