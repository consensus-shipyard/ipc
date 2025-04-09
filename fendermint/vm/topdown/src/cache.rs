// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{BlockHash, BlockHeight, Error, ParentState};
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::PowerChangeRequest;
use std::collections::VecDeque;

pub type ParentViewPayload = (BlockHash, Vec<PowerChangeRequest>, Vec<IpcEnvelope>);

/// TopdownViewContainer tracks the parent blocks pull along with the latest committed parent state
/// in the child subnet to provide validator to ensure:
///   1. syncer is pulling a "chained" parent
///   2. next vote for child subnet
///   3. Deal with null blocks in filecoin
#[derive(Clone)]
pub struct TopdownViewContainer {
    blocks: VecDeque<(BlockHeight, ParentViewPayload)>,
    parent_state: ParentState,
}

impl TopdownViewContainer {
    pub fn new(committed_parent_state: ParentState) -> Self {
        Self {
            blocks: Default::default(),
            parent_state: committed_parent_state,
        }
    }

    /// Get the latest parent state stored in the cache to pull the next block
    pub fn get_latest_parent_state(&self) -> (BlockHeight, BlockHash) {
        let p = self.parent_state.clone();
        (p.height, p.block_hash)
    }

    pub fn exceed_cache_size_limit(&self, limit: BlockHeight) -> bool {
        self.blocks.len() as BlockHeight > limit
    }

    pub fn set_committed(&mut self, checkpoint: ParentState) {
        while let Some((h, _)) = self.blocks.front() {
            if *h > checkpoint.height {
                break;
            }
            self.blocks.pop_front();
        }

        if self.parent_state.height < checkpoint.height {
            self.parent_state = checkpoint;
        }
    }

    /// Return the block with lowest block height that is not null
    pub(crate) fn next_vote(&self) -> Option<(BlockHeight, ParentViewPayload)> {
        self.blocks.front().cloned()
    }

    pub fn store_non_null_round(
        &mut self,
        height: BlockHeight,
        block_hash: BlockHash,
        validator_changes: Vec<PowerChangeRequest>,
        top_down_msgs: Vec<IpcEnvelope>,
    ) -> Result<(), Error> {
        let mut is_empty = true;

        if !top_down_msgs.is_empty() {
            // make sure incoming top down messages are ordered by nonce sequentially
            tracing::debug!(?top_down_msgs);
            ensure_sequential(&top_down_msgs, |msg| msg.local_nonce)?;
            is_empty = false;
        };
        if !validator_changes.is_empty() {
            tracing::debug!(?validator_changes, "validator changes");
            ensure_sequential(&validator_changes, |change| change.configuration_number)?;
            is_empty = false;
        }

        if !is_empty {
            self.blocks.push_back((
                height,
                (block_hash.clone(), validator_changes, top_down_msgs),
            ))
        }

        self.parent_state = ParentState { height, block_hash };

        Ok(())
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
    use super::TopdownViewContainer;
    use crate::cache::ParentViewPayload;
    use crate::{BlockHeight, Error, ParentState};

    async fn new_container(
        mut blocks: Vec<(BlockHeight, Option<ParentViewPayload>)>,
    ) -> TopdownViewContainer {
        let genesis = blocks.remove(0);
        let committed_finality = ParentState {
            height: genesis.0,
            block_hash: vec![0; 32],
        };

        let mut f = TopdownViewContainer::new(committed_finality);
        for (h, p) in blocks {
            if let Some(p) = p {
                f.store_non_null_round(h, p.0, p.1, p.2).unwrap();
            } else {
                f.store_null_round(h).unwrap();
            }
        }
        f
    }

    #[tokio::test]
    async fn test_sequential_insertion_works() {
        let parent_blocks = vec![
            (100, Some((vec![0; 32], vec![], vec![]))), // last committed block
            (101, Some((vec![1; 32], vec![], vec![]))), // fetched blocks start
            (102, Some((vec![2; 32], vec![], vec![]))),
            (103, Some((vec![3; 32], vec![], vec![]))),
            (104, Some((vec![4; 32], vec![], vec![]))),
            (105, Some((vec![5; 32], vec![], vec![]))),
            (106, Some((vec![6; 32], vec![], vec![]))),
            (107, Some((vec![7; 32], vec![], vec![]))),
        ];
        let mut provider = new_container(parent_blocks).await;

        assert_eq!(provider.get_latest_parent_state(), (107u64, vec![7u8; 32]));
        assert_eq!(
            provider.fetched_latest_parent_state(),
            Some((107, vec![7; 32]))
        );
        assert_eq!(
            provider.fetched_first_non_null_block(),
            Some((101, (vec![1; 32], vec![], vec![])))
        );

        let f = ParentState {
            height: 103,
            block_hash: vec![3; 32],
        };

        // Test set new finality
        provider.set_committed(f.clone());
        assert_eq!(provider.parent_state, f);
        assert_eq!(provider.get_latest_parent_state(), (107u64, vec![7u8; 32]));
        assert_eq!(
            provider.fetched_latest_parent_state(),
            Some((107u64, vec![7u8; 32]))
        );
        assert_eq!(
            provider.fetched_first_non_null_block(),
            Some((104u64, (vec![4u8; 32], vec![], vec![])))
        );

        // this ensures sequential insertion is still valid
        provider.store_null_round(108).unwrap();
        provider.store_null_round(109).unwrap();
        provider.store_null_round(110).unwrap();

        assert_eq!(provider.get_latest_parent_state(), (107u64, vec![7u8; 32]));
        assert_eq!(
            provider.fetched_latest_parent_state(),
            Some((107u64, vec![7u8; 32]))
        );
        assert_eq!(
            provider.fetched_first_non_null_block(),
            Some((104u64, (vec![4u8; 32], vec![], vec![])))
        );

        // non sequential inserts
        assert_eq!(
            provider.store_null_round(112).unwrap_err(),
            Error::NonSequentialParentViewInsert(111, 112)
        );
        assert_eq!(
            provider
                .store_non_null_round(112, vec![7u8; 32], vec![], vec![])
                .unwrap_err(),
            Error::NonSequentialParentViewInsert(111, 112)
        );

        // store a non null round
        provider
            .store_non_null_round(111, vec![11u8; 32], vec![], vec![])
            .unwrap();

        /*
        cached blocks
            (103, Some((vec![3; 32], vec![], vec![]))),  => last committed block
            (104, Some((vec![4; 32], vec![], vec![]))),
            (105, Some((vec![5; 32], vec![], vec![]))),
            (106, Some((vec![6; 32], vec![], vec![]))),
            (107, Some((vec![7; 32], vec![], vec![]))),
            (108, None),
            (109, None),
            (110, None),
            (111, Some((vec![11; 32], vec![], vec![]))),
         */
        assert_eq!(provider.get_latest_parent_state(), (111u64, vec![11u8; 32]));
        assert_eq!(
            provider.fetched_latest_parent_state(),
            Some((111u64, vec![11u8; 32]))
        );
        assert_eq!(
            provider.fetched_first_non_null_block(),
            Some((104u64, (vec![4u8; 32], vec![], vec![])))
        );

        let f = ParentState {
            height: 111,
            block_hash: vec![11; 32],
        };
        provider.set_committed(f.clone());
        /*
        cached blocks
            (111, Some((vec![11; 32], vec![], vec![]))), => last committed block
         */
        assert_eq!(provider.get_latest_parent_state(), (111u64, vec![11u8; 32]));
        assert_eq!(provider.fetched_latest_parent_state(), None);
        assert_eq!(provider.fetched_first_non_null_block(), None);

        provider.store_null_round(112).unwrap();
        provider.store_null_round(113).unwrap();
        provider.store_null_round(114).unwrap();

        /*
        cached blocks
            (111, Some((vec![11; 32], vec![], vec![]))), => last committed block
            (112, None)
            (113, None)
            (114, None)
         */
        assert_eq!(provider.get_latest_parent_state(), (111u64, vec![11u8; 32]));
        assert_eq!(provider.fetched_latest_parent_state(), None);
        assert_eq!(provider.fetched_first_non_null_block(), None);

        /*
        cached blocks
            (111, Some((vec![11; 32], vec![], vec![]))), => last committed block
            (112, None)
            (113, None)
            (114, None)
            (115, Some(...))
         */
        provider
            .store_non_null_round(115, vec![15u8; 32], vec![], vec![])
            .unwrap();
        assert_eq!(provider.get_latest_parent_state(), (115u64, vec![15u8; 32]));
        assert_eq!(
            provider.fetched_latest_parent_state(),
            Some((115u64, vec![15u8; 32]))
        );
        assert_eq!(
            provider.fetched_first_non_null_block(),
            Some((115u64, (vec![15u8; 32], vec![], vec![])))
        );

        /*
        cached blocks
            (111, Some((vec![11; 32], vec![], vec![]))), => last committed block
            (112, None)
            (113, None)
            (114, None)
            (115, Some(...))
            (116, Some(...))
         */
        provider
            .store_non_null_round(116, vec![16u8; 32], vec![], vec![])
            .unwrap();
        assert_eq!(provider.get_latest_parent_state(), (116u64, vec![16u8; 32]));
        assert_eq!(
            provider.fetched_latest_parent_state(),
            Some((116u64, vec![16u8; 32]))
        );
        assert_eq!(
            provider.fetched_first_non_null_block(),
            Some((115u64, (vec![15u8; 32], vec![], vec![])))
        );
    }
}
