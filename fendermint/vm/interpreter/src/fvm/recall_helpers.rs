// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Helper functions for Recall blob and read request operations
use crate::fvm::constants::BLOCK_GAS_LIMIT;
use anyhow::{anyhow, Result};
use fendermint_actor_blob_reader::{
    CloseReadRequestParams, GetOpenReadRequestsParams, GetPendingReadRequestsParams,
    GetReadRequestStatusParams,
    Method::{
        CloseReadRequest, GetOpenReadRequests, GetPendingReadRequests, GetReadRequestStatus,
        SetReadRequestPending,
    },
    ReadRequestStatus, SetReadRequestPendingParams, BLOB_READER_ACTOR_ADDR,
};
use fendermint_actor_blobs_shared::blobs::{
    BlobStatus, GetAddedBlobsParams, GetBlobStatusParams, GetPendingBlobsParams, SubscriptionId,
};
use fendermint_actor_blobs_shared::bytes::B256;
use fendermint_actor_blobs_shared::method::Method::{
    GetAddedBlobs, GetBlobStatus, GetPendingBlobs, GetStats,
};
use fendermint_actor_blobs_shared::{GetStatsReturn, BLOBS_ACTOR_ADDR};
use fendermint_vm_actor_interface::system;
use fendermint_vm_message::ipc::ClosedReadRequest;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::{address::Address, message::Message, MethodNum};
use iroh_blobs::Hash;
use std::collections::HashSet;
use std::sync::Arc;

use super::state::{FvmExecState, FvmQueryState};
use super::store::ReadOnlyBlockstore;
use crate::fvm::state::FvmApplyRet;

type BlobItem = (Hash, u64, HashSet<(Address, SubscriptionId, iroh::NodeId)>);
type ReadRequestItem = (Hash, Hash, u32, u32, Address, MethodNum);

/// Get added blobs from on chain state.
pub fn get_added_blobs<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    size: u32,
) -> Result<Vec<BlobItem>>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let params = GetAddedBlobsParams(size);
    let params = RawBytes::serialize(params)?;
    let msg = create_implicit_message(
        BLOBS_ACTOR_ADDR,
        GetAddedBlobs as u64,
        params,
        BLOCK_GAS_LIMIT,
    );
    let (apply_ret, _) = state.execute_implicit(msg)?;

    let data= apply_ret.msg_receipt.return_data.to_vec();
    fvm_ipld_encoding::from_slice::<Vec<BlobItem>>(&data)
        .map_err(|e| anyhow!("error parsing added blobs: {e}"))
}

/// Get pending blobs from on chain state.
pub fn get_pending_blobs<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    size: u32,
) -> Result<Vec<BlobItem>>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let params = GetPendingBlobsParams(size);
    let params = RawBytes::serialize(params)?;
    let msg = create_implicit_message(
        BLOBS_ACTOR_ADDR,
        GetPendingBlobs as u64,
        params,
        BLOCK_GAS_LIMIT,
    );
    let (apply_ret, _) = state.execute_implicit(msg)?;

    let data= apply_ret.msg_receipt.return_data.to_vec();
    fvm_ipld_encoding::from_slice::<Vec<BlobItem>>(&data)
        .map_err(|e| anyhow!("error parsing pending blobs: {e}"))
}

/// Helper function to check blob status by reading its on-chain state.
pub fn get_blob_status<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    subscriber: Address,
    hash: Hash,
    id: SubscriptionId,
) -> Result<Option<BlobStatus>>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let hash = B256(*hash.as_bytes());
    let params = GetBlobStatusParams {
        subscriber,
        hash,
        id,
    };
    let params = RawBytes::serialize(params)?;
    let msg = create_implicit_message(
        BLOBS_ACTOR_ADDR,
        GetBlobStatus as u64,
        params,
        BLOCK_GAS_LIMIT,
    );
    let (apply_ret, _) = state.execute_implicit(msg)?;

    let data= apply_ret.msg_receipt.return_data.to_vec();
    fvm_ipld_encoding::from_slice::<Option<BlobStatus>>(&data)
        .map_err(|e| anyhow!("error parsing blob status: {e}"))
}

/// Check if a blob is in the added state, by reading its on-chain state.
pub fn is_blob_added<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    subscriber: Address,
    hash: Hash,
    id: SubscriptionId,
) -> Result<(bool, Option<BlobStatus>)>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let status = get_blob_status(state, subscriber, hash, id)?;
    let added = if let Some(status) = status.clone() {
        matches!(status, BlobStatus::Added)
    } else {
        false
    };
    Ok((added, status))
}

/// Check if a blob is finalized (if it is resolved or failed), by reading its on-chain state.
pub fn is_blob_finalized<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    subscriber: Address,
    hash: Hash,
    id: SubscriptionId,
) -> Result<(bool, Option<BlobStatus>)>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let status = get_blob_status(state, subscriber, hash, id)?;
    let finalized = if let Some(status) = status.clone() {
        matches!(status, BlobStatus::Resolved | BlobStatus::Failed)
    } else {
        false
    };
    Ok((finalized, status))
}

/// Returns credit and blob stats from on-chain state.
pub fn get_blobs_stats<DB>(state: &mut FvmExecState<DB>) -> Result<GetStatsReturn>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let msg = create_implicit_message(
        BLOBS_ACTOR_ADDR,
        GetStats as u64,
        Default::default(),
        BLOCK_GAS_LIMIT,
    );
    let (apply_ret, _) = state.execute_implicit(msg)?;

    let data= apply_ret.msg_receipt.return_data.to_vec();
    fvm_ipld_encoding::from_slice::<GetStatsReturn>(&data)
        .map_err(|e| anyhow!("error parsing stats: {e}"))
}

/// Get open read requests from on chain state.
pub fn get_open_read_requests<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    size: u32,
) -> Result<Vec<ReadRequestItem>>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let params = RawBytes::serialize(GetOpenReadRequestsParams(size))?;
    let msg = create_implicit_message(
        BLOB_READER_ACTOR_ADDR,
        GetOpenReadRequests as u64,
        params,
        BLOCK_GAS_LIMIT,
    );
    let (apply_ret, _) = state.execute_implicit(msg)?;

    let data= apply_ret.msg_receipt.return_data.to_vec();
    fvm_ipld_encoding::from_slice::<Vec<ReadRequestItem>>(&data)
        .map_err(|e| anyhow!("error parsing read requests: {e}"))
}

/// Get pending read requests from on chain state.
pub fn get_pending_read_requests<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    size: u32,
) -> Result<Vec<ReadRequestItem>>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let params = RawBytes::serialize(GetPendingReadRequestsParams(size))?;
    let msg = create_implicit_message(
        BLOB_READER_ACTOR_ADDR,
        GetPendingReadRequests as u64,
        params,
        BLOCK_GAS_LIMIT,
    );
    let (apply_ret, _) = state.execute_implicit(msg)?;

    let data= apply_ret.msg_receipt.return_data.to_vec();
    fvm_ipld_encoding::from_slice::<Vec<ReadRequestItem>>(&data)
        .map_err(|e| anyhow!("error parsing read requests: {e}"))
}

/// Get the status of a read request from on chain state.
pub fn get_read_request_status<DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    id: Hash,
) -> Result<Option<ReadRequestStatus>>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let request_id = B256(*id.as_bytes());
    let params = RawBytes::serialize(GetReadRequestStatusParams(request_id))?;
    let msg = create_implicit_message(
        BLOB_READER_ACTOR_ADDR,
        GetReadRequestStatus as u64,
        params,
        BLOCK_GAS_LIMIT,
    );

    let (apply_ret, _) = state.execute_implicit(msg)?;
    let data= apply_ret.msg_receipt.return_data.to_vec();
    fvm_ipld_encoding::from_slice::<Option<ReadRequestStatus>>(&data)
        .map_err(|e| anyhow!("error parsing read request status: {e}"))
}

/// Set the on-chain state of a read request to pending.
pub fn set_read_request_pending<DB>(
    state: &mut FvmExecState<DB>,
    id: Hash,
) -> Result<FvmApplyRet>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let params = RawBytes::serialize(SetReadRequestPendingParams(B256(*id.as_bytes())))?;
    let gas_limit = BLOCK_GAS_LIMIT;
    let msg = create_implicit_message(
        BLOB_READER_ACTOR_ADDR,
        SetReadRequestPending as u64,
        params,
        gas_limit,
    );

    let (apply_ret, emitters) = state.execute_implicit(msg)?;
    Ok(FvmApplyRet {
        apply_ret,
        from: system::SYSTEM_ACTOR_ADDR,
        to: BLOB_READER_ACTOR_ADDR,
        method_num: SetReadRequestPending as u64,
        gas_limit,
        emitters,
    })
}

/// Execute the callback for a read request.
pub fn read_request_callback<DB>(
    state: &mut FvmExecState<DB>,
    read_request: &ClosedReadRequest,
) -> Result<()>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let ClosedReadRequest {
        id,
        blob_hash: _,
        offset: _,
        len: _,
        callback: (to, method_num),
        response,
    } = read_request.clone();

    let params = RawBytes::serialize((id, response))?;
    let msg = Message {
        version: Default::default(),
        from: BLOB_READER_ACTOR_ADDR,
        to,
        sequence: 0,
        value: Default::default(),
        method_num,
        params,
        gas_limit: BLOCK_GAS_LIMIT,
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };
    let result = state.execute_implicit(msg);
    match result {
        Ok((apply_ret, _)) => {
            tracing::debug!(
                "callback delivered for id: {:?}, exit code: {:?}",
                id,
                apply_ret.msg_receipt.exit_code
            );
        }
        Err(e) => {
            tracing::error!(
                "failed to execute read request callback for id: {:?}, error: {}",
                id,
                e
            );
        }
    }

    Ok(())
}

/// Remove a read request from on chain state.
pub fn close_read_request<DB>(state: &mut FvmExecState<DB>, id: Hash) -> Result<FvmApplyRet>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    let params = RawBytes::serialize(CloseReadRequestParams(B256(*id.as_bytes())))?;
    let gas_limit = BLOCK_GAS_LIMIT;
    let msg = create_implicit_message(
        BLOB_READER_ACTOR_ADDR,
        CloseReadRequest as u64,
        params,
        gas_limit,
    );

    let (apply_ret, emitters) = state.execute_implicit(msg)?;
    Ok(FvmApplyRet {
        apply_ret,
        from: system::SYSTEM_ACTOR_ADDR,
        to: BLOB_READER_ACTOR_ADDR,
        method_num: CloseReadRequest as u64,
        gas_limit,
        emitters,
    })
}

/// Creates a standard implicit message with default values
pub fn create_implicit_message(
    to: Address,
    method_num: u64,
    params: RawBytes,
    gas_limit: u64,
) -> Message {
    Message {
        version: Default::default(),
        from: system::SYSTEM_ACTOR_ADDR,
        to,
        sequence: 0,
        value: Default::default(),
        method_num,
        params,
        gas_limit,
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    }
}

/// Calls a function inside a state transaction.
pub fn with_state_transaction<F, R, DB>(
    state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
    f: F,
) -> Result<R>
where
    F: FnOnce(&mut FvmExecState<ReadOnlyBlockstore<DB>>) -> Result<R>,
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    state.state_tree_mut().begin_transaction();
    let result = f(state);
    state
        .state_tree_mut()
        .end_transaction(true)
        .expect("interpreter failed to end state transaction");
    result
}
