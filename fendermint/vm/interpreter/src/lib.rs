// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod errors;
pub mod fvm;
pub mod genesis;
pub(crate) mod selectors;
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
    async fn check_message(
        &self,
        state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
        msg: Vec<u8>,
        is_recheck: bool,
    ) -> Result<CheckResponse, CheckMessageError>;
    
    /// Set the proof cache for F3 proof-based parent finality (if supported)
    /// Default implementation does nothing for interpreters that don't support F3
    async fn set_proof_cache(
        &self, 
        _cache: std::sync::Arc<fendermint_vm_topdown_proof_service::ProofCache>,
    ) {
        // Default: no-op for interpreters without F3 support
    }

    async fn prepare_messages_for_block(
        &self,
        state: FvmExecState<ReadOnlyBlockstore<Arc<DB>>>,
        msgs: Vec<Vec<u8>>,
        max_transaction_bytes: u64,
    ) -> Result<PrepareMessagesResponse, PrepareMessagesError>;

    async fn attest_block_messages(
        &self,
        state: FvmExecState<ReadOnlyBlockstore<Arc<DB>>>,
        msgs: Vec<Vec<u8>>,
    ) -> Result<AttestMessagesResponse, AttestMessagesError>;

    async fn begin_block(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> Result<BeginBlockResponse, BeginBlockError>;

    async fn end_block(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> Result<EndBlockResponse, EndBlockError>;

    async fn apply_message(
        &self,
        state: &mut FvmExecState<DB>,
        msg: Vec<u8>,
    ) -> Result<ApplyMessageResponse, ApplyMessageError>;

    async fn query(
        &self,
        state: FvmQueryState<DB>,
        query: Query,
    ) -> Result<QueryResponse, QueryError>;
}
