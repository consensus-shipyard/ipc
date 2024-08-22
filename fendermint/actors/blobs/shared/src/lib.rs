// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::address::Address;
use fvm_shared::{ActorID, METHOD_CONSTRUCTOR};
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
    BuyCredit = frc42_dispatch::method_hash!("FundAccount"),
    GetAccount = frc42_dispatch::method_hash!("GetAccount"),
    AddBlob = frc42_dispatch::method_hash!("AddBlob"),
    GetResolvingBlobs = frc42_dispatch::method_hash!("GetResolvingBlobs"),
    IsBlobResolving = frc42_dispatch::method_hash!("IsBlobResolving"),
    ResolveBlob = frc42_dispatch::method_hash!("ResolveBlob"),
    DeleteBlob = frc42_dispatch::method_hash!("DeleteBlob"),
    GetBlob = frc42_dispatch::method_hash!("GetBlob"),
}
