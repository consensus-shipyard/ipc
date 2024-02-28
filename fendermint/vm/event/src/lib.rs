// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

#[derive(strum::Display)]
pub enum EventType {
    NewParentView,
    ParentFinalityCommitted,
    NewBottomUpCheckpoint,
    /// A new block is produced in fendermint
    NewBlock,
    /// A proposal is processed
    ProposalProcessed,
}

#[macro_export]
macro_rules! emit {
    ($event:expr, $($arg:tt)*) => {
        tracing::info!(event = tracing::field::display($event), $($arg)+)
    };
    ($event:expr) => {
        tracing::info!(event = tracing::field::display($event))
    };
}
