// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! The inner type of parent syncer

use crate::cache::{ParentViewPayload, TopdownViewContainer};
use crate::observe::ParentFinalityAcquired;
use crate::proxy::ParentQueryProxy;
use crate::{is_null_round_str, BlockHash, BlockHeight, Config, Error, ParentState};
use anyhow::anyhow;
use ethers::utils::hex;
use ipc_observability::{emit, serde::HexEncodableBlockHash};
use libp2p::futures::TryFutureExt;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tracing::instrument;

/// Sync every 10 parent block per sync call
const SYNC_BATCH_SIZE: usize = 10;

/// Parent syncer that constantly poll parent. This struct handles lotus null blocks and deferred
/// execution. For ETH based parent, it should work out of the box as well.
pub(crate) struct LotusParentSyncer<P> {
    config: Config,
    parent_proxy: Arc<P>,
    data_cache: Arc<Mutex<TopdownViewContainer>>,
}

impl<P> LotusParentSyncer<P>
where
    P: ParentQueryProxy + Send + Sync + 'static,
{
    pub fn new(
        config: Config,
        parent_proxy: Arc<P>,
        data_cache: Arc<Mutex<TopdownViewContainer>>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            parent_proxy,
            data_cache,
        })
    }

    pub async fn set_committed(&self, checkpoint: ParentState) {
        let mut cache = self.data_cache.lock().await;
        cache.set_committed(checkpoint);
    }

    pub async fn fetched_first_non_null_block(&self) -> Option<(BlockHeight, ParentViewPayload)> {
        let cache = self.data_cache.lock().await;
        cache.fetched_first_non_null_block()
    }

    /// Insert the height into cache when we see a new non null block
    pub async fn sync(&self) -> anyhow::Result<()> {
        let Some(chain_head) = self.finalized_chain_head().await? else {
            return Ok(());
        };

        let mut data_cache = self.data_cache.lock().await;

        let (mut latest_height_fetched, mut first_non_null_parent_hash) =
            data_cache.get_latest_parent_state();
        tracing::debug!(chain_head, latest_height_fetched, "syncing heights");

        if latest_height_fetched > chain_head {
            tracing::error!(
                chain_head,
                latest_height_fetched,
                "chain head went backwards, potential reorg detected from height"
            );
            return Err(anyhow!("parent reorg detected, latest_height_fetched {latest_height_fetched:}, chain_head: {chain_head:}"));
        }

        if latest_height_fetched == chain_head {
            tracing::debug!(
                chain_head,
                latest_height_fetched,
                "the parent has yet to produce a new block"
            );
            return Ok(());
        }

        let max_cache_blocks = self.config.max_cache_blocks();

        // performs a self rate limit against the RPC
        // also release the lock on data_cache so that voting loop can use it
        for _ in 0..SYNC_BATCH_SIZE {
            if data_cache.exceed_cache_size_limit(max_cache_blocks) {
                tracing::debug!("exceeded cache size limit");
                break;
            }

            first_non_null_parent_hash = self
                .poll_next(
                    &mut data_cache,
                    latest_height_fetched + 1,
                    first_non_null_parent_hash,
                )
                .await?;

            latest_height_fetched += 1;

            if latest_height_fetched == chain_head {
                tracing::debug!("reached the tip of the chain");
                break;
            }
        }

        Ok(())
    }
}

impl<P> LotusParentSyncer<P>
where
    P: ParentQueryProxy + Send + Sync + 'static,
{
    /// Poll the next block height. Returns finalized and executed block data.
    async fn poll_next(
        &self,
        data_cache: &mut MutexGuard<'_, TopdownViewContainer>,
        height: BlockHeight,
        parent_block_hash: BlockHash,
    ) -> Result<BlockHash, Error> {
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

                    data_cache.store_null_round(height)?;

                    emit(ParentFinalityAcquired {
                        source: "Parent syncer",
                        is_null: true,
                        block_height: height,
                        block_hash: None,
                        commitment_hash: None,
                        num_msgs: 0,
                        num_validator_changes: 0,
                    });

                    // Null block received, no block hash for the current height being polled.
                    // Return the previous parent hash as the non-null block hash.
                    return Ok(parent_block_hash);
                }
                return Err(Error::CannotQueryParent(
                    format!("get_block_hash: {e}"),
                    height,
                ));
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

        let data = fetch_data(
            self.parent_proxy.as_ref(),
            height,
            block_hash_res.block_hash,
        )
        .await?;

        tracing::debug!(
            height,
            staking_requests = data.1.len(),
            cross_messages = data.2.len(),
            "fetched data"
        );

        data_cache.store_non_null_round(height, data.0.clone(), data.1.clone(), data.2.clone())?;
        tracing::debug!(height, "non-null block pushed to cache");

        emit(ParentFinalityAcquired {
            source: "Parent syncer",
            is_null: false,
            block_height: height,
            block_hash: Some(HexEncodableBlockHash(data.0.clone())),
            // TODO Karel, Willes - when we introduce commitment hash, we should add it here
            commitment_hash: None,
            num_msgs: data.2.len(),
            num_validator_changes: data.1.len(),
        });

        Ok(data.0)
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
}

#[instrument(skip(parent_proxy))]
async fn fetch_data<P>(
    parent_proxy: &P,
    height: BlockHeight,
    block_hash: BlockHash,
) -> Result<ParentViewPayload, Error>
where
    P: ParentQueryProxy + Send + Sync + 'static,
{
    let changes_res = parent_proxy
        .get_validator_changes(height)
        .map_err(|e| Error::CannotQueryParent(format!("get_validator_changes: {e}"), height));

    let topdown_msgs_res = parent_proxy
        .get_top_down_msgs(height)
        .map_err(|e| Error::CannotQueryParent(format!("get_top_down_msgs: {e}"), height));

    let (changes_res, topdown_msgs_res) = tokio::join!(changes_res, topdown_msgs_res);
    let (changes_res, topdown_msgs_res) = (changes_res?, topdown_msgs_res?);

    if changes_res.block_hash != block_hash {
        tracing::warn!(
            height,
            change_set_hash = hex::encode(&changes_res.block_hash),
            block_hash = hex::encode(&block_hash),
            "change set block hash does not equal block hash",
        );
        return Err(Error::ParentChainReorgDetected);
    }

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

pub async fn fetch_topdown_events<P>(
    parent_proxy: &P,
    start_height: BlockHeight,
    end_height: BlockHeight,
) -> Result<Vec<(BlockHeight, ParentViewPayload)>, Error>
where
    P: ParentQueryProxy + Send + Sync + 'static,
{
    let mut events = Vec::new();
    for height in start_height..=end_height {
        match parent_proxy.get_block_hash(height).await {
            Ok(res) => {
                let (block_hash, changes, msgs) =
                    fetch_data(parent_proxy, height, res.block_hash).await?;

                if !(changes.is_empty() && msgs.is_empty()) {
                    events.push((height, (block_hash, changes, msgs)));
                }
            }
            Err(e) => {
                if is_null_round_str(&e.to_string()) {
                    continue;
                } else {
                    return Err(Error::CannotQueryParent(
                        format!("get_block_hash: {e}"),
                        height,
                    ));
                }
            }
        }
    }
    Ok(events)
}

// #[cfg(test)]
// mod tests {
//     use crate::cache::TopdownViewContainer;
//     use crate::proxy::ParentQueryProxy;
//     use crate::sync::syncer::LotusParentSyncer;
//     use crate::sync::FendermintStateQuery;
//     use crate::voting::VoteTally;
//     use crate::{
//         BlockHash, BlockHeight, Config, ParentState, SequentialKeyCache, NULL_ROUND_ERR_MSG,
//     };
//     use anyhow::anyhow;
//     use async_trait::async_trait;
//     use fendermint_vm_genesis::{Power, Validator};
//     use fvm_shared::chainid::ChainID;
//     use ipc_api::cross::IpcEnvelope;
//     use ipc_api::staking::PowerChangeRequest;
//     use ipc_provider::manager::{GetBlockHashResult, TopDownQueryPayload};
//     use std::sync::Arc;
//
//     /// How far behind the tip of the chain do we consider blocks final in the tests.
//     const FINALITY_DELAY: u64 = 2;
//
//     struct TestParentFinalityStateQuery {
//         latest_finality: ParentState,
//     }
//
//     impl FendermintStateQuery for TestParentFinalityStateQuery {
//         fn get_latest_topdown_parent_state(&self) -> anyhow::Result<Option<ParentState>> {
//             Ok(Some(self.latest_finality.clone()))
//         }
//
//         fn get_chain_id(&self) -> anyhow::Result<ChainID> {
//             Ok(ChainID::from(0))
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
//             Ok(self.blocks.last_key().unwrap())
//         }
//
//         async fn get_genesis_epoch(&self) -> anyhow::Result<BlockHeight> {
//             Ok(self.blocks.first_key().unwrap() - 1)
//         }
//
//         async fn get_block_hash(&self, height: BlockHeight) -> anyhow::Result<GetBlockHashResult> {
//             let r = self.blocks.get_value(height).unwrap();
//             if r.is_none() {
//                 return Err(anyhow!(NULL_ROUND_ERR_MSG));
//             }
//
//             for h in (self.blocks.first_key().unwrap()..height).rev() {
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
//         ) -> anyhow::Result<TopDownQueryPayload<Vec<PowerChangeRequest>>> {
//             Ok(TopDownQueryPayload {
//                 value: vec![],
//                 block_hash: self.blocks.get_value(height).cloned().unwrap().unwrap(),
//             })
//         }
//     }
//
//     async fn new_syncer(
//         blocks: SequentialKeyCache<BlockHeight, Option<BlockHash>>,
//         sync_many: bool,
//     ) -> LotusParentSyncer<TestParentFinalityStateQuery, TestParentProxy> {
//         let config = Config {
//             chain_head_delay: FINALITY_DELAY,
//             polling_interval: Default::default(),
//             vote_interval: Default::default(),
//             max_cache_blocks: None,
//         };
//         let genesis_epoch = blocks.first_key().unwrap();
//         let proxy = Arc::new(TestParentProxy { blocks });
//         let committed_finality = ParentState {
//             height: genesis_epoch,
//             block_hash: vec![0; 32],
//         };
//
//         let provider = TopdownViewContainer::new(committed_finality.clone()).unwrap();
//         let mut syncer = LotusParentSyncer::new(config, proxy, Arc::new(provider)).unwrap();
//
//         // Some tests expect to sync one block at a time.
//         syncer.sync_many = sync_many;
//
//         syncer
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
//             104 => Some(vec![4; 32]),   // after chain head delay, we fetch only to here
//             105 => Some(vec![5; 32]),
//             106 => Some(vec![6; 32])    // chain head
//         );
//
//         let mut syncer = new_syncer(parent_blocks, false).await;
//
//         for h in 101..=104 {
//             syncer.sync().await.unwrap();
//             let p = syncer.data_cache.latest_height();
//             assert_eq!(p, Some(h));
//         }
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
//         let mut syncer = new_syncer(parent_blocks, false).await;
//
//         for h in 101..=109 {
//             syncer.sync().await.unwrap();
//             assert_eq!(syncer.data_cache.latest_height(), Some(h));
//         }
//     }
// }
