// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! The inner type of parent syncer

use crate::finality::ParentViewPayload;
use crate::proxy::ParentQueryProxy;
use crate::sync::{query_starting_finality, ParentFinalityStateQuery};
use crate::voting::{self, VoteTally};
use crate::{
    is_null_round_str, BlockHash, BlockHeight, CachedFinalityProvider, Config, Error, Toggle,
};
use anyhow::anyhow;
use async_stm::{atomically, atomically_or_err, StmError};
use ethers::utils::hex;
use std::sync::Arc;

/// Parent syncer that constantly poll parent. This struct handles lotus null blocks and deferred
/// execution. For ETH based parent, it should work out of the box as well.
pub(crate) struct LotusParentSyncer<T, P> {
    config: Config,
    parent_proxy: Arc<P>,
    provider: Arc<Toggle<CachedFinalityProvider<P>>>,
    vote_tally: VoteTally,
    query: Arc<T>,
}

impl<T, P> LotusParentSyncer<T, P>
where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
{
    pub fn new(
        config: Config,
        parent_proxy: Arc<P>,
        provider: Arc<Toggle<CachedFinalityProvider<P>>>,
        vote_tally: VoteTally,
        query: Arc<T>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            parent_proxy,
            provider,
            vote_tally,
            query,
        })
    }

    /// Insert the height into cache when we see a new non null block
    pub async fn sync(&mut self) -> anyhow::Result<()> {
        let chain_head = if let Some(h) = self.finalized_chain_head().await? {
            h
        } else {
            return Ok(());
        };
        let (latest_height_fetched, first_non_null_parent_hash) = self.get_from_cache().await;
        tracing::debug!(chain_head, latest_height_fetched, "syncing heights");

        if latest_height_fetched > chain_head {
            tracing::warn!(
                chain_head,
                latest_height_fetched,
                "chain head went backwards, potential detected from height"
            );
            return self.reset().await;
        }

        if latest_height_fetched == chain_head {
            tracing::debug!(
                chain_head,
                latest_height_fetched,
                "the parent has yet to produce a new block"
            );
            return Ok(());
        }

        if self.exceed_cache_size_limit().await {
            tracing::debug!("exceeded cache size limit");
            return Ok(());
        }

        match self
            .poll_next(latest_height_fetched + 1, first_non_null_parent_hash)
            .await
        {
            Ok(_) => {}
            Err(Error::ParentChainReorgDetected) => {
                tracing::warn!("potential reorg detected, clear cache and retry");
                self.reset().await?;
            }
            Err(e) => return Err(anyhow!(e)),
        }
        Ok(())
    }
}

impl<T, P> LotusParentSyncer<T, P>
where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
{
    async fn exceed_cache_size_limit(&self) -> bool {
        let max_cache_blocks = self.config.max_cache_blocks();
        atomically(|| self.provider.cached_blocks()).await > max_cache_blocks
    }

    /// Get the data needed to pull the next block
    async fn get_from_cache(&self) -> (BlockHeight, BlockHash) {
        // we are getting the latest height fetched in cache along with the first non null block
        // that is stored in cache.
        // we are doing two fetches in one `atomically` as if we get the data in two `atomically`,
        // the cache might be updated in between the two calls. `atomically` should guarantee atomicity.
        atomically(|| {
            let latest_height = if let Some(h) = self.provider.latest_height()? {
                h
            } else {
                unreachable!("guaranteed to have last committed finality, report bug please")
            };

            // first try to get the first non null block before latest_height + 1, i.e. from cache
            let prev_non_null_height =
                if let Some(height) = self.provider.first_non_null_block(latest_height)? {
                    tracing::debug!(height, "first non null block in cache");
                    height
                } else if let Some(p) = self.provider.last_committed_finality()? {
                    tracing::debug!(
                        height = p.height,
                        "first non null block not in cache, use latest finality"
                    );
                    p.height
                } else {
                    unreachable!("guaranteed to have last committed finality, report bug please")
                };

            let hash = if let Some(h) = self.provider.block_hash(prev_non_null_height)? {
                h
            } else {
                unreachable!(
                    "guaranteed to have hash as the height {} is found",
                    prev_non_null_height
                )
            };

            Ok((latest_height, hash))
        })
        .await
    }

    /// Poll the next block height. Returns finalized and executed block data.
    async fn poll_next(
        &mut self,
        height: BlockHeight,
        parent_block_hash: BlockHash,
    ) -> Result<(), Error> {
        tracing::debug!(
            height,
            parent_block_hash = hex::encode(&parent_block_hash),
            "polling height with parent hash"
        );

        let block_hash_res = match self.parent_proxy.get_block_hash(height).await {
            Ok(res) => res,
            Err(e) => {
                let err = e.to_string();
                if is_null_round_str(&err) {
                    tracing::debug!(
                        height,
                        "detected null round at height, inserted None to cache"
                    );

                    atomically_or_err::<_, Error, _>(|| {
                        self.provider.new_parent_view(height, None)?;
                        self.vote_tally
                            .add_block(height, None)
                            .map_err(map_voting_err)?;
                        Ok(())
                    })
                    .await?;

                    return Ok(());
                }
                return Err(Error::CannotQueryParent(err, height));
            }
        };

        if block_hash_res.parent_block_hash != parent_block_hash {
            tracing::warn!(
                height,
                parent_hash = hex::encode(&block_hash_res.parent_block_hash),
                previous_hash = hex::encode(&parent_block_hash),
                "parent block hash diff than previous hash",
            );
            return Err(Error::ParentChainReorgDetected);
        }

        let data = self.fetch_data(height, block_hash_res.block_hash).await?;
        atomically_or_err::<_, Error, _>(|| {
            self.provider.new_parent_view(height, Some(data.clone()))?;
            self.vote_tally
                .add_block(height, Some(data.0.clone()))
                .map_err(map_voting_err)?;
            tracing::debug!(height, "non-null block pushed to cache");
            Ok(())
        })
        .await?;

        Ok(())
    }

    async fn fetch_data(
        &self,
        height: BlockHeight,
        block_hash: BlockHash,
    ) -> Result<ParentViewPayload, Error> {
        let changes_res = self
            .parent_proxy
            .get_validator_changes(height)
            .await
            .map_err(|e| Error::CannotQueryParent(e.to_string(), height))?;

        if changes_res.block_hash != block_hash {
            tracing::warn!(
                height,
                change_set_hash = hex::encode(&changes_res.block_hash),
                block_hash = hex::encode(&block_hash),
                "change set block hash does not equal block hash",
            );
            return Err(Error::ParentChainReorgDetected);
        }

        let topdown_msgs_res = self
            .parent_proxy
            .get_top_down_msgs(height)
            .await
            .map_err(|e| Error::CannotQueryParent(e.to_string(), height))?;

        if topdown_msgs_res.block_hash != block_hash {
            tracing::warn!(
                height,
                topdown_msgs_hash = hex::encode(&topdown_msgs_res.block_hash),
                block_hash = hex::encode(&block_hash),
                "topdown messages block hash does not equal block hash",
            );
            return Err(Error::ParentChainReorgDetected);
        }

        Ok((block_hash, changes_res.value, topdown_msgs_res.value))
    }

    async fn finalized_chain_head(&self) -> anyhow::Result<Option<BlockHeight>> {
        let parent_chain_head_height = self.parent_proxy.get_chain_head_height().await?;
        // sanity check
        if parent_chain_head_height < self.config.chain_head_delay {
            tracing::debug!("latest height not more than the chain head delay");
            return Ok(None);
        }

        // we consider the chain head finalized only after the `chain_head_delay`
        Ok(Some(
            parent_chain_head_height - self.config.chain_head_delay,
        ))
    }

    /// Reset the cache in the face of a reorg
    async fn reset(&self) -> anyhow::Result<()> {
        let finality = query_starting_finality(&self.query, &self.parent_proxy).await?;
        atomically(|| self.provider.reset(finality.clone())).await;
        Ok(())
    }
}

fn map_voting_err(e: StmError<voting::Error>) -> StmError<Error> {
    match e {
        StmError::Abort(e) => {
            tracing::error!(
                error = e.to_string(),
                "failed to append block to voting tally"
            );
            StmError::Abort(Error::NotSequential)
        }
        StmError::Control(c) => StmError::Control(c),
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::proxy::ParentQueryProxy;
//     use crate::sync::syncer::LotusParentSyncer;
//     use crate::sync::ParentFinalityStateQuery;
//     use crate::voting::VoteTally;
//     use crate::{
//         BlockHash, BlockHeight, CachedFinalityProvider, Config, IPCParentFinality,
//         SequentialKeyCache, Toggle, NULL_ROUND_ERR_MSG,
//     };
//     use anyhow::anyhow;
//     use async_stm::atomically;
//     use async_trait::async_trait;
//     use fendermint_vm_genesis::{Power, Validator};
//     use ipc_api::cross::IpcEnvelope;
//     use ipc_api::staking::StakingChangeRequest;
//     use ipc_provider::manager::{GetBlockHashResult, TopDownQueryPayload};
//     use std::sync::Arc;
//
//     struct TestParentFinalityStateQuery {
//         latest_finality: IPCParentFinality,
//     }
//
//     impl ParentFinalityStateQuery for TestParentFinalityStateQuery {
//         fn get_latest_committed_finality(&self) -> anyhow::Result<Option<IPCParentFinality>> {
//             Ok(Some(self.latest_finality.clone()))
//         }
//         fn get_power_table(&self) -> anyhow::Result<Option<Vec<Validator<Power>>>> {
//             Ok(Some(vec![]))
//         }
//     }
//
//     struct TestParentProxy {
//         blocks: SequentialKeyCache<BlockHeight, Option<BlockHash>>,
//     }
//
//     #[async_trait]
//     impl ParentQueryProxy for TestParentProxy {
//         async fn get_chain_head_height(&self) -> anyhow::Result<BlockHeight> {
//             Ok(self.blocks.upper_bound().unwrap())
//         }
//
//         async fn get_genesis_epoch(&self) -> anyhow::Result<BlockHeight> {
//             Ok(self.blocks.lower_bound().unwrap() - 1)
//         }
//
//         async fn get_block_hash(&self, height: BlockHeight) -> anyhow::Result<GetBlockHashResult> {
//             let r = self.blocks.get_value(height).unwrap();
//             if r.is_none() {
//                 return Err(anyhow!(NULL_ROUND_ERR_MSG));
//             }
//
//             for h in (self.blocks.lower_bound().unwrap()..height).rev() {
//                 let v = self.blocks.get_value(h).unwrap();
//                 if v.is_none() {
//                     continue;
//                 }
//                 return Ok(GetBlockHashResult {
//                     parent_block_hash: v.clone().unwrap(),
//                     block_hash: r.clone().unwrap(),
//                 });
//             }
//             panic!("invalid testing data")
//         }
//
//         async fn get_top_down_msgs(
//             &self,
//             height: BlockHeight,
//         ) -> anyhow::Result<TopDownQueryPayload<Vec<IpcEnvelope>>> {
//             Ok(TopDownQueryPayload {
//                 value: vec![],
//                 block_hash: self.blocks.get_value(height).cloned().unwrap().unwrap(),
//             })
//         }
//
//         async fn get_validator_changes(
//             &self,
//             height: BlockHeight,
//         ) -> anyhow::Result<TopDownQueryPayload<Vec<StakingChangeRequest>>> {
//             Ok(TopDownQueryPayload {
//                 value: vec![],
//                 block_hash: self.blocks.get_value(height).cloned().unwrap().unwrap(),
//             })
//         }
//     }
//
//     async fn new_syncer(
//         blocks: SequentialKeyCache<BlockHeight, Option<BlockHash>>,
//     ) -> LotusParentSyncer<TestParentFinalityStateQuery, TestParentProxy> {
//         let config = Config {
//             chain_head_delay: 2,
//             polling_interval: Default::default(),
//             exponential_back_off: Default::default(),
//             exponential_retry_limit: 0,
//             max_proposal_range: Some(1),
//             max_cache_blocks: None,
//             proposal_delay: None,
//         };
//         let genesis_epoch = blocks.lower_bound().unwrap();
//         let proxy = Arc::new(TestParentProxy { blocks });
//         let committed_finality = IPCParentFinality {
//             height: genesis_epoch,
//             block_hash: vec![0; 32],
//         };
//
//         let provider = CachedFinalityProvider::new(
//             config.clone(),
//             genesis_epoch,
//             Some(committed_finality.clone()),
//             proxy.clone(),
//         );
//
//         let vote_tally = VoteTally::new(
//             vec![],
//             (
//                 committed_finality.height,
//                 committed_finality.block_hash.clone(),
//             ),
//         );
//
//         LotusParentSyncer::new(
//             config,
//             proxy,
//             Arc::new(Toggle::enabled(provider)),
//             vote_tally,
//             Arc::new(TestParentFinalityStateQuery {
//                 latest_finality: committed_finality,
//             }),
//         )
//         .unwrap()
//     }
//
//     /// Creates a mock of a new parent blockchain view. The key is the height and the value is the
//     /// block hash. If block hash is None, it means the current height is a null block.
//     macro_rules! new_parent_blocks {
//         ($($key:expr => $val:expr),* ,) => (
//             hash_map!($($key => $val),*)
//         );
//         ($($key:expr => $val:expr),*) => ({
//             let mut map = SequentialKeyCache::sequential();
//             $( map.append($key, $val).unwrap(); )*
//             map
//         });
//     }
//
//     #[tokio::test]
//     async fn happy_path() {
//         let parent_blocks = new_parent_blocks!(
//             100 => Some(vec![0; 32]),   // genesis block
//             101 => Some(vec![1; 32]),
//             102 => Some(vec![2; 32]),
//             103 => Some(vec![3; 32]),
//             104 => Some(vec![4; 32]),
//             105 => Some(vec![5; 32])    // chain head
//         );
//
//         let mut syncer = new_syncer(parent_blocks).await;
//
//         assert_eq!(syncer.sync_pointers.head(), 100);
//         assert_eq!(syncer.sync_pointers.tail(), None);
//
//         // sync block 101, which is a non-null block
//         let r = syncer.sync().await;
//         assert!(r.is_ok());
//         assert_eq!(syncer.sync_pointers.head(), 101);
//         assert_eq!(syncer.sync_pointers.tail(), Some((101, vec![1; 32])));
//         // latest height is None as we are yet to confirm block 101, so latest height should equal
//         // to the last committed finality initialized, which is the genesis block 100
//         assert_eq!(
//             atomically(|| syncer.provider.latest_height()).await,
//             Some(100)
//         );
//
//         // sync block 101, which is a non-null block
//         let r = syncer.sync().await;
//         assert!(r.is_ok());
//         assert_eq!(syncer.sync_pointers.head(), 102);
//         assert_eq!(syncer.sync_pointers.tail(), Some((102, vec![2; 32])));
//         assert_eq!(
//             atomically(|| syncer.provider.latest_height()).await,
//             Some(101)
//         );
//     }
//
//     #[tokio::test]
//     async fn with_non_null_block() {
//         let parent_blocks = new_parent_blocks!(
//             100 => Some(vec![0; 32]),   // genesis block
//             101 => None,
//             102 => None,
//             103 => None,
//             104 => Some(vec![4; 32]),
//             105 => None,
//             106 => None,
//             107 => None,
//             108 => Some(vec![5; 32]),
//             109 => None,
//             110 => None,
//             111 => None
//         );
//
//         let mut syncer = new_syncer(parent_blocks).await;
//
//         assert_eq!(syncer.sync_pointers.head(), 100);
//         assert_eq!(syncer.sync_pointers.tail(), None);
//
//         // sync block 101 to 103, which are null blocks
//         for h in 101..=103 {
//             let r = syncer.sync().await;
//             assert!(r.is_ok());
//             assert_eq!(syncer.sync_pointers.head(), h);
//             assert_eq!(syncer.sync_pointers.tail(), None);
//         }
//
//         // sync block 104, which is a non-null block
//         syncer.sync().await.unwrap();
//         assert_eq!(syncer.sync_pointers.head(), 104);
//         assert_eq!(syncer.sync_pointers.tail(), Some((104, vec![4; 32])));
//         // latest height is None as we are yet to confirm block 104, so latest height should equal
//         // to the last committed finality initialized, which is the genesis block 100
//         assert_eq!(
//             atomically(|| syncer.provider.latest_height()).await,
//             Some(100)
//         );
//
//         // sync block 105 to 107, which are null blocks
//         for h in 105..=107 {
//             let r = syncer.sync().await;
//             assert!(r.is_ok());
//             assert_eq!(syncer.sync_pointers.head(), h);
//             assert_eq!(syncer.sync_pointers.tail(), Some((104, vec![4; 32])));
//         }
//
//         // sync block 108, which is a non-null block
//         syncer.sync().await.unwrap();
//         assert_eq!(syncer.sync_pointers.head(), 108);
//         assert_eq!(syncer.sync_pointers.tail(), Some((108, vec![5; 32])));
//         // latest height is None as we are yet to confirm block 108, so latest height should equal
//         // to the previous confirmed block, which is 104
//         assert_eq!(
//             atomically(|| syncer.provider.latest_height()).await,
//             Some(104)
//         );
//     }
// }

#[cfg(test)]
mod tests {
    use crate::proxy::ParentQueryProxy;
    use crate::sync::syncer::LotusParentSyncer;
    use crate::sync::ParentFinalityStateQuery;
    use crate::voting::VoteTally;
    use crate::{
        BlockHash, BlockHeight, CachedFinalityProvider, Config, IPCParentFinality,
        SequentialKeyCache, Toggle, NULL_ROUND_ERR_MSG,
    };
    use anyhow::anyhow;
    use async_stm::atomically;
    use async_trait::async_trait;
    use fendermint_vm_genesis::{Power, Validator};
    use ipc_provider::manager::{GetBlockHashResult, TopDownQueryPayload};
    use ipc_sdk::cross::CrossMsg;
    use ipc_sdk::staking::StakingChangeRequest;
    use std::sync::Arc;

    struct TestParentFinalityStateQuery {
        latest_finality: IPCParentFinality,
    }

    impl ParentFinalityStateQuery for TestParentFinalityStateQuery {
        fn get_latest_committed_finality(&self) -> anyhow::Result<Option<IPCParentFinality>> {
            Ok(Some(self.latest_finality.clone()))
        }
        fn get_power_table(&self) -> anyhow::Result<Option<Vec<Validator<Power>>>> {
            Ok(Some(vec![]))
        }
    }

    struct TestParentProxy {
        blocks: SequentialKeyCache<BlockHeight, Option<BlockHash>>,
    }

    #[async_trait]
    impl ParentQueryProxy for TestParentProxy {
        async fn get_chain_head_height(&self) -> anyhow::Result<BlockHeight> {
            Ok(self.blocks.upper_bound().unwrap())
        }

        async fn get_genesis_epoch(&self) -> anyhow::Result<BlockHeight> {
            Ok(self.blocks.lower_bound().unwrap() - 1)
        }

        async fn get_block_hash(&self, height: BlockHeight) -> anyhow::Result<GetBlockHashResult> {
            let r = self.blocks.get_value(height).unwrap();
            if r.is_none() {
                return Err(anyhow!(NULL_ROUND_ERR_MSG));
            }

            for h in (self.blocks.lower_bound().unwrap()..height).rev() {
                let v = self.blocks.get_value(h).unwrap();
                if v.is_none() {
                    continue;
                }
                return Ok(GetBlockHashResult {
                    parent_block_hash: v.clone().unwrap(),
                    block_hash: r.clone().unwrap(),
                });
            }
            panic!("invalid testing data")
        }

        async fn get_top_down_msgs(
            &self,
            height: BlockHeight,
        ) -> anyhow::Result<TopDownQueryPayload<Vec<CrossMsg>>> {
            Ok(TopDownQueryPayload {
                value: vec![],
                block_hash: self.blocks.get_value(height).cloned().unwrap().unwrap(),
            })
        }

        async fn get_validator_changes(
            &self,
            height: BlockHeight,
        ) -> anyhow::Result<TopDownQueryPayload<Vec<StakingChangeRequest>>> {
            Ok(TopDownQueryPayload {
                value: vec![],
                block_hash: self.blocks.get_value(height).cloned().unwrap().unwrap(),
            })
        }
    }

    async fn new_syncer(
        blocks: SequentialKeyCache<BlockHeight, Option<BlockHash>>,
    ) -> LotusParentSyncer<TestParentFinalityStateQuery, TestParentProxy> {
        let config = Config {
            chain_head_delay: 2,
            polling_interval: Default::default(),
            exponential_back_off: Default::default(),
            exponential_retry_limit: 0,
            max_proposal_range: Some(1),
            max_cache_blocks: None,
            proposal_delay: None,
        };
        let genesis_epoch = blocks.lower_bound().unwrap();
        let proxy = Arc::new(TestParentProxy { blocks });
        let committed_finality = IPCParentFinality {
            height: genesis_epoch,
            block_hash: vec![0; 32],
        };

        let vote_tally = VoteTally::new(
            vec![],
            (
                committed_finality.height,
                committed_finality.block_hash.clone(),
            ),
        );

        let provider = CachedFinalityProvider::new(
            config.clone(),
            genesis_epoch,
            Some(committed_finality.clone()),
            proxy.clone(),
        );
        LotusParentSyncer::new(
            config,
            proxy,
            Arc::new(Toggle::enabled(provider)),
            vote_tally,
            Arc::new(TestParentFinalityStateQuery {
                latest_finality: committed_finality,
            }),
        )
        .unwrap()
    }

    /// Creates a mock of a new parent blockchain view. The key is the height and the value is the
    /// block hash. If block hash is None, it means the current height is a null block.
    macro_rules! new_parent_blocks {
        ($($key:expr => $val:expr),* ,) => (
            hash_map!($($key => $val),*)
        );
        ($($key:expr => $val:expr),*) => ({
            let mut map = SequentialKeyCache::sequential();
            $( map.append($key, $val).unwrap(); )*
            map
        });
    }

    #[tokio::test]
    async fn happy_path() {
        let parent_blocks = new_parent_blocks!(
            100 => Some(vec![0; 32]),   // genesis block
            101 => Some(vec![1; 32]),
            102 => Some(vec![2; 32]),
            103 => Some(vec![3; 32]),
            104 => Some(vec![4; 32]),   // after chain head delay, we fetch only to here
            105 => Some(vec![5; 32]),
            106 => Some(vec![6; 32])    // chain head
        );

        let mut syncer = new_syncer(parent_blocks).await;

        for h in 101..=104 {
            syncer.sync().await.unwrap();
            let p = atomically(|| syncer.provider.latest_height()).await;
            assert_eq!(p, Some(h));
        }
    }

    #[tokio::test]
    async fn with_non_null_block() {
        let parent_blocks = new_parent_blocks!(
            100 => Some(vec![0; 32]),   // genesis block
            101 => None,
            102 => None,
            103 => None,
            104 => Some(vec![4; 32]),
            105 => None,
            106 => None,
            107 => None,
            108 => Some(vec![5; 32]),
            109 => None,
            110 => None,
            111 => None
        );

        let mut syncer = new_syncer(parent_blocks).await;

        // sync block 101 to 103, which are null blocks
        for h in 101..=109 {
            syncer.sync().await.unwrap();
            assert_eq!(
                atomically(|| syncer.provider.latest_height()).await,
                Some(h)
            );
        }
    }
}
