// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{deserialize_block, extract_send_result, ActorError};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigUint;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::sys::SendFlags;
use fvm_shared::{ActorID, MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;

use crate::state::{Account, CreditApproval, Subscription};

pub mod params;
pub mod state;

pub const BLOBS_ACTOR_ID: ActorID = 49;
pub const BLOBS_ACTOR_ADDR: Address = Address::new_id(BLOBS_ACTOR_ID);

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    GetStats = frc42_dispatch::method_hash!("GetStats"),
    BuyCredit = frc42_dispatch::method_hash!("BuyCredit"),
    ApproveCredit = frc42_dispatch::method_hash!("ApproveCredit"),
    RevokeCredit = frc42_dispatch::method_hash!("RevokeCredit"),
    GetAccount = frc42_dispatch::method_hash!("GetAccount"),
    DebitAccounts = frc42_dispatch::method_hash!("DebitAccounts"),
    AddBlob = frc42_dispatch::method_hash!("AddBlob"),
    GetBlob = frc42_dispatch::method_hash!("GetBlob"),
    GetBlobStatus = frc42_dispatch::method_hash!("GetBlobStatus"),
    GetPendingBlobs = frc42_dispatch::method_hash!("GetPendingBlobs"),
    FinalizeBlob = frc42_dispatch::method_hash!("FinalizeBlob"),
    DeleteBlob = frc42_dispatch::method_hash!("DeleteBlob"),
}

pub fn buy_credit(rt: &impl Runtime, recipient: Address) -> Result<Account, ActorError> {
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::BuyCredit as MethodNum,
        IpldBlock::serialize_cbor(&params::BuyCreditParams(recipient))?,
        rt.message().value_received(),
    ))?)
}

pub fn approve_credit(
    rt: &impl Runtime,
    receiver: Address,
    required_caller: Option<Address>,
    limit: Option<BigUint>,
    ttl: Option<ChainEpoch>,
) -> Result<CreditApproval, ActorError> {
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::ApproveCredit as MethodNum,
        IpldBlock::serialize_cbor(&params::ApproveCreditParams {
            receiver,
            required_caller,
            limit,
            ttl,
        })?,
        rt.message().value_received(),
    ))?)
}

pub fn revoke_credit(
    rt: &impl Runtime,
    receiver: Address,
    required_caller: Option<Address>,
) -> Result<(), ActorError> {
    extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::RevokeCredit as MethodNum,
        IpldBlock::serialize_cbor(&params::RevokeCreditParams {
            receiver,
            required_caller,
        })?,
        rt.message().value_received(),
    ))?;
    Ok(())
}

pub fn add_blob(
    rt: &impl Runtime,
    sponsor: Option<Address>,
    source: state::PublicKey,
    hash: state::Hash,
    size: u64,
    ttl: Option<ChainEpoch>,
) -> Result<Subscription, ActorError> {
    let params = IpldBlock::serialize_cbor(&params::AddBlobParams {
        sponsor,
        source,
        hash,
        size,
        ttl,
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
    sponsor: Option<Address>,
    hash: state::Hash,
) -> Result<(), ActorError> {
    extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::DeleteBlob as MethodNum,
        IpldBlock::serialize_cbor(&params::DeleteBlobParams { sponsor, hash })?,
        rt.message().value_received(),
    ))?;
    Ok(())
}
