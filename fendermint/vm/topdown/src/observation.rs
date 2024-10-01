// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::syncer::error::Error;
use crate::syncer::store::ParentViewStore;
use crate::{BlockHash, BlockHeight, Bytes, Checkpoint};
use arbitrary::Arbitrary;
use cid::Cid;
use fvm_ipld_encoding::DAG_CBOR;
use multihash::Code;
use multihash::MultihashDigest;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::fmt::{Display, Formatter};

use crate::syncer::payload::ParentBlockView;

/// Default topdown observation height range
const DEFAULT_MAX_OBSERVATION_RANGE: BlockHeight = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationConfig {
    /// The max number of blocks one should make the topdown observation from the previous
    /// committed checkpoint
    pub max_observation_range: Option<BlockHeight>,
}

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

/// Check in the store to see if there is a new observation available.
/// Caller should make sure:
/// - the store has votes since the last committed checkpoint
/// - the votes have at least 1 non-null block
pub fn deduce_new_observation<S: ParentViewStore>(
    store: &S,
    checkpoint: &Checkpoint,
    config: &ObservationConfig,
) -> Result<ObservationCommitment, Error> {
    let Some(latest_height) = store.max_parent_view_height()? else {
        tracing::info!("no observation yet as height not available");
        return Err(Error::BlockStoreEmpty);
    };

    if latest_height < checkpoint.target_height() {
        tracing::info!("committed vote height more than latest parent view");
        return Err(Error::CommittedParentHeightNotPurged);
    }

    let max_observation_height = checkpoint.target_height() + config.max_observation_range();
    let candidate_height = min(max_observation_height, latest_height);
    tracing::debug!(
        max_observation_height,
        candidate_height,
        "propose observation height"
    );

    // aggregate commitment for the observation
    let mut agg = LinearizedParentBlockView::from(checkpoint);
    for h in checkpoint.target_height() + 1..=candidate_height {
        let Some(p) = store.get(h)? else {
            tracing::debug!(height = h, "not parent block view");
            return Err(Error::MissingBlockView(h, candidate_height));
        };

        agg.append(p)?;
    }

    // TODO: integrate local hash
    let observation = agg.into_commitment(vec![])?;
    tracing::info!(
        height = observation.ballot.parent_height,
        "new observation derived"
    );

    Ok(observation)
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

impl AsRef<[u8]> for Ballot {
    fn as_ref(&self) -> &[u8] {
        todo!()
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

impl ObservationConfig {
    pub fn max_observation_range(&self) -> BlockHeight {
        self.max_observation_range
            .unwrap_or(DEFAULT_MAX_OBSERVATION_RANGE)
    }
}

struct LinearizedParentBlockView {
    parent_height: u64,
    parent_hash: Option<BlockHash>,
    cumulative_effects_comm: Bytes,
}

impl From<&Checkpoint> for LinearizedParentBlockView {
    fn from(value: &Checkpoint) -> Self {
        LinearizedParentBlockView {
            parent_height: value.target_height(),
            parent_hash: Some(value.target_hash().clone()),
            cumulative_effects_comm: value.cumulative_effects_comm().clone(),
        }
    }
}

impl LinearizedParentBlockView {
    fn new_commitment(&mut self, to_append: Bytes) {
        let bytes = [
            self.cumulative_effects_comm.as_slice(),
            to_append.as_slice(),
        ]
        .concat();
        let cid = Cid::new_v1(DAG_CBOR, Code::Blake2b256.digest(&bytes));
        self.cumulative_effects_comm = cid.to_bytes();
    }

    pub fn append(&mut self, view: ParentBlockView) -> Result<(), Error> {
        if self.parent_height + 1 != view.parent_height {
            return Err(Error::NotSequential);
        }

        self.parent_height += 1;

        self.new_commitment(view.effects_commitment()?);

        if let Some(p) = view.payload {
            self.parent_hash = Some(p.parent_hash);
        }

        Ok(())
    }

    fn into_commitment(self, local_hash: BlockHash) -> Result<ObservationCommitment, Error> {
        let Some(hash) = self.parent_hash else {
            return Err(Error::CannotCommitObservationAtNullBlock(
                self.parent_height,
            ));
        };
        Ok(ObservationCommitment::new(
            local_hash,
            self.parent_height,
            hash,
            self.cumulative_effects_comm,
        ))
    }
}
