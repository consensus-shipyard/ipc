// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::finality::null::FinalityWithNull;
use crate::finality::ParentViewPayload;
use crate::proxy::ParentQueryProxy;
use crate::{
    BlockHash, BlockHeight, Config, Error, IPCParentFinality, ParentFinalityProvider,
    ParentViewProvider, TopdownProposal,
};
use async_stm::{Stm, StmResult};
use std::sync::Arc;

/// The finality provider that performs io to the parent if not found in cache
#[derive(Clone)]
pub struct CachedFinalityProvider {
    inner: FinalityWithNull,
}

#[async_trait::async_trait]
impl ParentViewProvider for CachedFinalityProvider {
    fn genesis_epoch(&self) -> anyhow::Result<BlockHeight> {
        self.inner.genesis_epoch()
    }
}

impl ParentFinalityProvider for CachedFinalityProvider {
    fn next_proposal(&self) -> Stm<Option<TopdownProposal>> {
        self.inner.next_proposal()
    }

    fn proposal_at_height(&self, height: BlockHeight) -> Stm<Option<TopdownProposal>> {
        self.inner.proposal_at_height(height)
    }

    fn set_new_finality(
        &self,
        finality: IPCParentFinality,
        previous_finality: Option<IPCParentFinality>,
    ) -> Stm<()> {
        self.inner.set_new_finality(finality, previous_finality)
    }
}

impl CachedFinalityProvider {
    /// Creates an uninitialized provider
    /// We need this because `fendermint` has yet to be initialized and might
    /// not be able to provide an existing finality from the storage. This provider requires an
    /// existing committed finality. Providing the finality will enable other functionalities.
    pub async fn uninitialized<T: ParentQueryProxy + Send + Sync + 'static>(
        config: Config,
        parent_client: Arc<T>,
    ) -> anyhow::Result<Self> {
        let genesis = parent_client.get_genesis_epoch().await?;
        Ok(Self::new(config, genesis, None))
    }
}

impl CachedFinalityProvider {
    pub(crate) fn new(
        config: Config,
        genesis_epoch: BlockHeight,
        committed_finality: Option<IPCParentFinality>,
    ) -> Self {
        let inner = FinalityWithNull::new(config.clone(), genesis_epoch, committed_finality);
        Self { inner }
    }

    pub fn block_hash(&self, height: BlockHeight) -> Stm<Option<BlockHash>> {
        self.inner.block_hash_at_height(height)
    }

    pub fn latest_height_in_cache(&self) -> Stm<Option<BlockHeight>> {
        self.inner.latest_height_in_cache()
    }

    /// Get the latest height tracked in the provider, includes both cache and last committed finality
    pub fn latest_height(&self) -> Stm<Option<BlockHeight>> {
        self.inner.latest_height()
    }

    pub fn last_committed_finality(&self) -> Stm<Option<IPCParentFinality>> {
        self.inner.last_committed_finality()
    }

    /// Clear the cache and set the committed finality to the provided value
    pub fn reset(&self, finality: IPCParentFinality) -> Stm<()> {
        self.inner.reset(finality)
    }

    pub fn new_parent_view(
        &self,
        height: BlockHeight,
        maybe_payload: Option<ParentViewPayload>,
    ) -> StmResult<(), Error> {
        self.inner.new_parent_view(height, maybe_payload)
    }

    /// Returns the number of blocks cached.
    pub fn cached_blocks(&self) -> Stm<BlockHeight> {
        self.inner.cached_blocks()
    }

    pub fn first_non_null_block(&self, height: BlockHeight) -> Stm<Option<BlockHeight>> {
        self.inner.first_non_null_block(height)
    }
}
