// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

/// Messages involved in InterPlanetary Consensus.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum IpcMessage {
    /// A top-down checkpoint parent finality proposal. This proposal should contain the latest parent
    /// state that to be checked and voted by validators.
    TopDownExec(ParentFinality),
}

/// A proposal of the parent view that validators will be voting on.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ParentFinality {
    /// Block height of this proposal.
    pub height: ChainEpoch,
    /// The block hash of the parent, expressed as bytes
    pub block_hash: Vec<u8>,
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
