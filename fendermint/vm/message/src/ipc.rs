// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

/// Messages involved in InterPlanetary Consensus.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum IpcMessage {
    /// A top-down checkpoint parent finality proposal. This proposal should contain the latest parent
    /// state that to be checked and voted by validators.
    TopDownExec(ParentFinality),
    /// Proof-based topdown finality with cryptographic F3 certificates and proof bundles.
    /// This is the v2 approach that replaces voting with deterministic verification.
    /// The bundle can span multiple blocks as F3 certificates finalize chains of epochs.
    TopDownWithProof(TopDownProofBundle),
}

/// A proposal of the parent view that validators will be voting on.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ParentFinality {
    /// Block height of this proposal.
    pub height: ChainEpoch,
    /// The block hash of the parent, expressed as bytes
    pub block_hash: Vec<u8>,
}

/// Proof-based topdown finality bundle with cryptographic verification.
///
/// This contains:
/// - A validated F3 certificate with instance ID and finalized epochs (chain of blocks)
/// - A proof bundle with storage proofs (completeness) and event proofs (topdown messages/validator changes)
///
/// Validators verify this deterministically without requiring gossip-based voting.
/// The certificate can finalize multiple blocks, so height/block_hash are not in the bundle itself.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TopDownProofBundle {
    /// Validated F3 certificate (serializable for consensus)
    pub certificate: fendermint_vm_topdown_proof_service::types::SerializableF3Certificate,
    /// Cryptographic proof bundle (storage + event proofs + witness blocks)
    pub proof_bundle: proofs::proofs::common::bundle::UnifiedProofBundle,
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
