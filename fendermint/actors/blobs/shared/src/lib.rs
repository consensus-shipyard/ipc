// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{deserialize_block, extract_send_result, ActorError};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::{ActorID, MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;

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
    GetAccount = frc42_dispatch::method_hash!("GetAccount"),
    AddBlob = frc42_dispatch::method_hash!("AddBlob"),
    GetBlob = frc42_dispatch::method_hash!("GetBlob"),
    GetBlobStatus = frc42_dispatch::method_hash!("GetBlobStatus"),
    GetPendingBlobs = frc42_dispatch::method_hash!("GetPendingBlobs"),
    FinalizeBlob = frc42_dispatch::method_hash!("FinalizeBlob"),
    DeleteBlob = frc42_dispatch::method_hash!("DeleteBlob"),
}

pub fn add_blob(
    rt: &impl Runtime,
    from: Address,
    source: state::PublicKey,
    hash: state::Hash,
    size: u64,
    ttl: ChainEpoch,
) -> Result<(), ActorError> {
    let add_params = IpldBlock::serialize_cbor(&params::AddBlobParams {
        from: Some(from),
        source,
        hash,
        size,
        ttl,
    })?;
    extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::AddBlob as MethodNum,
        add_params,
        rt.message().value_received(),
    ))?;
    Ok(())
}

pub fn get_blob(rt: &impl Runtime, hash: state::Hash) -> Result<state::Blob, ActorError> {
    deserialize_block::<state::Blob>(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::GetBlob as MethodNum,
        IpldBlock::serialize_cbor(&params::GetBlobParams(hash))?,
        rt.message().value_received(),
    ))?)
}

pub fn delete_blob(rt: &impl Runtime, from: Address, hash: state::Hash) -> Result<(), ActorError> {
    extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::DeleteBlob as MethodNum,
        IpldBlock::serialize_cbor(&params::DeleteBlobParams {
            from: Some(from),
            hash,
        })?,
        rt.message().value_received(),
    ))?;
    Ok(())
}
