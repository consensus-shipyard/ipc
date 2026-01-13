// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

/// Messages involved in InterPlanetary Consensus.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum IpcMessage {
    /// A top-down checkpoint parent finality proposal (legacy voting-based)
    TopDownExec(ParentFinality),
    /// Generalized top-down finality with extensible certificate types
    GeneralisedTopDown(GeneralisedTopDown),
}

/// A proposal of the parent view that validators will be voting on.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ParentFinality {
    /// Block height of this proposal.
    pub height: ChainEpoch,
    /// The block hash of the parent, expressed as bytes
    pub block_hash: Vec<u8>,
}

/// Generalized top-down finality structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralisedTopDown {
    /// The chain epoch this finality is for (height)
    pub height: ChainEpoch,
    /// The certificate that certifies finality (type-specific, proof is fetched from local cache)
    pub certificate: Certificate,
}

/// Certificate types (extensible for future certificate types)
/// Each variant contains the certificate data. Proofs are fetched from local cache when needed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Certificate {
    /// Filecoin F3 certificate (proof bundle is fetched from local cache using instance ID)
    FilecoinF3(fendermint_vm_topdown_proof_service::types::SerializableF3Certificate),
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
