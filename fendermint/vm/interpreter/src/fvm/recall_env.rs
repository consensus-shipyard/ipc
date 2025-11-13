// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Recall environment types for blob and read request resolution.

use fendermint_actor_blobs_shared::blobs::SubscriptionId;
use fendermint_vm_iroh_resolver::pool::{
    ResolveKey as IrohResolveKey, ResolvePool as IrohResolvePool,
    ResolveSource as IrohResolveSource, TaskType as IrohTaskType,
};
use fvm_shared::{address::Address, MethodNum};
use iroh::NodeId;
use iroh_blobs::Hash;

pub type BlobPool = IrohResolvePool<BlobPoolItem>;
pub type ReadRequestPool = IrohResolvePool<ReadRequestPoolItem>;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct BlobPoolItem {
    pub subscriber: Address,
    pub hash: Hash,
    pub size: u64,
    pub id: SubscriptionId,
    pub source: NodeId,
}

impl From<&BlobPoolItem> for IrohResolveKey {
    fn from(value: &BlobPoolItem) -> Self {
        Self { hash: value.hash }
    }
}

impl From<&BlobPoolItem> for IrohTaskType {
    fn from(value: &BlobPoolItem) -> Self {
        Self::ResolveBlob {
            source: IrohResolveSource { id: value.source },
            size: value.size,
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ReadRequestPoolItem {
    /// The unique id of the read request.
    pub id: Hash,
    /// The hash of the blob that the read request is for.
    pub blob_hash: Hash,
    /// The offset of the read request.
    pub offset: u32,
    /// The length of the read request.
    pub len: u32,
    /// The address and method to callback when the read request is closed.
    pub callback: (Address, MethodNum),
}

impl From<&ReadRequestPoolItem> for IrohResolveKey {
    fn from(value: &ReadRequestPoolItem) -> Self {
        Self { hash: value.id }
    }
}

impl From<&ReadRequestPoolItem> for IrohTaskType {
    fn from(value: &ReadRequestPoolItem) -> Self {
        Self::CloseReadRequest {
            blob_hash: value.blob_hash,
            offset: value.offset,
            len: value.len,
        }
    }
}
