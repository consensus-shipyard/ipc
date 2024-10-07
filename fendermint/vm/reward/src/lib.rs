// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;
use std::fmt::Debug;
use fvm_shared::clock::ChainEpoch;
use fendermint_crypto::PublicKey;

pub struct BlockMined {
    validator: PublicKey,
    height: ChainEpoch,
}

#[derive(Debug, Clone)]
pub struct ActivitySummary<T> {
    block_range: (ChainEpoch, ChainEpoch),
    details: HashMap<PublicKey, T>
}

/// Tracks the validator activities in the current blockchain
pub trait ValidatorActivityTracker {
    type ValidatorSummaryDetail: Clone + Debug + TryInto<Vec<u8>>;

    /// Mark the validator has mined the target block.
    fn track_block_mined(&mut self, block: BlockMined) -> anyhow::Result<()>;

    /// Get the validators activities summary since the checkpoint height
    fn get_activities_summary(&self) -> anyhow::Result<ActivitySummary<Self::ValidatorSummaryDetail>>;

    /// Purge the current validator activities summary
    fn purge_activities(&mut self) -> anyhow::Result<()>;
}