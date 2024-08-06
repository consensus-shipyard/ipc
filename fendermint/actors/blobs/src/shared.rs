// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub use crate::state::{Blob, State};
use cid::Cid;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const BLOBS_ACTOR_NAME: &str = "blobs";

/// Params for actor construction.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    // The total storage capacity of the subnet.
    pub capacity: u64,
    /// The byte-blocks per atto token rate.
    pub debit_rate: u64,
}

/// Params for funding.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct FundParams {
    /// The actor address to fund.
    pub address: Address,
}

/// Params for putting a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AddParams {
    /// Blob content identifier.
    pub cid: Cid,
    /// Blob size.
    pub size: u64,
    /// Blob expiry epoch.
    pub expiry: ChainEpoch,
    /// Blob metadata.
    pub metadata: HashMap<String, String>,
}

/// Params for resolving a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResolveParams(pub Cid);

/// Params for deleting a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeleteParams(pub Cid);

/// Params for getting a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetParams(pub Cid);

/// The status of the blob actor.
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct Status {
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
}

/// Account storage and credit details.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Account {
    /// Total size of all blobs managed by the account.
    pub capacity_used: BigInt,
    /// Current free credit in byte-blocks that can be used for new commitments.
    pub credit_free: BigInt,
    /// Current committed credit in byte-blocks that will be used for debits.
    pub credit_committed: BigInt,
    /// The chain epoch of the last debit.
    pub last_debit_epoch: ChainEpoch,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    FundAccount = frc42_dispatch::method_hash!("FundAccount"),
    AddBlob = frc42_dispatch::method_hash!("AddBlob"),
    ResolveBlob = frc42_dispatch::method_hash!("ResolveBlob"),
    DeleteBlob = frc42_dispatch::method_hash!("DeleteBlob"),
    GetBlob = frc42_dispatch::method_hash!("GetBlob"),
}
