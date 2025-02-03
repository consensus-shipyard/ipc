// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::state::{
    BlobStatus, Credit, Hash, PublicKey, SubscriptionId, TokenCreditRate, TtlStatus,
};

/// Params for buying credits.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuyCreditParams(pub Address);

/// Params for updating credit.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct UpdateGasAllowanceParams {
    /// Account address that initiated the update.
    pub from: Address,
    /// Optional account address that is sponsoring the update.
    pub sponsor: Option<Address>,
    /// Token amount to add, which can be negative.
    pub add_amount: TokenAmount,
}

/// Params for approving credit.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ApproveCreditParams {
    /// Account address that is making the approval.
    pub from: Address,
    /// Account address that is receiving the approval.
    pub to: Address,
    /// Optional restriction on caller addresses, e.g., a bucket.
    /// The receiver will only be able to use the approval via an allowlisted caller.
    /// If not present, any caller is allowed.
    pub caller_allowlist: Option<HashSet<Address>>,
    /// Optional credit approval limit.
    /// If specified, the approval becomes invalid once the used credits reach the
    /// specified limit.
    pub credit_limit: Option<Credit>,
    /// Optional gas fee limit.
    /// If specified, the approval becomes invalid once the used gas fees reach the
    /// specified limit.
    pub gas_fee_limit: Option<TokenAmount>,
    /// Optional credit approval time-to-live epochs.
    /// If specified, the approval becomes invalid after this duration.
    pub ttl: Option<ChainEpoch>,
}

/// Params for revoking credit.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct RevokeCreditParams {
    /// Account address that is revoking the approval.
    pub from: Address,
    /// Account address whose approval is being revoked.
    pub to: Address,
    /// Optional caller address to remove from the caller allowlist.
    /// If not present, the entire approval is revoked.
    pub for_caller: Option<Address>,
}

/// Params for setting sponsor.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct SetSponsorParams {
    /// Account address that is setting a credit sponsor.
    pub from: Address,
    /// Credit sponsor.
    /// If not present, the sponsor is unset.
    pub sponsor: Option<Address>,
}

/// Params for setting account status.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct SetAccountStatusParams {
    /// Address to set the account status for.
    pub subscriber: Address,
    /// TTL status to set.
    pub status: TtlStatus,
}

/// Params for getting an account.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetAccountParams(pub Address);

/// Params for looking up a credit approval.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct GetCreditApprovalParams {
    /// Account address that made the approval.
    pub from: Address,
    /// Account address that received the approval.
    pub to: Address,
}

/// Params for looking up credit allowance.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetGasAllowanceParams(pub Address);

/// Params for adding a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AddBlobParams {
    /// Optional sponsor address.
    /// Origin or caller must still have a delegation from sponsor.
    pub sponsor: Option<Address>,
    /// Source Iroh node ID used for ingestion.
    pub source: PublicKey,
    /// Blob blake3 hash.
    pub hash: Hash,
    /// Blake3 hash of the metadata to use for blob recovery.
    pub metadata_hash: Hash,
    /// Identifier used to differentiate blob additions for the same subscriber.
    pub id: SubscriptionId,
    /// Blob size.
    pub size: u64,
    /// Blob time-to-live epochs.
    /// If not specified, the current default TTL from the config actor is used.
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
    /// Identifier used to differentiate blob additions for the same subscriber.
    pub id: SubscriptionId,
}

/// Params for getting added blobs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetAddedBlobsParams(pub u32);

/// Params for getting pending blobs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetPendingBlobsParams(pub u32);

/// Params for setting a blob to pending.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct SetBlobPendingParams {
    /// Source Iroh node ID used for ingestion.
    pub source: PublicKey,
    /// The address that requested the blob.
    pub subscriber: Address,
    /// Blob blake3 hash.
    pub hash: Hash,
    /// Identifier used to differentiate blob additions for the same subscriber.
    pub id: SubscriptionId,
}

/// Params for finalizing a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct FinalizeBlobParams {
    /// The address that requested the blob.
    /// This could be a wallet or machine.
    pub subscriber: Address,
    /// Blob blake3 hash.
    pub hash: Hash,
    /// Identifier used to differentiate blob additions for the same subscriber.
    pub id: SubscriptionId,
    /// The status to set as final.
    pub status: BlobStatus,
}

/// Params for deleting a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct DeleteBlobParams {
    /// Optional sponsor address.
    /// Origin or caller must still have a delegation from sponsor.
    /// Must be used if the caller is the delegate who added the blob.
    pub sponsor: Option<Address>,
    /// Blob blake3 hash.
    pub hash: Hash,
    /// Identifier used to differentiate blob additions for the same subscriber.
    pub id: SubscriptionId,
}

/// Params for overwriting a blob, i.e., deleting one and adding another.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct OverwriteBlobParams {
    /// Blake3 hash of the blob to be deleted.
    pub old_hash: Hash,
    /// Params for a new blob to add.
    pub add: AddBlobParams,
}

/// Params for trimming blob expiries.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct TrimBlobExpiriesParams {
    /// Address to trim blob expiries for.
    pub subscriber: Address,
    /// Starting hash to trim expiries from.
    pub starting_hash: Option<Hash>,
    /// Limit of blobs to trim expiries for.
    /// This specifies the maximum number of blobs that will be examined for trimming.
    pub limit: Option<usize>,
}

/// The stats of the blob actor.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct GetStatsReturn {
    /// The current token balance earned by the subnet.
    pub balance: TokenAmount,
    /// The total free storage capacity of the subnet.
    pub capacity_free: u64,
    /// The total used storage capacity of the subnet.
    pub capacity_used: u64,
    /// The total number of credits sold in the subnet.
    pub credit_sold: Credit,
    /// The total number of credits committed to active storage in the subnet.
    pub credit_committed: Credit,
    /// The total number of credits debited in the subnet.
    pub credit_debited: Credit,
    /// The token to credit rate.
    pub token_credit_rate: TokenCreditRate,
    /// Total number of debit accounts.
    pub num_accounts: u64,
    /// Total number of actively stored blobs.
    pub num_blobs: u64,
    /// Total number of blobs that are not yet added to the validator's resolve pool.
    pub num_added: u64,
    // Total bytes of all blobs that are not yet added to the validator's resolve pool.
    pub bytes_added: u64,
    /// Total number of currently resolving blobs.
    pub num_resolving: u64,
    /// Total bytes of all currently resolving blobs.
    pub bytes_resolving: u64,
}
