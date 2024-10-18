// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tracks the current blockchain block mining activities and propagates to the parent subnet if
//! needed.

pub mod actor;
mod merkle;

use crate::fvm::activities::merkle::MerkleProofGen;
use fendermint_actor_activity_tracker::ValidatorSummary;
use fendermint_crypto::PublicKey;
use fvm_shared::clock::ChainEpoch;
use ipc_api::checkpoint::ActivitySummary;
use std::fmt::Debug;

pub struct BlockMined {
    pub(crate) validator: PublicKey,
}

#[derive(Debug, Clone)]
pub struct ActivityDetails<T> {
    pub details: Vec<T>,
}

/// Tracks the validator activities in the current blockchain
pub trait ValidatorActivityTracker {
    type ValidatorSummaryDetail: Clone + Debug;

    /// Mark the validator has mined the target block.
    fn track_block_mined(&mut self, block: BlockMined) -> anyhow::Result<()>;

    /// Get the validators activities summary since the checkpoint height
    fn get_activities_summary(
        &self,
    ) -> anyhow::Result<ActivityDetails<Self::ValidatorSummaryDetail>>;

    /// Purge the current validator activities summary
    fn purge_activities(&mut self) -> anyhow::Result<()>;
}

impl ActivityDetails<ValidatorSummary> {
    pub fn commitment(&self, checkpoint_height: ChainEpoch) -> anyhow::Result<ActivitySummary> {
        let gen = MerkleProofGen::new(self.details.as_slice(), checkpoint_height)?;
        Ok(ActivitySummary {
            total_active_validators: self.details.len() as u64,
            commitment: gen.root().to_fixed_bytes().to_vec(),
        })
    }
}
