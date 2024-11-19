// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tracks the current blockchain block mining activities and propagates to the parent subnet if
//! needed.

pub mod actor;
mod merkle;

use crate::fvm::activities::merkle::MerkleProofGen;
use fendermint_actor_activity_tracker::ValidatorData;
use fendermint_crypto::PublicKey;
use fvm_shared::clock::ChainEpoch;
use std::fmt::Debug;

pub struct BlockMined {
    pub(crate) validator: PublicKey,
}

#[derive(Debug, Clone)]
pub struct ActivityDetails<T> {
    pub details: Vec<T>,
    pub cycle_start: ChainEpoch,
}

/// Tracks the validator activities in the current blockchain
pub trait ValidatorActivityTracker {
    type ValidatorSummaryDetail: Clone + Debug;

    /// Mark the validator has mined the target block.
    fn track_block_mined(&mut self, block: BlockMined) -> anyhow::Result<()>;

    /// Get the validators activities summary since the checkpoint height
    fn get_activities_summary(
        &mut self,
    ) -> anyhow::Result<ActivityDetails<Self::ValidatorSummaryDetail>>;

    /// Purge the current validator activities summary
    fn purge_activities(&mut self) -> anyhow::Result<()>;
}

impl ActivityDetails<ValidatorData> {
    pub fn commitment(&self) -> anyhow::Result<Vec<u8>> {
        let gen = MerkleProofGen::new(self.details.as_slice())?;
        Ok(gen.root().to_fixed_bytes().to_vec())
    }
}

impl<T> ActivityDetails<T> {
    pub fn elapsed(&self, height: ChainEpoch) -> ChainEpoch {
        height.saturating_sub(self.cycle_start)
    }

    pub fn active_validators(&self) -> usize {
        self.details.len()
    }
}
