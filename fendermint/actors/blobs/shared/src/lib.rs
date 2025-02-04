// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashSet;

use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{deserialize_block, extract_send_result, ActorError};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::sys::SendFlags;
use fvm_shared::{ActorID, MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;

use crate::state::{Account, Credit, CreditApproval, Subscription};

pub mod params;
pub mod state;

pub const BLOBS_ACTOR_ID: ActorID = 66;
pub const BLOBS_ACTOR_ADDR: Address = Address::new_id(BLOBS_ACTOR_ID);

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,

    // User methods
    BuyCredit = frc42_dispatch::method_hash!("BuyCredit"),
    ApproveCredit = frc42_dispatch::method_hash!("ApproveCredit"),
    RevokeCredit = frc42_dispatch::method_hash!("RevokeCredit"),
    SetAccountSponsor = frc42_dispatch::method_hash!("SetAccountSponsor"),
    GetAccount = frc42_dispatch::method_hash!("GetAccount"),
    GetCreditApproval = frc42_dispatch::method_hash!("GetCreditApproval"),
    AddBlob = frc42_dispatch::method_hash!("AddBlob"),
    GetBlob = frc42_dispatch::method_hash!("GetBlob"),
    DeleteBlob = frc42_dispatch::method_hash!("DeleteBlob"),
    OverwriteBlob = frc42_dispatch::method_hash!("OverwriteBlob"),

    // System methods
    GetGasAllowance = frc42_dispatch::method_hash!("GetGasAllowance"),
    UpdateGasAllowance = frc42_dispatch::method_hash!("UpdateGasAllowance"),
    GetBlobStatus = frc42_dispatch::method_hash!("GetBlobStatus"),
    GetAddedBlobs = frc42_dispatch::method_hash!("GetAddedBlobs"),
    GetPendingBlobs = frc42_dispatch::method_hash!("GetPendingBlobs"),
    SetBlobPending = frc42_dispatch::method_hash!("SetBlobPending"),
    FinalizeBlob = frc42_dispatch::method_hash!("FinalizeBlob"),
    DebitAccounts = frc42_dispatch::method_hash!("DebitAccounts"),

    // Admin methods
    SetAccountStatus = frc42_dispatch::method_hash!("SetAccountStatus"),
    TrimBlobExpiries = frc42_dispatch::method_hash!("TrimBlobExpiries"),

    // Metrics methods
    GetStats = frc42_dispatch::method_hash!("GetStats"),
}

pub fn buy_credit(rt: &impl Runtime, to: Address) -> Result<Account, ActorError> {
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::BuyCredit as MethodNum,
        IpldBlock::serialize_cbor(&params::BuyCreditParams(to))?,
        rt.message().value_received(),
    ))?)
}

pub fn approve_credit(
    rt: &impl Runtime,
    from: Address,
    to: Address,
    caller_allowlist: Option<HashSet<Address>>,
    credit_limit: Option<Credit>,
    gas_fee_limit: Option<TokenAmount>,
    ttl: Option<ChainEpoch>,
) -> Result<CreditApproval, ActorError> {
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::ApproveCredit as MethodNum,
        IpldBlock::serialize_cbor(&params::ApproveCreditParams {
            from,
            to,
            caller_allowlist,
            credit_limit,
            gas_fee_limit,
            ttl,
        })?,
        rt.message().value_received(),
    ))?)
}

pub fn get_credit_approval(
    rt: &impl Runtime,
    from: Address,
    to: Address,
) -> Result<Option<CreditApproval>, ActorError> {
    let params = params::GetCreditApprovalParams { from, to };

    deserialize_block(extract_send_result(rt.send(
        &BLOBS_ACTOR_ADDR,
        Method::GetCreditApproval as MethodNum,
        IpldBlock::serialize_cbor(&params)?,
        rt.message().value_received(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}

pub fn revoke_credit(
    rt: &impl Runtime,
    from: Address,
    to: Address,
    for_caller: Option<Address>,
) -> Result<(), ActorError> {
    extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::RevokeCredit as MethodNum,
        IpldBlock::serialize_cbor(&params::RevokeCreditParams {
            from,
            to,
            for_caller,
        })?,
        rt.message().value_received(),
    ))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn add_blob(
    rt: &impl Runtime,
    sub_id: state::SubscriptionId,
    hash: state::Hash,
    sponsor: Option<Address>,
    source: state::PublicKey,
    metadata_hash: state::Hash,
    size: u64,
    ttl: Option<ChainEpoch>,
    from: Address,
) -> Result<Subscription, ActorError> {
    let params = IpldBlock::serialize_cbor(&params::AddBlobParams {
        sponsor,
        source,
        hash,
        metadata_hash,
        id: sub_id,
        size,
        ttl,
        from,
    })?;
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::AddBlob as MethodNum,
        params,
        rt.message().value_received(),
    ))?)
}

pub fn get_blob(rt: &impl Runtime, hash: state::Hash) -> Result<Option<state::Blob>, ActorError> {
    deserialize_block(extract_send_result(rt.send(
        &BLOBS_ACTOR_ADDR,
        Method::GetBlob as MethodNum,
        IpldBlock::serialize_cbor(&params::GetBlobParams(hash))?,
        rt.message().value_received(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}

pub fn delete_blob(
    rt: &impl Runtime,
    sub_id: state::SubscriptionId,
    hash: state::Hash,
    sponsor: Option<Address>,
) -> Result<(), ActorError> {
    extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::DeleteBlob as MethodNum,
        IpldBlock::serialize_cbor(&params::DeleteBlobParams {
            sponsor,
            hash,
            id: sub_id,
        })?,
        rt.message().value_received(),
    ))?;
    Ok(())
}

/// Overwrite a blob, i.e., delete one and add another in a single call.
#[allow(clippy::too_many_arguments)]
pub fn overwrite_blob(
    rt: &impl Runtime,
    old_hash: state::Hash,
    sub_id: state::SubscriptionId,
    hash: state::Hash,
    sponsor: Option<Address>,
    source: state::PublicKey,
    metadata_hash: state::Hash,
    size: u64,
    ttl: Option<ChainEpoch>,
) -> Result<Subscription, ActorError> {
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::OverwriteBlob as MethodNum,
        IpldBlock::serialize_cbor(&params::OverwriteBlobParams {
            old_hash,
            add: params::AddBlobParams {
                sponsor,
                id: sub_id,
                source,
                hash,
                metadata_hash,
                size,
                ttl,
                from: rt.message().caller(),
            },
        })?,
        rt.message().value_received(),
    ))?)
}
