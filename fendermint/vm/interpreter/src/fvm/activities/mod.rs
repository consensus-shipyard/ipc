// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tracks the current blockchain block mining activities and propagates to the parent subnet if
//! needed.

pub mod actor;
mod merkle;

use std::fmt::Debug;
use fendermint_actor_activity_tracker::ValidatorSummary;
use fendermint_crypto::PublicKey;
use ipc_api::checkpoint::ActivityCommitment;
use crate::fvm::activities::merkle::MerkleProofGen;

pub struct BlockMined {
    pub(crate) validator: PublicKey,
}

#[derive(Debug, Clone)]
pub struct ActivitySummary<T> {
    pub details: Vec<T>
}

/// Tracks the validator activities in the current blockchain
pub trait ValidatorActivityTracker {
    type ValidatorSummaryDetail: Clone + Debug;

    /// Mark the validator has mined the target block.
    fn track_block_mined(&mut self, block: BlockMined) -> anyhow::Result<()>;

    /// Get the validators activities summary since the checkpoint height
    fn get_activities_summary(&self) -> anyhow::Result<ActivitySummary<Self::ValidatorSummaryDetail>>;

    /// Purge the current validator activities summary
    fn purge_activities(&mut self) -> anyhow::Result<()>;
}

impl ActivitySummary<ValidatorSummary> {
    pub fn commitment(&self) -> anyhow::Result<ActivityCommitment> {
        let gen = MerkleProofGen::try_from(self.details.as_slice())?;
        Ok(ActivityCommitment{
            summary: gen.root().to_fixed_bytes().to_vec()
        })
    }
}