// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("checkpoint not resolved")]
    CheckpointNotResolved,
    #[error("parent finality not available")]
    ParentFinalityNotAvailable,
    #[error("too many messages: {0}")]
    TooManyMessages(usize),
    #[error("failed to decode message in proposal as ChainMessage: {0}")]
    FailedToDecodeMessage(String),
    #[error("")]
    Empty,
}
