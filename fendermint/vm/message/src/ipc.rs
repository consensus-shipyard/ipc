// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::blobs::SubscriptionId;

use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

/// Messages involved in InterPlanetary Consensus.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum IpcMessage {
    /// A top-down checkpoint parent finality proposal. This proposal should contain the latest parent
    /// state that to be checked and voted by validators.
    TopDownExec(ParentFinality),

    /// Proposed by validators at the credit debit interval set at genesis.
    DebitCreditAccounts,

    /// List of blobs that needs to be enqueued for resolution.
    BlobPending(PendingBlob),

    /// Proposed by validators when a blob has been finalized and is ready to be executed.
    BlobFinalized(FinalizedBlob),

    /// Proposed by validators when a read request has been enqueued for resolution.
    ReadRequestPending(PendingReadRequest),

    /// Proposed by validators when a read request has been closed.
    ReadRequestClosed(ClosedReadRequest),
}

/// A proposal of the parent view that validators will be voting on.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ParentFinality {
    /// Block height of this proposal.
    pub height: ChainEpoch,
    /// The block hash of the parent, expressed as bytes
    pub block_hash: Vec<u8>,
}

/// A blob resolution target that the validators will be voting on.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FinalizedBlob {
    /// The address that requested the blob.
    pub subscriber: Address,
    /// The blake3 hash of the blob.
    pub hash: Hash,
    /// The size of the blob.
    pub size: u64,
    /// Identifier used to differentiate blob additions for the same subscriber.
    pub id: SubscriptionId,
    /// The node ID of the source node serving validators the blob.
    pub source: NodeId,
    /// Whether the blob was resolved or failed.
    pub succeeded: bool,
}

/// A blob that has been added but not yet queued for resolution.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PendingBlob {
    /// The address that requested the blob.
    pub subscriber: Address,
    /// The blake3 hash of the blob.
    pub hash: Hash,
    /// The size of the blob.
    pub size: u64,
    /// Identifier used to differentiate blob additions for the same subscriber.
    pub id: SubscriptionId,
    /// The node ID of the source node serving validators the blob.
    pub source: NodeId,
}

/// A read request that the validators will be voting on.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClosedReadRequest {
    /// The request ID.
    pub id: Hash,
    /// The hash of the blob to read from.
    pub blob_hash: Hash,
    /// The offset in the blob to read from.
    pub offset: u32,
    /// The length of the read request.
    pub len: u32,
    /// The address and method to callback when the read request is closed.
    pub callback: (Address, MethodNum),
    /// The data read from the blob.
    pub response: Vec<u8>,
}

/// A read request that is pending resolution.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PendingReadRequest {
    /// The request ID.
    pub id: Hash,
    /// The hash of the blob to read from.
    pub blob_hash: Hash,
    /// The offset in the blob to read from.
    pub offset: u32,
    /// The length of the read request.
    pub len: u32,
    /// The address and method to callback when the read request is closed.
    pub callback: (Address, MethodNum),
}

#[cfg(feature = "arb")]
mod arb {

    use crate::ipc::ParentFinality;
    use quickcheck::{Arbitrary, Gen};

    use super::IpcMessage;

    impl Arbitrary for IpcMessage {
        fn arbitrary(g: &mut Gen) -> Self {
            IpcMessage::TopDownExec(Arbitrary::arbitrary(g))
        }
    }

    impl Arbitrary for ParentFinality {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                height: u32::arbitrary(g).into(),
                block_hash: Vec::arbitrary(g),
            }
        }
    }
}
