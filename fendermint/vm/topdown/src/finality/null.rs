// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::finality::{
    ensure_sequential, topdown_cross_msgs, validator_changes, ParentViewPayload,
};
use crate::{BlockHash, BlockHeight, Config, Error, IPCParentFinality, SequentialKeyCache};
use async_stm::{abort, atomically, Stm, StmResult, TVar};
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::PowerChangeRequest;
use std::cmp::max;

use fendermint_tracing::emit;
use fendermint_vm_event::ParentFinalityCommitted;

/// Finality provider that can handle null blocks
#[derive(Clone)]
pub struct FinalityWithNull {
    config: Config,
    genesis_epoch: BlockHeight,
    /// Cached data that always syncs with the latest parent chain proactively
    cached_data: TVar<SequentialKeyCache<BlockHeight, Option<ParentViewPayload>>>,
    /// This is a in memory view of the committed parent finality. We need this as a starting point
    /// for populating the cache
    last_committed_finality: TVar<Option<IPCParentFinality>>,
}

impl FinalityWithNull {
    pub fn new(
        config: Config,
        genesis_epoch: BlockHeight,
        committed_finality: Option<IPCParentFinality>,
    ) -> Self {
        Self {
            config,
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
    ) -> anyhow::Result<Option<Vec<PowerChangeRequest>>> {
        let r = atomically(|| self.handle_null_block(height, validator_changes, Vec::new)).await;
        Ok(r)
    }

    pub async fn top_down_msgs(
        &self,
        height: BlockHeight,
    ) -> anyhow::Result<Option<Vec<IpcEnvelope>>> {
        let r = atomically(|| self.handle_null_block(height, topdown_cross_msgs, Vec::new)).await;
        Ok(r)
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
        let height = if let Some(h) = self.propose_next_height()? {
            h
        } else {
            return Ok(None);
        };

        // safe to unwrap as we make sure null height will not be proposed
        let block_hash = self.block_hash_at_height(height)?.unwrap();

        let proposal = IPCParentFinality { height, block_hash };
        tracing::debug!(proposal = proposal.to_string(), "new proposal");
        Ok(Some(proposal))
    }

    pub fn set_new_finality(&self, finality: IPCParentFinality) -> Stm<()> {
        // the height to clear
        let height = finality.height;

        self.cached_data.update(|mut cache| {
            // only remove cache below height, but not at height, as we have delayed execution
            cache.remove_key_below(height);
            cache
        })?;

        let hash = hex::encode(&finality.block_hash);

        self.last_committed_finality.write(Some(finality))?;

        // emit event only after successful write
        emit!(ParentFinalityCommitted {
            block_height: height,
            block_hash: &hash
        });

        Ok(())
    }
}

impl FinalityWithNull {
    /// Returns the number of blocks cached.
    pub(crate) fn cached_blocks(&self) -> Stm<BlockHeight> {
        let cache = self.cached_data.read()?;
        Ok(cache.size() as BlockHeight)
    }

    pub(crate) fn block_hash_at_height(&self, height: BlockHeight) -> Stm<Option<BlockHash>> {
        if let Some(f) = self.last_committed_finality.read()?.as_ref() {
            if f.height == height {
                return Ok(Some(f.block_hash.clone()));
            }
        }

        self.get_at_height(height, |i| i.0.clone())
    }

    pub(crate) fn latest_height_in_cache(&self) -> Stm<Option<BlockHeight>> {
        let cache = self.cached_data.read()?;
        Ok(cache.upper_bound())
    }

    /// Get the latest height tracked in the provider, includes both cache and last committed finality
    pub(crate) fn latest_height(&self) -> Stm<Option<BlockHeight>> {
        let h = if let Some(h) = self.latest_height_in_cache()? {
            h
        } else if let Some(p) = self.last_committed_finality()? {
            p.height
        } else {
            return Ok(None);
        };
        Ok(Some(h))
    }

    /// Get the first non-null block in the range of earliest cache block till the height specified, inclusive.
    pub(crate) fn first_non_null_block(&self, height: BlockHeight) -> Stm<Option<BlockHeight>> {
        let cache = self.cached_data.read()?;
        Ok(cache.lower_bound().and_then(|lower_bound| {
            for h in (lower_bound..=height).rev() {
                if let Some(Some(_)) = cache.get_value(h) {
                    return Some(h);
                }
            }
            None
        }))
    }
}

/// All the private functions
impl FinalityWithNull {
    fn propose_next_height(&self) -> Stm<Option<BlockHeight>> {
        let latest_height = if let Some(h) = self.latest_height_in_cache()? {
            h
        } else {
            tracing::debug!("no proposal yet as height not available");
            return Ok(None);
        };

        let last_committed_height = if let Some(h) = self.last_committed_finality.read_clone()? {
            h.height
        } else {
            unreachable!("last committed finality will be available at this point");
        };

        // Propose only the next non-null block after the last committed height.
        // This avoids moving-target quorum where proposal candidates keep advancing.
        let next_non_null_height = if let Some(h) =
            self.next_non_null_block_after(last_committed_height, latest_height)?
        {
            h
        } else {
            tracing::debug!(
                last_committed_height,
                latest_height,
                "no next non-null block found after last committed"
            );
            return Ok(None);
        };

        // Require additional headroom before proposing this next block.
        let delay = self.config.proposal_delay();
        let min_observed_height = next_non_null_height.saturating_add(delay);
        if latest_height < min_observed_height {
            tracing::debug!(
                next_non_null_height,
                latest_height,
                delay,
                min_observed_height,
                "not enough delayed parent view to propose next non-null block"
            );
            return Ok(None);
        }

        tracing::debug!(
            proposal_height = next_non_null_height,
            delay,
            latest_height,
            "new proposal height (single-next policy)"
        );
        Ok(Some(next_non_null_height))
    }

    fn next_non_null_block_after(
        &self,
        lower_exclusive: BlockHeight,
        upper_inclusive: BlockHeight,
    ) -> Stm<Option<BlockHeight>> {
        let cache = self.cached_data.read()?;
        let lower = if let Some(lb) = cache.lower_bound() {
            max(lb, lower_exclusive.saturating_add(1))
        } else {
            return Ok(None);
        };

        if lower > upper_inclusive {
            return Ok(None);
        }

        for h in lower..=upper_inclusive {
            if let Some(Some(_)) = cache.get_value(h) {
                return Ok(Some(h));
            }
        }

        Ok(None)
    }

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
                tracing::debug!(height, "a null round detected, return default");
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
        validator_changes: Vec<PowerChangeRequest>,
        top_down_msgs: Vec<IpcEnvelope>,
    ) -> StmResult<(), Error> {
        if !top_down_msgs.is_empty() {
            // make sure incoming top down messages are ordered by nonce sequentially
            tracing::debug!(?top_down_msgs);
            ensure_sequential(&top_down_msgs, |msg| msg.local_nonce)?;
        };
        if !validator_changes.is_empty() {
            tracing::debug!(?validator_changes, "validator changes");
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

}

#[cfg(test)]
mod tests {
    use super::FinalityWithNull;
    use crate::finality::ParentViewPayload;
    use crate::{BlockHeight, Config, IPCParentFinality};
    use async_stm::{atomically, atomically_or_err};

    async fn new_provider(
        mut blocks: Vec<(BlockHeight, Option<ParentViewPayload>)>,
    ) -> FinalityWithNull {
        let config = Config {
            chain_head_delay: 2,
            polling_interval: Default::default(),
            exponential_back_off: Default::default(),
            exponential_retry_limit: 0,
            max_proposal_range: Some(6),
            max_cache_blocks: None,
            proposal_delay: Some(2),
        };
        let committed_finality = IPCParentFinality {
            height: blocks[0].0,
            block_hash: vec![0; 32],
        };

        blocks.remove(0);

        let f = FinalityWithNull::new(config, 1, Some(committed_finality));
        for (h, p) in blocks {
            atomically_or_err(|| f.new_parent_view(h, p.clone()))
                .await
                .unwrap();
        }
        f
    }

    #[tokio::test]
    async fn test_happy_path() {
        // Under single-next policy with proposal_delay=2 we propose the next non-null (101)
        // once we've observed at least two more parent heights.
        let parent_blocks = vec![
            (100, Some((vec![0; 32], vec![], vec![]))), // last committed block
            (101, Some((vec![1; 32], vec![], vec![]))), // cache start
            (102, Some((vec![2; 32], vec![], vec![]))),
            (103, Some((vec![3; 32], vec![], vec![]))),
            (104, Some((vec![4; 32], vec![], vec![]))), // final delayed height + proposal height
            (105, Some((vec![5; 32], vec![], vec![]))),
            (106, Some((vec![6; 32], vec![], vec![]))), // max proposal height (last committed + 6), first non null block
            (107, Some((vec![7; 32], vec![], vec![]))), // cache latest height
        ];
        let provider = new_provider(parent_blocks).await;

        let f = IPCParentFinality {
            height: 101,
            block_hash: vec![1; 32],
        };
        assert_eq!(
            atomically(|| provider.next_proposal()).await,
            Some(f.clone())
        );

        // Test set new finality
        atomically(|| provider.set_new_finality(f.clone())).await;

        assert_eq!(
            atomically(|| provider.last_committed_finality()).await,
            Some(f.clone())
        );

        // this ensures sequential insertion is still valid
        atomically_or_err(|| provider.new_parent_view(108, None))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_not_enough_view() {
        // Under single-next policy, this still proposes the first non-null after commitment
        // (101), because enough delayed view is available.
        let parent_blocks = vec![
            (100, Some((vec![0; 32], vec![], vec![]))), // last committed block
            (101, Some((vec![1; 32], vec![], vec![]))),
            (102, Some((vec![2; 32], vec![], vec![]))),
            (103, Some((vec![3; 32], vec![], vec![]))), // delayed height + final height
            (104, Some((vec![4; 32], vec![], vec![]))),
            (105, Some((vec![4; 32], vec![], vec![]))), // cache latest height, first non null block
                                                        // max proposal height is 106
        ];
        let provider = new_provider(parent_blocks).await;

        assert_eq!(
            atomically(|| provider.next_proposal()).await,
            Some(IPCParentFinality {
                height: 101,
                block_hash: vec![1; 32]
            })
        );
    }

    #[tokio::test]
    async fn test_with_all_null_blocks() {
        // max_proposal_range is 10. proposal_delay is 2
        let parent_blocks = vec![
            (102, Some((vec![2; 32], vec![], vec![]))), // last committed block
            (103, None),
            (104, None),
            (105, None),
            (106, None),
            (107, None),
            (108, None),
            (109, None),
            (110, Some((vec![4; 32], vec![], vec![]))), // cache latest height
                                                        // max proposal height is 112
        ];
        let mut provider = new_provider(parent_blocks).await;
        provider.config.max_proposal_range = Some(8);

        assert_eq!(atomically(|| provider.next_proposal()).await, None);
    }

    #[tokio::test]
    async fn test_with_partially_null_blocks_i() {
        // max_proposal_range is 10. proposal_delay is 2
        let parent_blocks = vec![
            (102, Some((vec![2; 32], vec![], vec![]))), // last committed block
            (103, None),
            (104, None), // we wont have a proposal because after delay, there is no more non-null proposal
            (105, None),
            (106, None),
            (107, None),
            (108, None), // delayed block
            (109, Some((vec![8; 32], vec![], vec![]))),
            (110, Some((vec![10; 32], vec![], vec![]))), // cache latest height, first non null block
                                                         // max proposal height is 112
        ];
        let mut provider = new_provider(parent_blocks).await;
        provider.config.max_proposal_range = Some(10);

        assert_eq!(atomically(|| provider.next_proposal()).await, None);
    }

    #[tokio::test]
    async fn test_with_partially_null_blocks_ii() {
        // Under single-next policy this proposes 103 (the first non-null after committed 102),
        // once enough delayed parent view is present.
        let parent_blocks = vec![
            (102, Some((vec![2; 32], vec![], vec![]))), // last committed block
            (103, Some((vec![3; 32], vec![], vec![]))),
            (104, None),
            (105, None),
            (106, None),
            (107, Some((vec![7; 32], vec![], vec![]))), // first non null after delay
            (108, None),                                // delayed block
            (109, None),
            (110, Some((vec![10; 32], vec![], vec![]))), // cache latest height, first non null block
                                                         // max proposal height is 112
        ];
        let mut provider = new_provider(parent_blocks).await;
        provider.config.max_proposal_range = Some(10);

        assert_eq!(
            atomically(|| provider.next_proposal()).await,
            Some(IPCParentFinality {
                height: 103,
                block_hash: vec![3; 32]
            })
        );
    }

    #[tokio::test]
    async fn test_with_partially_null_blocks_iii() {
        let parent_blocks = vec![
            (102, Some((vec![2; 32], vec![], vec![]))), // last committed block
            (103, Some((vec![3; 32], vec![], vec![]))),
            (104, None),
            (105, None),
            (106, None),
            (107, Some((vec![7; 32], vec![], vec![]))), // first non null delayed block, final
            (108, None),                                // delayed block
            (109, None),
            (110, Some((vec![10; 32], vec![], vec![]))), // first non null block
            (111, None),
            (112, None),
            // max proposal height is 122
        ];
        let mut provider = new_provider(parent_blocks).await;
        provider.config.max_proposal_range = Some(20);

        assert_eq!(
            atomically(|| provider.next_proposal()).await,
            Some(IPCParentFinality {
                height: 103,
                block_hash: vec![3; 32]
            })
        );
    }
}
