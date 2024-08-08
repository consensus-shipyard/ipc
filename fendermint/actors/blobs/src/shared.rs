// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

pub use crate::state::{Account, Blob, State};

pub const BLOBS_ACTOR_NAME: &str = "blobs";

/// Params for actor construction.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    // The total storage capacity of the subnet.
    pub capacity: u64,
    /// The byte-blocks per atto token rate.
    pub debit_rate: u64,
}

/// Params for buying credits.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuyCreditParams(pub Address);

/// Params for getting an account.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetAccountParams(pub Address);

/// Params for adding a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AddBlobParams {
    /// Blob content identifier.
    pub cid: Cid,
    /// Blob size.
    pub size: u64,
    /// Blob expiry epoch.
    pub expiry: ChainEpoch,
    /// Optional source actor robust address. Required is source is a machine.
    pub source: Option<Address>,
}

/// Params for resolving a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResolveBlobParams(pub Cid);

/// Params for deleting a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeleteBlobParams(pub Cid);

/// Params for getting a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetBlobParams(pub Cid);

/// The stats of the blob actor.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct GetStatsReturn {
    /// The current token balance earned by the subnet.
    pub balance: TokenAmount,
    /// The total free storage capacity of the subnet.
    pub capacity_free: BigInt,
    /// The total used storage capacity of the subnet.
    pub capacity_used: BigInt,
    /// The total number of credits sold in the subnet.
    pub credit_sold: BigInt,
    /// The total number of credits committed to active storage in the subnet.
    pub credit_committed: BigInt,
    /// The total number of credits debited in the subnet.
    pub credit_debited: BigInt,
    /// The byte-blocks per atto token rate set at genesis.
    pub credit_debit_rate: u64,
    /// Total number of debit accounts.
    pub num_accounts: u64,
    /// Total number of actively stored blobs.
    pub num_blobs: u64,
    /// Total number of currently resolving blobs.
    pub num_resolving: u64,
}

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
