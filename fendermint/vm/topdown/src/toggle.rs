// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{
    BlockHash, BlockHeight, CachedFinalityProvider, Error, IPCParentFinality,
    ParentFinalityProvider, ParentViewProvider,
};
use anyhow::anyhow;
use async_stm::{Stm, StmResult};
use ipc_sdk::cross::CrossMsg;
use ipc_sdk::staking::StakingChangeRequest;

/// The parent finality provider could have all functionalities disabled.
#[derive(Clone)]
pub struct Toggle<P> {
    inner: Option<P>,
}

impl<P> Toggle<P> {
    pub fn disabled() -> Self {
        Self { inner: None }
    }

    pub fn enabled(inner: P) -> Self {
        Self { inner: Some(inner) }
    }

    pub fn is_enabled(&self) -> bool {
        self.inner.is_some()
    }

    fn perform_or_else<F, T, E>(&self, f: F, other: T) -> Result<T, E>
    where
        F: FnOnce(&P) -> Result<T, E>,
    {
        match &self.inner {
            Some(p) => f(p),
            None => Ok(other),
        }
    }
}

#[async_trait::async_trait]
impl<P: ParentViewProvider + Send + Sync + 'static> ParentViewProvider for Toggle<P> {
    async fn validator_changes(
        &self,
        height: BlockHeight,
    ) -> anyhow::Result<Vec<StakingChangeRequest>> {
        match self.inner.as_ref() {
            Some(p) => p.validator_changes(height).await,
            None => Err(anyhow!("provider is toggled off")),
        }
    }

    async fn top_down_msgs(
        &self,
        height: BlockHeight,
        block_hash: &BlockHash,
    ) -> anyhow::Result<Vec<CrossMsg>> {
        match self.inner.as_ref() {
            Some(p) => p.top_down_msgs(height, block_hash).await,
            None => Err(anyhow!("provider is toggled off")),
        }
    }
}

impl<P: ParentFinalityProvider + Send + Sync + 'static> ParentFinalityProvider for Toggle<P> {
    fn next_proposal(&self) -> Stm<Option<IPCParentFinality>> {
        self.perform_or_else(|p| p.next_proposal(), None)
    }

    fn check_proposal(&self, proposal: &IPCParentFinality) -> Stm<bool> {
        self.perform_or_else(|p| p.check_proposal(proposal), false)
    }

    fn set_new_finality(&self, finality: IPCParentFinality) -> Stm<()> {
        self.perform_or_else(|p| p.set_new_finality(finality), ())
    }
}

impl<P> Toggle<CachedFinalityProvider<P>> {
    pub fn latest_height_hash(&self) -> Stm<Option<(BlockHeight, BlockHash)>> {
        self.perform_or_else(|p| p.latest_height_hash(), None)
    }

    pub fn last_committed_finality(&self) -> Stm<Option<IPCParentFinality>> {
        self.perform_or_else(|p| p.last_committed_finality(), None)
    }

    pub fn new_parent_view(
        &self,
        height: BlockHeight,
        block_hash: BlockHash,
        validator_changes: Vec<StakingChangeRequest>,
        top_down_msgs: Vec<CrossMsg>,
    ) -> StmResult<(), Error> {
        self.perform_or_else(
            |p| p.new_parent_view(height, block_hash, validator_changes, top_down_msgs),
            (),
        )
    }

    pub fn reset(&self, finality: IPCParentFinality) -> Stm<()> {
        self.perform_or_else(|p| p.reset(finality), ())
    }
}
