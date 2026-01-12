// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::{
    blobs::{BlobStatus, SubscriptionId},
    bytes::B256,
};
use fvm_shared::{clock::ChainEpoch, econ::TokenAmount};

/// Params for adding a blob.
#[derive(Clone, Debug)]
pub struct AddBlobStateParams {
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
    /// Chain epoch.
    pub epoch: ChainEpoch,
    /// Token amount sent with the transaction.
    pub token_amount: TokenAmount,
}

impl AddBlobStateParams {
    pub fn from_actor_params(
        params: fendermint_actor_blobs_shared::blobs::AddBlobParams,
        epoch: ChainEpoch,
        token_amount: TokenAmount,
    ) -> Self {
        Self {
            source: params.source,
            hash: params.hash,
            metadata_hash: params.metadata_hash,
            id: params.id,
            size: params.size,
            ttl: params.ttl,
            epoch,
            token_amount,
        }
    }
}

/// Params for deleting a blob.
#[derive(Clone, Debug)]
pub struct DeleteBlobStateParams {
    /// Blob blake3 hash.
    pub hash: B256,
    /// Identifier used to differentiate blob additions for the same subscriber.
    pub id: SubscriptionId,
    /// Chain epoch.
    pub epoch: ChainEpoch,
    /// Whether to skip returning credit for an over-debit.
    /// This is needed to handle cases where multiple subscriptions are being expired in the same
    /// epoch for the same subscriber.
    pub skip_credit_return: bool,
}

impl DeleteBlobStateParams {
    pub fn from_actor_params(
        params: fendermint_actor_blobs_shared::blobs::DeleteBlobParams,
        epoch: ChainEpoch,
    ) -> Self {
        Self {
            hash: params.hash,
            id: params.id,
            epoch,
            skip_credit_return: false,
        }
    }
}

/// Params for setting a blob to pending state.
#[derive(Clone, Debug)]
pub struct SetPendingBlobStateParams {
    /// Source Iroh node ID used for ingestion.
    pub source: B256,
    /// Blob blake3 hash.
    pub hash: B256,
    /// Blob size.
    pub size: u64,
    /// Identifier used to differentiate blob additions for the same subscriber.
    pub id: SubscriptionId,
}

impl SetPendingBlobStateParams {
    pub fn from_actor_params(
        params: fendermint_actor_blobs_shared::blobs::SetBlobPendingParams,
    ) -> Self {
        Self {
            source: params.source,
            hash: params.hash,
            size: params.size,
            id: params.id,
        }
    }
}

/// Params for finalizing a blob.
#[derive(Clone, Debug)]
pub struct FinalizeBlobStateParams {
    /// Source Iroh node ID used for ingestion.
    pub source: B256,
    /// Blob blake3 hash.
    pub hash: B256,
    /// Blob size.
    pub size: u64,
    /// Identifier used to differentiate blob additions for the same subscriber.
    pub id: SubscriptionId,
    /// Finalized status.
    pub status: BlobStatus,
    /// Chain epoch.
    pub epoch: ChainEpoch,
}

impl FinalizeBlobStateParams {
    pub fn from_actor_params(
        params: fendermint_actor_blobs_shared::blobs::FinalizeBlobParams,
        epoch: ChainEpoch,
    ) -> Self {
        Self {
            source: params.source,
            hash: params.hash,
            size: params.size,
            id: params.id,
            status: params.status,
            epoch,
        }
    }
}
