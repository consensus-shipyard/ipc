// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::syncer::error::Error;
use crate::syncer::store::ParentViewStore;
use crate::{BlockHash, BlockHeight, Checkpoint};
use arbitrary::Arbitrary;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// The content that validators gossip among each other
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Arbitrary)]
pub struct ObservationCommitment {
    /// The hash of the subnet's last committed block when this observation was made.
    /// Used to discard stale observations that are, e.g. replayed by an attacker
    /// at a later time. Also used to detect nodes that might be wrongly gossiping
    /// whilst being out of sync.
    local_hash: BlockHash,
    pub(crate) ballot: Ballot,
}

/// The actual content that validators should agree upon, or put in another way the content
/// that a quorum should be formed upon
#[derive(Serialize, Deserialize, Hash, Debug, Clone, Eq, PartialEq, Arbitrary)]
pub struct Ballot {
    pub(crate) parent_height: u64,
    /// The hash of the chain unit at that height. Usually a block hash, but could
    /// be another entity (e.g. tipset CID), depending on the parent chain
    /// and our interface to it. For example, if the parent is a Filecoin network,
    /// this would be a tipset CID coerced into a block hash if queried through
    /// the Eth API, or the tipset CID as-is if accessed through the Filecoin API.
    pub(crate) parent_hash: crate::Bytes,
    /// A rolling/cumulative commitment to topdown effects since the beginning of
    /// time, including the ones in this block.
    pub(crate) cumulative_effects_comm: crate::Bytes,
}

/// check in the store to see if there is a new observation available
pub fn deduce_new_observation<S: ParentViewStore>(
    _store: &S,
    _checkpoint: &Checkpoint,
) -> Result<ObservationCommitment, Error> {
    todo!()
}

impl Display for Ballot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ballot(parent_height={}, parent_hash={}, commitment={})",
            self.parent_height,
            hex::encode(&self.parent_hash),
            hex::encode(&self.cumulative_effects_comm),
        )
    }
}

impl Ballot {
    pub fn parent_height(&self) -> BlockHeight {
        self.parent_height
    }
}

impl ObservationCommitment {
    pub fn new(
        local_hash: crate::Bytes,
        parent_height: BlockHeight,
        parent_hash: crate::Bytes,
        commitment: crate::Bytes,
    ) -> Self {
        Self {
            local_hash,
            ballot: Ballot {
                parent_height,
                parent_hash,
                cumulative_effects_comm: commitment,
            },
        }
    }
}
