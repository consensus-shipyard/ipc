// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{BlockHash, BlockHeight, Error, IPCParentFinality, SequentialKeyCache};
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::PowerChangeRequest;

pub type ParentViewPayload = (BlockHash, Vec<PowerChangeRequest>, Vec<IpcEnvelope>);

/// Finality provider that can handle null blocks
#[derive(Clone)]
pub struct FinalityWithNull {
    blocks: SequentialKeyCache<BlockHeight, Option<ParentViewPayload>>,
    committed_checkpoint: IPCParentFinality,
}

impl FinalityWithNull {
    pub fn new(committed_checkpoint: IPCParentFinality) -> anyhow::Result<Self> {
        Ok(Self {
            blocks: SequentialKeyCache::sequential(),
            committed_checkpoint,
        })
    }

    /// Get the latest data stored in the cache to pull the next block
    pub fn latest_cached_data(&self) -> (BlockHeight, BlockHash) {
        // we are getting the latest height fetched in cache along with the first non null block
        // that is stored in cache.
        let latest_height = self.latest_height();

        // first try to get the first non null block before latest_height + 1, i.e. from cache
        let prev_non_null_height = if let Some(height) = self.first_non_null_block(latest_height) {
            tracing::debug!(height, "first non null block in cache");
            height
        } else {
            let p = self.last_committed_checkpoint();
            tracing::debug!(
                height = p.height,
                "first non null block not in cache, use latest finality"
            );
            p.height
        };

        let hash = if let Some(h) = self.block_hash_at_height(prev_non_null_height) {
            h
        } else {
            unreachable!(
                "guaranteed to have hash as the height {} is found",
                prev_non_null_height
            )
        };

        (latest_height, hash)
    }

    pub fn exceed_cache_size_limit(&self, limit: BlockHeight) -> bool {
        self.blocks.size() as BlockHeight > limit
    }

    pub fn last_committed_checkpoint(&self) -> IPCParentFinality {
        self.committed_checkpoint.clone()
    }

    pub fn new_parent_view(
        &mut self,
        height: BlockHeight,
        maybe_payload: Option<ParentViewPayload>,
    ) -> Result<(), Error> {
        if let Some((block_hash, validator_changes, top_down_msgs)) = maybe_payload {
            self.parent_block_filled(height, block_hash, validator_changes, top_down_msgs)
        } else {
            self.parent_null_round(height)
        }
    }

    pub fn finalized_checkpoint(&mut self, checkpoint: IPCParentFinality) {
        self.blocks.remove_key_below(checkpoint.height + 1);
        self.committed_checkpoint = checkpoint;
    }

    pub(crate) fn block_hash_at_height(&self, height: BlockHeight) -> Option<BlockHash> {
        if let Some(Some(v)) = self.blocks.get_value(height) {
            Some(v.0.clone())
        } else {
            None
        }
    }

    /// Get the latest height tracked in the provider, includes both cache and last committed checkpoint
    pub(crate) fn latest_height(&self) -> BlockHeight {
        if let Some(h) = self.blocks.upper_bound() {
            return h;
        }
        self.committed_checkpoint.height
    }

    pub fn get_payload_at_height(&self, height: BlockHeight) -> Option<&ParentViewPayload> {
        let h = self.blocks.get_value(height)?;
        h.as_ref()
    }

    /// Get the first non-null block in the range of earliest cache block till the height specified, inclusive.
    pub(crate) fn first_non_null_block(&self, height: BlockHeight) -> Option<BlockHeight> {
        self.blocks.lower_bound().and_then(|lower_bound| {
            for h in (lower_bound..=height).rev() {
                if let Some(Some(_)) = self.blocks.get_value(h) {
                    return Some(h);
                }
            }
            None
        })
    }

    fn parent_block_filled(
        &mut self,
        height: BlockHeight,
        block_hash: BlockHash,
        validator_changes: Vec<PowerChangeRequest>,
        top_down_msgs: Vec<IpcEnvelope>,
    ) -> Result<(), Error> {
        if !top_down_msgs.is_empty() {
            // make sure incoming top down messages are ordered by nonce sequentially
            tracing::debug!(?top_down_msgs);
            ensure_sequential(&top_down_msgs, |msg| msg.local_nonce)?;
        };
        if !validator_changes.is_empty() {
            tracing::debug!(?validator_changes, "validator changes");
            ensure_sequential(&validator_changes, |change| change.configuration_number)?;
        }

        self.blocks
            .append(height, Some((block_hash, validator_changes, top_down_msgs)))
            .map_err(Error::NonSequentialParentViewInsert)
    }

    /// When there is a new parent view, but it is actually a null round, call this function.
    fn parent_null_round(&mut self, height: BlockHeight) -> Result<(), Error> {
        self.blocks
            .append(height, None)
            .map_err(Error::NonSequentialParentViewInsert)
    }
}

fn ensure_sequential<T, F: Fn(&T) -> u64>(msgs: &[T], f: F) -> Result<(), Error> {
    if msgs.is_empty() {
        return Ok(());
    }

    let first = msgs.first().unwrap();
    let mut nonce = f(first);
    for msg in msgs.iter().skip(1) {
        if nonce + 1 != f(msg) {
            return Err(Error::NotSequential);
        }
        nonce += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::FinalityWithNull;
    use crate::finality::ParentViewPayload;
    use crate::{BlockHeight, Config, IPCParentFinality};

    async fn new_provider(
        mut blocks: Vec<(BlockHeight, Option<ParentViewPayload>)>,
    ) -> FinalityWithNull {
        let config = Config {
            chain_head_delay: 2,
            polling_interval: Default::default(),
            vote_interval: Default::default(),
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

        let mut f = FinalityWithNull::new(committed_finality).unwrap();
        for (h, p) in blocks {
            f.new_parent_view(h, p.clone()).unwrap();
        }
        f
    }

    #[tokio::test]
    async fn test_happy_path() {
        // max_proposal_range is 6. proposal_delay is 2
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
        let mut provider = new_provider(parent_blocks).await;

        let f = IPCParentFinality {
            height: 104,
            block_hash: vec![4; 32],
        };
        assert_eq!(provider.next_proposal(), Some(f.clone()));

        // Test set new finality
        provider.finalized_checkpoint(f.clone());

        assert_eq!(provider.last_committed_checkpoint(), f.clone());

        // this ensures sequential insertion is still valid
        provider.new_parent_view(108, None).unwrap()
    }

    #[tokio::test]
    async fn test_not_enough_view() {
        // max_proposal_range is 6. proposal_delay is 2
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
            provider.next_proposal(),
            Some(IPCParentFinality {
                height: 103,
                block_hash: vec![3; 32]
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

        assert_eq!(provider.next_proposal(), None);
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

        assert_eq!(provider.next_proposal(), None);
    }

    #[tokio::test]
    async fn test_with_partially_null_blocks_ii() {
        // max_proposal_range is 10. proposal_delay is 2
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
            provider.next_proposal(),
            Some(IPCParentFinality {
                height: 107,
                block_hash: vec![7; 32]
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
            provider.next_proposal(),
            Some(IPCParentFinality {
                height: 107,
                block_hash: vec![7; 32]
            })
        );
    }
}
