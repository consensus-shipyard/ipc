// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod errors;
pub mod fvm;
pub mod genesis;
pub(crate) mod selector;
pub mod types;

#[cfg(feature = "arb")]
mod arb;

use crate::errors::*;
use crate::fvm::state::{FvmExecState, FvmQueryState};
use crate::fvm::store::ReadOnlyBlockstore;
use crate::types::*;
use async_trait::async_trait;
use std::sync::Arc;

use fvm_ipld_blockstore::Blockstore;

#[async_trait]
pub trait MessagesInterpreter<DB>
where
    DB: Blockstore + Clone,
{
    /// Check a message without consuming state.
    async fn check_message(
        &self,
        state: FvmExecState<ReadOnlyBlockstore<Arc<DB>>>,
        msg: Vec<u8>,
        is_recheck: bool,
    ) -> Result<CheckResponse, CheckMessageError>;

    /// Prepare messages for a block (read-only state) and return a dedicated response struct.
    async fn prepare_messages_for_block(
        &self,
        state: FvmExecState<ReadOnlyBlockstore<Arc<DB>>>,
        msgs: Vec<Vec<u8>>,
        max_transaction_bytes: u64,
    ) -> Result<PrepareMessagesResponse, PrepareMessagesError>;

    /// Attest block messages (read-only state).
    async fn attest_block_messages(
        &self,
        state: FvmExecState<ReadOnlyBlockstore<Arc<DB>>>,
        msgs: Vec<Vec<u8>>,
    ) -> Result<AttestMessagesResponse, AttestMessagesError>;

    /// Begin a block (state-consuming).
    async fn begin_block(
        &self,
        state: FvmExecState<DB>,
    ) -> Result<(FvmExecState<DB>, BeginBlockResponse), BeginBlockError>;

    /// End a block (state-consuming).
    async fn end_block(
        &self,
        state: FvmExecState<DB>,
    ) -> Result<(FvmExecState<DB>, EndBlockResponse), EndBlockError>;

    /// Apply a message (state-consuming).
    async fn apply_message(
        &self,
        state: FvmExecState<DB>,
        msg: Vec<u8>,
    ) -> Result<(FvmExecState<DB>, ApplyMessageResponse), ApplyMessageError>;

    /// Process a query (read-only state).
    async fn query(
        &self,
        state: FvmQueryState<DB>,
        query: Query,
    ) -> Result<QueryResponse, QueryError>;
}
