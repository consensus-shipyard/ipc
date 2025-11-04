// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::{deserialize_block, extract_send_result, runtime::Runtime, ActorError};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::{address::Address, sys::SendFlags, MethodNum};

use crate::{
    blobs::{
        AddBlobParams, Blob, DeleteBlobParams, GetBlobParams, OverwriteBlobParams, Subscription,
    },
    credit::{CreditApproval, GetCreditApprovalParams},
    method::Method,
    BLOBS_ACTOR_ADDR,
};

/// Returns a credit approval from one account to another if it exists.
pub fn get_credit_approval(
    rt: &impl Runtime,
    from: Address,
    to: Address,
) -> Result<Option<CreditApproval>, ActorError> {
    let params = GetCreditApprovalParams { from, to };

    deserialize_block(extract_send_result(rt.send(
        &BLOBS_ACTOR_ADDR,
        Method::GetCreditApproval as MethodNum,
        IpldBlock::serialize_cbor(&params)?,
        rt.message().value_received(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}

/// Returns `true` if `from` and `to` are the same address,
/// or if `from` has a credit delegation to `to` that has not yet expired.
pub fn has_credit_approval(
    rt: &impl Runtime,
    from: Address,
    to: Address,
) -> Result<bool, ActorError> {
    if from != to {
        let approval = get_credit_approval(rt, from, to)?;
        let curr_epoch = rt.curr_epoch();
        Ok(approval.is_some_and(|a| a.expiry.map_or(true, |e| e >= curr_epoch)))
    } else {
        Ok(true)
    }
}

/// Adds a blob.
pub fn add_blob(rt: &impl Runtime, params: AddBlobParams) -> Result<Subscription, ActorError> {
    let params = IpldBlock::serialize_cbor(&params)?;
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::AddBlob as MethodNum,
        params,
        rt.message().value_received(),
    ))?)
}

/// Returns information about a blob.
pub fn get_blob(rt: &impl Runtime, params: GetBlobParams) -> Result<Option<Blob>, ActorError> {
    deserialize_block(extract_send_result(rt.send(
        &BLOBS_ACTOR_ADDR,
        Method::GetBlob as MethodNum,
        IpldBlock::serialize_cbor(&params)?,
        rt.message().value_received(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}

/// Deletes a blob.
pub fn delete_blob(rt: &impl Runtime, params: DeleteBlobParams) -> Result<(), ActorError> {
    extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::DeleteBlob as MethodNum,
        IpldBlock::serialize_cbor(&params)?,
        rt.message().value_received(),
    ))?;
    Ok(())
}

/// Overwrite a blob, i.e., delete one and add another in a single call.
pub fn overwrite_blob(
    rt: &impl Runtime,
    params: OverwriteBlobParams,
) -> Result<Subscription, ActorError> {
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::OverwriteBlob as MethodNum,
        IpldBlock::serialize_cbor(&params)?,
        rt.message().value_received(),
    ))?)
}
