// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, clock::ChainEpoch};
use serde::{Deserialize, Serialize};

use super::{BlobStatus, SubscriptionId};
use crate::bytes::B256;

/// Params for adding a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AddBlobParams {
    /// Address of the entity adding the blob.
    pub from: Address,
    /// Optional sponsor address.
    /// Origin or caller must still have a delegation from sponsor.
    pub sponsor: Option<Address>,
    /// Source Iroh node ID used for ingestion.
    pub source: B256,
    /// Blob blake3 hash.
    pub hash: B256,
    /// Blake3 hash of the metadata to use for blob recovery.
    pub metadata_hash: B256,
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
pub struct GetBlobParams(pub B256);

/// Params for getting blob status.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct GetBlobStatusParams {
    /// The origin address that requested the blob.
    /// This could be a wallet or machine.
    pub subscriber: Address,
    /// Blob blake3 hash.
    pub hash: B256,
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
    pub source: B256,
    /// The address that requested the blob.
    pub subscriber: Address,
    /// Blob blake3 hash.
    pub hash: B256,
    /// Blob size.
    pub size: u64,
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
    pub hash: B256,
    /// Identifier used to differentiate blob additions for the same subscriber.
    pub id: SubscriptionId,
    /// The status to set as final.
    pub status: BlobStatus,
}

/// Params for deleting a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct DeleteBlobParams {
    /// Account address that initiated the deletion.
    pub from: Address,
    /// Optional sponsor address.
    /// Origin or caller must still have a delegation from sponsor.
    /// Must be used if the caller is the delegate who added the blob.
    pub sponsor: Option<Address>,
    /// Blob blake3 hash.
    pub hash: B256,
    /// Identifier used to differentiate blob additions for the same subscriber.
    pub id: SubscriptionId,
}

/// Params for overwriting a blob, i.e., deleting one and adding another.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct OverwriteBlobParams {
    /// Blake3 hash of the blob to be deleted.
    pub old_hash: B256,
    /// Params for a new blob to add.
    pub add: AddBlobParams,
}

/// Params for trimming blob expiries.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct TrimBlobExpiriesParams {
    /// Address to trim blob expiries for.
    pub subscriber: Address,
    /// Starting hash to trim expiries from.
    pub starting_hash: Option<B256>,
    /// Limit of blobs to trim expiries for.
    /// This specifies the maximum number of blobs that will be examined for trimming.
    pub limit: Option<u32>,
}
