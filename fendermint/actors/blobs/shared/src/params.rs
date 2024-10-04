// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::{BigInt, BigUint};
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use serde::{Deserialize, Serialize};

use crate::state::{BlobStatus, Hash, PublicKey};

/// Params for buying credits.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuyCreditParams(pub Address);

/// Params for approving credit.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ApproveCreditParams {
    /// Account address that is receiving the approval.
    pub receiver: Address,
    /// Optional restriction on caller address, e.g., an object store.
    /// The receiver will only be able to use the approval via a caller contract.
    pub required_caller: Option<Address>,
    /// Optional credit approval limit.
    /// If specified, the approval becomes invalid once the committed credits reach the
    /// specified limit.
    pub limit: Option<BigUint>,
    /// Optional credit approval time-to-live epochs.
    /// If specified, the approval becomes invalid after this duration.
    pub ttl: Option<ChainEpoch>,
}

/// Params for revoking credit.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct RevokeCreditParams {
    /// Account address that is receiving the approval.
    pub receiver: Address,
    /// Optional restriction on caller address, e.g., an object store.
    /// This allows the origin of a transaction to use an approval limited to the caller.
    pub required_caller: Option<Address>,
}

/// Params for getting an account.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetAccountParams(pub Address);

/// Params for adding a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AddBlobParams {
    /// Optional sponsor address.
    /// Caller must have a delegation from sponsor.
    pub sponsor: Option<Address>,
    /// Source Iroh node ID used for ingestion.
    pub source: PublicKey,
    /// Blob blake3 hash.
    pub hash: Hash,
    /// Blob size.
    pub size: u64,
    /// Blob time-to-live epochs.
    /// If not specified, the auto-debitor maintains about one hour of credits as an
    /// ongoing commitment.
    pub ttl: Option<ChainEpoch>,
}

/// Params for getting a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetBlobParams(pub Hash);

/// Params for getting blob status.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct GetBlobStatusParams {
    /// The origin address that requested the blob.
    /// This could be a wallet or machine.
    pub subscriber: Address,
    /// Blob blake3 hash.
    pub hash: Hash,
}

/// Params for getting pending blobs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetPendingBlobsParams(pub u32);

/// Params for finalizing a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct FinalizeBlobParams {
    /// The address that requested the blob.
    /// This could be a wallet or machine.
    pub subscriber: Address,
    /// Blob blake3 hash.
    pub hash: Hash,
    /// The status to set as final.
    pub status: BlobStatus,
}

/// Params for deleting a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct DeleteBlobParams {
    /// Optional sponsor address.
    /// Caller must still have a delegation from sponsor.
    /// Must be used if the caller is the delegate who added the blob.
    pub sponsor: Option<Address>,
    /// Blob blake3 hash.
    pub hash: Hash,
}

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
