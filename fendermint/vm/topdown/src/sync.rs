// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! A constant running process that fetch or listener to parent state

use crate::error::Error;
use crate::proxy::{IPCProviderProxy, ParentQueryProxy};
use crate::{
    BlockHash, BlockHeight, CachedFinalityProvider, Config, IPCParentFinality,
    ParentFinalityProvider, Toggle,
};
use anyhow::anyhow;
use async_stm::{atomically, atomically_or_err};
use ipc_sdk::cross::CrossMsg;
use ipc_sdk::staking::StakingChangeRequest;
use std::cmp::min;
use std::sync::Arc;
use std::time::Duration;

/// The max number of blocks polling should query each parent view update. If the number of blocks
/// polled equals this value, it would stop polling for this iteration and commit the result to cache.
const MAX_PARENT_VIEW_BLOCK_GAP: BlockHeight = 100;
/// When polling parent view, if the number of top down messages exceeds this limit,
/// the polling will stop for this iteration and commit the result to cache.
const TOPDOWN_MSG_LEN_THRESHOLD: usize = 500;

/// Query the parent finality from the block chain state
pub trait ParentFinalityStateQuery {
    /// Get the latest committed finality from the state
    fn get_latest_committed_finality(&self) -> anyhow::Result<Option<IPCParentFinality>>;
}

/// Constantly syncing with parent through polling
struct PollingParentSyncer<T> {
    config: Config,
    parent_view_provider: Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>,
    parent_client: Arc<IPCProviderProxy>,
    committed_state_query: Arc<T>,
}

/// Queries the starting finality for polling. First checks the committed finality, if none, that
/// means the chain has just started, then query from the parent to get the genesis epoch.
async fn query_starting_finality<T: ParentFinalityStateQuery + Send + Sync + 'static>(
    query: &Arc<T>,
    parent_client: &Arc<IPCProviderProxy>,
) -> anyhow::Result<IPCParentFinality> {
    loop {
        let mut finality = match query.get_latest_committed_finality() {
            Ok(Some(finality)) => finality,
            Ok(None) => {
                tracing::debug!("app not ready for query yet");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
            Err(e) => {
                tracing::warn!("cannot get committed finality: {e}");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        tracing::info!("latest finality committed: {finality:?}");

        // this means there are no previous committed finality yet, we fetch from parent to get
        // the genesis epoch of the current subnet and its corresponding block hash.
        if finality.height == 0 {
            let genesis_epoch = parent_client.get_genesis_epoch().await?;
            tracing::debug!("obtained genesis epoch: {genesis_epoch:?}");
            let r = parent_client.get_block_hash(genesis_epoch).await?;
            tracing::debug!("obtained genesis block hash: {:?}", r.block_hash);

            finality = IPCParentFinality {
                height: genesis_epoch,
                block_hash: r.block_hash,
            };
            tracing::info!(
                "no previous finality committed, fetched from genesis epoch: {finality:?}"
            );
        }

        return Ok(finality);
    }
}

/// Start the polling parent syncer in the background
pub async fn launch_polling_syncer<T: ParentFinalityStateQuery + Send + Sync + 'static>(
    query: T,
    config: Config,
    view_provider: Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>,
    parent_client: Arc<IPCProviderProxy>,
) -> anyhow::Result<()> {
    if !view_provider.is_enabled() {
        return Err(anyhow!("provider not enabled, enable to run syncer"));
    }

    tracing::info!("launching polling syncer");

    let query = Arc::new(query);
    let finality = query_starting_finality(&query, &parent_client).await?;
    atomically(|| view_provider.set_new_finality(finality.clone(), None)).await;
    tracing::info!("obtained last committed finality: {finality:?}");

    let poll = PollingParentSyncer::new(config, view_provider, parent_client, query);
    poll.start();

    Ok(())
}

impl<T> PollingParentSyncer<T> {
    pub fn new(
        config: Config,
        parent_view_provider: Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>,
        parent_client: Arc<IPCProviderProxy>,
        query: Arc<T>,
    ) -> Self {
        Self {
            config,
            parent_view_provider,
            parent_client,
            committed_state_query: query,
        }
    }
}

impl<T: ParentFinalityStateQuery + Send + Sync + 'static> PollingParentSyncer<T> {
    /// Start the parent finality listener in the background
    pub fn start(self) {
        let config = self.config;
        let provider = self.parent_view_provider;
        let parent_client = self.parent_client;
        let query = self.committed_state_query;

        let mut interval = tokio::time::interval(config.polling_interval);

        tokio::spawn(async move {
            loop {
                interval.tick().await;

                if let Err(e) = sync_with_parent(&config, &parent_client, &provider, &query).await {
                    tracing::error!("sync with parent encountered error: {e}");
                }
            }
        });
    }
}

/// Syncing with parent with the below steps:
/// 1. Get the latest height in cache or latest height committed increment by 1 as the
///    starting height
/// 2. Get the latest chain head height deduct away N blocks as the ending height
/// 3. Fetches the data between starting and ending height
/// 4. Update the data into cache
async fn sync_with_parent<T: ParentFinalityStateQuery + Send + Sync + 'static>(
    config: &Config,
    parent_proxy: &Arc<IPCProviderProxy>,
    provider: &Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>,
    query: &Arc<T>,
) -> anyhow::Result<()> {
    let (last_recorded_height, last_height_hash) =
        if let Some(h) = last_recorded_height(provider).await? {
            h
        } else {
            // cannot get starting recorded height, we just wait for the next loop execution
            return Ok(());
        };

    let parent_chain_head_height = parent_proxy.get_chain_head_height().await?;
    // sanity check
    if parent_chain_head_height < config.chain_head_delay {
        tracing::debug!("latest height not more than the chain head delay");
        return Ok(());
    }

    let ending_height = parent_chain_head_height - config.chain_head_delay;

    tracing::debug!(
        "last recorded height: {}, parent chain head: {}, ending_height: {}",
        last_recorded_height,
        parent_chain_head_height,
        ending_height
    );

    if last_recorded_height == ending_height {
        tracing::debug!(
            "the parent has yet to produce a new block, stops at height: {last_recorded_height}"
        );
        return Ok(());
    }

    // we are going backwards in terms of block height, the latest block height is lower
    // than our previously fetched head. It could be a chain reorg. We clear all the cache
    // in `provider` and start from scratch
    if last_recorded_height > ending_height {
        tracing::warn!(
            "last recorded height: {last_recorded_height} more than ending height: {ending_height}"
        );
        return reset_cache(parent_proxy, provider, query).await;
    }

    // we are adding 1 to the height because we are fetching block by block, we also configured
    // the sequential cache to use increment == 1.
    let starting_height = last_recorded_height + 1;
    let ending_height = min(ending_height, MAX_PARENT_VIEW_BLOCK_GAP + starting_height);
    tracing::debug!("parent view range: {starting_height}-{ending_height}");

    let new_parent_views = match get_new_parent_views(
        parent_proxy,
        last_height_hash,
        starting_height,
        ending_height,
    )
    .await
    {
        Ok(views) => views,
        Err(Error::ParentChainReorgDetected) => {
            return reset_cache(parent_proxy, provider, query).await;
        }
        Err(Error::CannotQueryParent(e)) => return Err(anyhow!(e)),
        _ => unreachable!(),
    };
    tracing::debug!("new parent views: {new_parent_views:?}");

    atomically_or_err::<_, Error, _>(move || {
        for (height, block_hash, validator_set, messages) in new_parent_views.clone() {
            provider.new_parent_view(height, block_hash, validator_set, messages)?;
        }
        Ok(())
    })
    .await?;

    tracing::debug!("updated new parent views till height: {ending_height}");

    Ok(())
}

/// Reset the cache in the face of a reorg
async fn reset_cache<T: ParentFinalityStateQuery + Send + Sync + 'static>(
    parent_proxy: &Arc<IPCProviderProxy>,
    provider: &Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>,
    query: &Arc<T>,
) -> anyhow::Result<()> {
    let finality = query_starting_finality(query, parent_proxy).await?;
    atomically(|| provider.reset(finality.clone())).await;
    Ok(())
}

/// Obtains the last recorded height from provider cache or from last committed finality height.
async fn last_recorded_height(
    provider: &Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>,
) -> anyhow::Result<Option<(BlockHeight, BlockHash)>> {
    let result = atomically(|| {
        let h = if let Some(h) = provider.latest_height_hash()? {
            Some(h)
        } else if let Some(f) = provider.last_committed_finality()? {
            Some((f.height, f.block_hash))
        } else {
            None
        };
        Ok(h)
    })
    .await;

    Ok(result)
}

/// Obtain the new parent views for the input block height range
async fn get_new_parent_views(
    parent_proxy: &Arc<IPCProviderProxy>,
    mut previous_hash: BlockHash,
    start_height: BlockHeight,
    end_height: BlockHeight,
) -> Result<
    Vec<(
        BlockHeight,
        BlockHash,
        Vec<StakingChangeRequest>,
        Vec<CrossMsg>,
    )>,
    Error,
> {
    let mut block_height_to_update = vec![];
    let mut total_top_down_msgs = 0;

    for h in start_height..=end_height {
        let block_hash_res = parent_proxy
            .get_block_hash(h)
            .await
            .map_err(|e| Error::CannotQueryParent(e.to_string()))?;
        if block_hash_res.parent_block_hash != previous_hash {
            tracing::warn!(
                "parent block hash at {h} is {:02x?} diff than previous hash: {previous_hash:02x?}",
                block_hash_res.parent_block_hash
            );
            return Err(Error::ParentChainReorgDetected);
        }

        let changes_res = parent_proxy
            .get_validator_changes(h)
            .await
            .map_err(|e| Error::CannotQueryParent(e.to_string()))?;
        if changes_res.block_hash != block_hash_res.block_hash {
            tracing::warn!(
                "change set block hash at {h} is {:02x?} diff than hash: {:02x?}",
                block_hash_res.parent_block_hash,
                block_hash_res.block_hash
            );
            return Err(Error::ParentChainReorgDetected);
        }

        // for `lotus`, the state at height h is only finalized at h + 1. The block hash
        // at height h will return empty top down messages. In this case, we need to get
        // the block hash at height h + 1 to query the top down messages.
        let next_hash = parent_proxy
            .get_block_hash(h + 1)
            .await
            .map_err(|e| Error::CannotQueryParent(e.to_string()))?;
        if next_hash.parent_block_hash != block_hash_res.block_hash {
            tracing::warn!(
                "next block hash at {} is {:02x?} diff than hash: {:02x?}",
                h + 1,
                next_hash.parent_block_hash,
                block_hash_res.block_hash
            );
            return Err(Error::ParentChainReorgDetected);
        }
        let top_down_msgs_res = parent_proxy
            .get_top_down_msgs_with_hash(h, &next_hash.block_hash)
            .await
            .map_err(|e| Error::CannotQueryParent(e.to_string()))?;

        total_top_down_msgs += top_down_msgs_res.len();

        previous_hash = block_hash_res.block_hash.clone();

        block_height_to_update.push((
            h,
            block_hash_res.block_hash,
            changes_res.value,
            top_down_msgs_res,
        ));
        if total_top_down_msgs >= TOPDOWN_MSG_LEN_THRESHOLD {
            break;
        }
    }

    tracing::debug!("obtained updates: {block_height_to_update:?}");

    Ok(block_height_to_update)
}
