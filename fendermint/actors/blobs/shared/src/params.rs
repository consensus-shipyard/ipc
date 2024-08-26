// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use serde::{Deserialize, Serialize};

use crate::state::{Hash, PublicKey};

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
    /// Robust address of caller. Required if the caller is a machine.
    pub from: Option<Address>,
    /// Source Iroh node ID used for ingestion.
    pub source: PublicKey,
    /// Blob blake3 hash.
    pub hash: Hash,
    /// Blob size.
    pub size: u64,
    /// Blob expiry epoch.
    pub expiry: ChainEpoch,
}

/// Params for getting a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetBlobParams(pub Hash);

/// Params for resolving a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResolveBlobParams(pub Hash);

/// Params for failing a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FailBlobParams(pub Hash);

/// Params for deleting a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeleteBlobParams(pub Hash);

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
