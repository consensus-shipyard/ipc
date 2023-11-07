// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::finality::{
    ensure_sequential, topdown_cross_msgs, validator_changes, ParentViewPayload,
};
use crate::{BlockHash, BlockHeight, Error, IPCParentFinality, SequentialKeyCache};
use async_stm::{abort, atomically, Stm, StmResult, TVar};
use ipc_sdk::cross::CrossMsg;
use ipc_sdk::staking::StakingChangeRequest;

/// Finality provider that can handle null blocks
#[derive(Clone)]
pub struct FinalityWithNull {
    genesis_epoch: BlockHeight,
    /// Cached data that always syncs with the latest parent chain proactively
    cached_data: TVar<SequentialKeyCache<BlockHeight, Option<ParentViewPayload>>>,
    /// This is a in memory view of the committed parent finality. We need this as a starting point
    /// for populating the cache
    last_committed_finality: TVar<Option<IPCParentFinality>>,
}

impl FinalityWithNull {
    pub fn new(genesis_epoch: BlockHeight, committed_finality: Option<IPCParentFinality>) -> Self {
        Self {
            genesis_epoch,
            cached_data: TVar::new(SequentialKeyCache::sequential()),
            last_committed_finality: TVar::new(committed_finality),
        }
    }

    pub fn genesis_epoch(&self) -> anyhow::Result<BlockHeight> {
        Ok(self.genesis_epoch)
    }

    pub async fn validator_changes(
        &self,
        height: BlockHeight,
    ) -> anyhow::Result<Option<Vec<StakingChangeRequest>>> {
        let r = atomically(|| self.handle_null_block(height, validator_changes, Vec::new)).await;
        Ok(r)
    }

    pub async fn top_down_msgs(
        &self,
        height: BlockHeight,
    ) -> anyhow::Result<Option<Vec<CrossMsg>>> {
        let r = atomically(|| self.handle_null_block(height, topdown_cross_msgs, Vec::new)).await;
        Ok(r)
    }

    pub fn first_non_null_parent_hash(&self, height: BlockHeight) -> Stm<Option<BlockHash>> {
        let cache = self.cached_data.read()?;
        if let Some(lower_bound) = cache.lower_bound() {
            for h in (lower_bound..height).rev() {
                if let Some(Some(p)) = cache.get_value(h) {
                    return Ok(Some(p.0.clone()));
                }
            }
        }

        // nothing is found in cache, check the last committed finality
        let last_committed_finality = self.last_committed_finality.read_clone()?;
        if let Some(f) = last_committed_finality {
            Ok(Some(f.block_hash))
        } else {
            Ok(None)
        }
    }

    pub fn last_committed_finality(&self) -> Stm<Option<IPCParentFinality>> {
        self.last_committed_finality.read_clone()
    }

    /// Clear the cache and set the committed finality to the provided value
    pub fn reset(&self, finality: IPCParentFinality) -> Stm<()> {
        self.cached_data.write(SequentialKeyCache::sequential())?;
        self.last_committed_finality.write(Some(finality))
    }

    pub fn new_parent_view(
        &self,
        height: BlockHeight,
        maybe_payload: Option<ParentViewPayload>,
    ) -> StmResult<(), Error> {
        if let Some((block_hash, validator_changes, top_down_msgs)) = maybe_payload {
            self.parent_block_filled(height, block_hash, validator_changes, top_down_msgs)
        } else {
            self.parent_null_round(height)
        }
    }

    pub fn next_proposal(&self) -> Stm<Option<IPCParentFinality>> {
        let height = if let Some(h) = self.latest_height()? {
            h
        } else {
            tracing::debug!("no proposal yet as height not available");
            return Ok(None);
        };

        let block_hash = if let Some(h) = self.block_hash_at_height(height)? {
            h
        } else {
            // Oops, we have a null round in parent, skip this proposal and wait for future blocks.
            return Ok(None);
        };

        let proposal = IPCParentFinality { height, block_hash };
        tracing::debug!("new proposal: {proposal:?}");
        Ok(Some(proposal))
    }

    pub fn check_proposal(&self, proposal: &IPCParentFinality) -> Stm<bool> {
        if !self.check_height(proposal)? {
            return Ok(false);
        }
        self.check_block_hash(proposal)
    }

    pub fn set_new_finality(
        &self,
        finality: IPCParentFinality,
        previous_finality: Option<IPCParentFinality>,
    ) -> Stm<()> {
        debug_assert!(previous_finality == self.last_committed_finality.read_clone()?);

        // the height to clear
        let height = finality.height;

        self.cached_data.update(|mut cache| {
            cache.remove_key_below(height + 1);
            cache
        })?;

        self.last_committed_finality.write(Some(finality))
    }
}

impl FinalityWithNull {
    pub(crate) fn block_hash_at_height(&self, height: BlockHeight) -> Stm<Option<BlockHash>> {
        self.get_at_height(height, |i| i.0.clone())
    }

    pub(crate) fn latest_height(&self) -> Stm<Option<BlockHeight>> {
        let cache = self.cached_data.read()?;
        Ok(cache.upper_bound())
    }
}

/// All the private functions
impl FinalityWithNull {
    fn handle_null_block<T, F: Fn(&ParentViewPayload) -> T, D: Fn() -> T>(
        &self,
        height: BlockHeight,
        f: F,
        d: D,
    ) -> Stm<Option<T>> {
        let cache = self.cached_data.read()?;
        Ok(cache.get_value(height).map(|v| {
            if let Some(i) = v.as_ref() {
                f(i)
            } else {
                tracing::debug!("height: {height} is a null round, return default");
                d()
            }
        }))
    }

    fn get_at_height<T, F: Fn(&ParentViewPayload) -> T>(
        &self,
        height: BlockHeight,
        f: F,
    ) -> Stm<Option<T>> {
        let cache = self.cached_data.read()?;
        Ok(if let Some(Some(v)) = cache.get_value(height) {
            Some(f(v))
        } else {
            None
        })
    }

    fn parent_block_filled(
        &self,
        height: BlockHeight,
        block_hash: BlockHash,
        validator_changes: Vec<StakingChangeRequest>,
        top_down_msgs: Vec<CrossMsg>,
    ) -> StmResult<(), Error> {
        if !top_down_msgs.is_empty() {
            // make sure incoming top down messages are ordered by nonce sequentially
            tracing::debug!("top down messages: {top_down_msgs:#?}");
            ensure_sequential(&top_down_msgs, |msg| msg.msg.nonce)?;
        };
        if !validator_changes.is_empty() {
            tracing::debug!("validator changes: {validator_changes:#?}");
            ensure_sequential(&validator_changes, |change| change.configuration_number)?;
        }

        let r = self.cached_data.modify(|mut cache| {
            let r = cache
                .append(height, Some((block_hash, validator_changes, top_down_msgs)))
                .map_err(Error::NonSequentialParentViewInsert);
            (cache, r)
        })?;

        if let Err(e) = r {
            return abort(e);
        }

        Ok(())
    }

    /// When there is a new parent view, but it is actually a null round, call this function.
    fn parent_null_round(&self, height: BlockHeight) -> StmResult<(), Error> {
        let r = self.cached_data.modify(|mut cache| {
            let r = cache
                .append(height, None)
                .map_err(Error::NonSequentialParentViewInsert);
            (cache, r)
        })?;

        if let Err(e) = r {
            return abort(e);
        }

        Ok(())
    }

    fn check_height(&self, proposal: &IPCParentFinality) -> Stm<bool> {
        let binding = self.last_committed_finality.read()?;
        // last committed finality is not ready yet, we don't vote, just reject
        let last_committed_finality = if let Some(f) = binding.as_ref() {
            f
        } else {
            return Ok(false);
        };

        // the incoming proposal has height already committed, reject
        if last_committed_finality.height >= proposal.height {
            return Ok(false);
        }

        if let Some(latest_height) = self.latest_height()? {
            // requires the incoming height cannot be more advanced than our trusted parent node
            Ok(latest_height >= proposal.height)
        } else {
            // latest height is not found, meaning we dont have any prefetched cache, we just be
            // strict and vote no simply because we don't know..
            Ok(false)
        }
    }

    fn check_block_hash(&self, proposal: &IPCParentFinality) -> Stm<bool> {
        Ok(
            if let Some(block_hash) = self.block_hash_at_height(proposal.height)? {
                block_hash == proposal.block_hash
            } else {
                false
            },
        )
    }
}
