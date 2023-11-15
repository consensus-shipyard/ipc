// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! A constant running process that fetch or listener to parent state

use crate::error::Error;
use crate::finality::ParentViewPayload;
use crate::proxy::{IPCProviderProxy, ParentQueryProxy};
use crate::{
    is_null_round_str, BlockHash, BlockHeight, CachedFinalityProvider, Config, IPCParentFinality,
    ParentFinalityProvider, Toggle,
};

use async_stm::{atomically, atomically_or_err, Stm};
use ipc_provider::manager::GetBlockHashResult;

use anyhow::{anyhow, Context};

use ethers::utils::hex;
use std::cmp::{max, min};
use std::sync::Arc;
use std::time::Duration;

/// The max number of blocks polling should query each parent view update. If the number of blocks
/// polled equals this value, it would stop polling for this iteration and commit the result to cache.
const MAX_PARENT_VIEW_BLOCK_GAP: BlockHeight = 100;
/// When polling parent view, if the number of top down messages exceeds this limit,
/// the polling will stop for this iteration and commit the result to cache.
const TOPDOWN_MSG_LEN_THRESHOLD: usize = 500;

type GetParentViewPayload = Vec<(BlockHeight, Option<ParentViewPayload>)>;

/// Query the parent finality from the block chain state
pub trait ParentFinalityStateQuery {
    /// Get the latest committed finality from the state
    fn get_latest_committed_finality(&self) -> anyhow::Result<Option<IPCParentFinality>>;
}

/// Constantly syncing with parent through polling
struct PollingParentSyncer<T, C> {
    config: Config,
    parent_view_provider: Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>,
    parent_client: Arc<IPCProviderProxy>,
    committed_state_query: Arc<T>,
    tendermint_client: C,
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
                tracing::warn!(error = e.to_string(), "cannot get committed finality");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        tracing::info!(finality = finality.to_string(), "latest finality committed");

        // this means there are no previous committed finality yet, we fetch from parent to get
        // the genesis epoch of the current subnet and its corresponding block hash.
        if finality.height == 0 {
            let genesis_epoch = parent_client.get_genesis_epoch().await?;
            tracing::debug!(genesis_epoch = genesis_epoch, "obtained genesis epoch");
            let r = parent_client.get_block_hash(genesis_epoch).await?;
            tracing::debug!(
                block_hash = hex::encode(&r.block_hash),
                "obtained genesis block hash",
            );

            finality = IPCParentFinality {
                height: genesis_epoch,
                block_hash: r.block_hash,
            };
            tracing::info!(
                genesis_finality = finality.to_string(),
                "no previous finality committed, fetched from genesis epoch"
            );
        }

        return Ok(finality);
    }
}

/// Start the polling parent syncer in the background
pub async fn launch_polling_syncer<T, C>(
    query: T,
    config: Config,
    view_provider: Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>,
    parent_client: Arc<IPCProviderProxy>,
    tendermint_client: C,
) -> anyhow::Result<()>
where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    C: tendermint_rpc::Client + Send + Sync + 'static,
{
    if !view_provider.is_enabled() {
        return Err(anyhow!("provider not enabled, enable to run syncer"));
    }

    let query = Arc::new(query);
    let finality = query_starting_finality(&query, &parent_client).await?;
    atomically(|| view_provider.set_new_finality(finality.clone(), None)).await;

    tracing::info!(
        finality = finality.to_string(),
        "launching parent syncer with last committed finality"
    );

    let poll = PollingParentSyncer::new(
        config,
        view_provider,
        parent_client,
        query,
        tendermint_client,
    );
    poll.start();

    Ok(())
}

impl<T, C> PollingParentSyncer<T, C> {
    pub fn new(
        config: Config,
        parent_view_provider: Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>,
        parent_client: Arc<IPCProviderProxy>,
        query: Arc<T>,
        tendermint_client: C,
    ) -> Self {
        Self {
            config,
            parent_view_provider,
            parent_client,
            committed_state_query: query,
            tendermint_client,
        }
    }
}

impl<T, C> PollingParentSyncer<T, C>
where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    C: tendermint_rpc::Client + Send + Sync + 'static,
{
    /// Start the parent finality listener in the background
    pub fn start(self) {
        let config = self.config;
        let provider = self.parent_view_provider;
        let parent_client = self.parent_client;
        let query = self.committed_state_query;
        let tendermint_client = self.tendermint_client;

        let mut interval = tokio::time::interval(config.polling_interval);

        tokio::spawn(async move {
            loop {
                interval.tick().await;

                if let Err(e) = sync_with_parent(
                    &config,
                    &parent_client,
                    &provider,
                    &query,
                    &tendermint_client,
                )
                .await
                {
                    tracing::error!(error = e.to_string(), "sync with parent encountered error");
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
async fn sync_with_parent<T, C>(
    config: &Config,
    parent_proxy: &Arc<IPCProviderProxy>,
    provider: &Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>,
    query: &Arc<T>,
    tendermint_client: &C,
) -> anyhow::Result<()>
where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    C: tendermint_rpc::Client + Send + Sync + 'static,
{
    let status: tendermint_rpc::endpoint::status::Response = tendermint_client
        .status()
        .await
        .context("failed to get Tendermint status")?;

    if status.sync_info.catching_up {
        tracing::debug!("syncing with peer, skip parent finality syncing this round");
        return Ok(());
    }

    let (last_recorded_height, last_height_hash) =
        if let Some(h) = last_recorded_data(provider).await? {
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

    // we consider the chain head finalized only after the `chain_head_delay`
    let max_ending_height = parent_chain_head_height - config.chain_head_delay;

    tracing::debug!(
        last_recorded_height = last_recorded_height,
        parent_chain_head_height = parent_chain_head_height,
        max_ending_height = max_ending_height,
        "syncing heights",
    );

    if last_recorded_height == max_ending_height {
        tracing::debug!(
            last_recorded_height = last_recorded_height,
            "the parent has yet to produce a new block"
        );
        return Ok(());
    }

    // we are going backwards in terms of block height, the latest block height is lower
    // than our previously fetched head. It could be a chain reorg. We clear all the cache
    // in `provider` and start from scratch
    if last_recorded_height > max_ending_height {
        tracing::warn!(
            last_recorded_height = last_recorded_height,
            max_ending_height = max_ending_height,
            "last recorded height more than max ending height"
        );
        return reset_cache(parent_proxy, provider, query).await;
    }

    // we are adding 1 to the height because we are fetching block by block, we also configured
    // the sequential cache to use increment == 1.
    let starting_height = last_recorded_height + 1;
    let ending_height = min(
        max_ending_height,
        MAX_PARENT_VIEW_BLOCK_GAP + starting_height,
    );
    tracing::debug!(
        start = starting_height,
        end = ending_height,
        "parent view range"
    );

    let new_parent_views = parent_views_in_block_range(
        parent_proxy,
        last_height_hash,
        starting_height,
        ending_height,
        max_ending_height,
    )
    .await?;

    atomically_or_err::<_, Error, _>(move || {
        for (height, maybe_payload) in new_parent_views.clone() {
            provider.new_parent_view(height, maybe_payload)?;
        }
        Ok(())
    })
    .await?;

    tracing::debug!(height = ending_height, "updated new parent views to");

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

/// A util struct that tracks the last recorded height
enum LastRecordedBlock {
    FilledBlock {
        height: BlockHeight,
        hash: BlockHash,
    },
    NullBlock(BlockHeight),
    Empty,
}

impl LastRecordedBlock {
    fn filled(height: BlockHeight, hash: BlockHash) -> Self {
        Self::FilledBlock { height, hash }
    }

    fn null(height: BlockHeight) -> Self {
        Self::NullBlock(height)
    }

    fn empty() -> Self {
        Self::Empty
    }
}

/// Getting the last recorded block height/hash
async fn last_recorded_data(
    provider: &Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>,
) -> anyhow::Result<Option<(BlockHeight, BlockHash)>> {
    match atomically(|| last_recorded_block(provider)).await {
        LastRecordedBlock::Empty => Ok(None),
        LastRecordedBlock::FilledBlock { height, hash } => Ok(Some((height, hash))),
        LastRecordedBlock::NullBlock(height) => {
            tracing::info!(height, "last recorded height is a null block");

            // Imaging the list of blocks as follows:
            //
            // F0  B0  B1  N0  N1  B2  B3  B4
            //
            // where F0 represents the last committed finality, B* represents non-null blocks and
            // N* represents null blocks.
            //
            // Currently the last recorded block is N1, so the next block to sync in parent is B2.
            // The response from getting block hash at height B2 from fvm eth apis would return:
            //
            // Block height: B2, Block hash: hash(B2), Parent block hash: hash(B1)
            //
            // F0  B0  B1  N0  N1  B2
            //     B0' N0' N1' N2' B2  B3  B4       <====== reorged chain case 1
            //     B0  B1  B2' B3' B2               <====== reorged chain case 2
            //     B0  B1  N0' B1' B2               <====== reorged chain case 3
            //
            // If last recorded block is null (say N1), to ensure the chain has not reorg before B2:
            // we just need to get the first non null parent in cache or committed finality and use
            // that block hash as previous block hash in following steps.
            match atomically(|| provider.first_non_null_parent_hash(height)).await {
                None => unreachable!("should have last committed finality at this point"),
                Some(hash) => {
                    tracing::info!(
                        block_height = height,
                        parent_hash = hex::encode(&hash),
                        "First non null parent",
                    );
                    Ok(Some((height, hash)))
                }
            }
        }
    }
}

/// Obtains the last recorded block from provider cache or from last committed finality height.
fn last_recorded_block(
    provider: &Arc<Toggle<CachedFinalityProvider<IPCProviderProxy>>>,
) -> Stm<LastRecordedBlock> {
    let latest_height = if let Some(h) = provider.latest_height()? {
        h
    } else if let Some(f) = provider.last_committed_finality()? {
        // this means provider has cleared cache, but only previous committed finality
        return Ok(LastRecordedBlock::filled(f.height, f.block_hash));
    } else {
        return Ok(LastRecordedBlock::empty());
    };

    if let Some(hash) = provider.block_hash(latest_height)? {
        Ok(LastRecordedBlock::filled(latest_height, hash))
    } else {
        Ok(LastRecordedBlock::null(latest_height))
    }
}

/// Obtain the new parent views for the input block height range
async fn parent_views_in_block_range(
    parent_proxy: &Arc<IPCProviderProxy>,
    mut previous_hash: BlockHash,
    start_height: BlockHeight,
    end_height: BlockHeight,
    max_ending_height: BlockHeight,
) -> Result<GetParentViewPayload, Error> {
    let mut updates = vec![];
    let mut total_top_down_msgs = 0;

    for h in start_height..=end_height {
        match parent_views_at_height(parent_proxy, &previous_hash, h, max_ending_height).await {
            Ok((hash, changeset, cross_msgs)) => {
                total_top_down_msgs += cross_msgs.len();

                tracing::debug!(
                    height = h,
                    previous_previous_hahs = hex::encode(&previous_hash),
                    previous_hash = hex::encode(&hash),
                    "matching hashes",
                );
                previous_hash = hash.clone();

                updates.push((h, Some((hash, changeset, cross_msgs))));
                if total_top_down_msgs >= TOPDOWN_MSG_LEN_THRESHOLD {
                    break;
                }
            }
            // Handles lotus null round error.
            //
            // This is the error that we see when there is a null round:
            // https://github.com/filecoin-project/lotus/blob/7bb1f98ac6f5a6da2cc79afc26d8cd9fe323eb30/node/impl/full/eth.go#L164
            // This happens when we request the block for a round without blocks in the tipset.
            // A null round will never have a block, which means that we can advance to the next height.
            Err(e) => {
                let err_msg = e.to_string();
                if is_null_round_str(&err_msg) {
                    tracing::warn!(height = h, "null round detected, skip");
                    updates.push((h, None));
                } else if let Error::LookAheadLimitReached(start, limit) = e {
                    tracing::warn!(
                        start_height = start,
                        limit_height = limit,
                        "look ahead limit reached, store updates so far in cache",
                    );
                    break;
                } else {
                    return Err(e);
                }
            }
        }
    }

    tracing::debug!(?updates, "obtained parent view updates");

    Ok(updates)
}

/// Obtain the new parent views for the target height.
///
/// For `max_ending_height`, the explanation is as follows:
/// Say the current height is h and we need to fetch the top down messages. For `lotus`, the state
/// at height h is only finalized at h + 1. The block hash at height h will return empty top down
/// messages. In this case, we need to get the block hash at height h + 1 to query the top down messages.
/// Sadly, the height h + 1 could be null block, we need to continuously look ahead until we found
/// a height that is not null. But we cannot go all the way to the block head as it's not considered
/// final yet. So we need to use a `max_ending_height` that restricts how far as head we should go.
/// If we still cannot find a height that is non-null, maybe we should try later
async fn parent_views_at_height(
    parent_proxy: &Arc<IPCProviderProxy>,
    previous_hash: &BlockHash,
    height: BlockHeight,
    max_ending_height: BlockHeight,
) -> Result<ParentViewPayload, Error> {
    let block_hash_res = parent_proxy
        .get_block_hash(height)
        .await
        .map_err(|e| Error::CannotQueryParent(e.to_string(), height))?;
    if block_hash_res.parent_block_hash != *previous_hash {
        tracing::warn!(
            height,
            parent_hash = hex::encode(&block_hash_res.parent_block_hash),
            previous_hash = hex::encode(previous_hash),
            "parent block hash diff than previous hash",
        );
        return Err(Error::ParentChainReorgDetected);
    }

    let changes_res = parent_proxy
        .get_validator_changes(height)
        .await
        .map_err(|e| Error::CannotQueryParent(e.to_string(), height))?;
    if changes_res.block_hash != block_hash_res.block_hash {
        tracing::warn!(
            height,
            change_set_hash = hex::encode(&changes_res.block_hash),
            block_hash = hex::encode(&block_hash_res.block_hash),
            "change set block hash does not equal block hash",
        );
        return Err(Error::ParentChainReorgDetected);
    }

    // for `lotus`, the state at height h is only finalized at h + 1. The block hash
    // at height h will return empty top down messages. In this case, we need to get
    // the block hash at height h + 1 to query the top down messages.
    // Sadly, the height h + 1 could be null block, we need to continuously look ahead
    // until we found a height that is not null
    let next_hash = first_non_null_block_hash(parent_proxy, height + 1, max_ending_height).await?;
    if next_hash.parent_block_hash != block_hash_res.block_hash {
        tracing::warn!(
            next_block_height = height + 1,
            next_block_parent = hex::encode(&next_hash.parent_block_hash),
            block_hash = hex::encode(&block_hash_res.block_hash),
            "next block's parent hash does not equal block hash",
        );
        return Err(Error::ParentChainReorgDetected);
    }
    let top_down_msgs_res = parent_proxy
        .get_top_down_msgs_with_hash(height, &next_hash.block_hash)
        .await
        .map_err(|e| Error::CannotQueryParent(e.to_string(), height))?;

    Ok((
        block_hash_res.block_hash,
        changes_res.value,
        top_down_msgs_res,
    ))
}

/// Get the first non-null block hash in between heights. If height is a null round, then we need
/// to look further util we find one that is not null round.
async fn first_non_null_block_hash(
    parent_proxy: &Arc<IPCProviderProxy>,
    start: BlockHeight,
    mut end: BlockHeight,
) -> Result<GetBlockHashResult, Error> {
    // at least we run for height
    end = max(start, end);

    for h in start..=end {
        match parent_proxy.get_block_hash(h).await {
            Ok(h) => return Ok(h),
            Err(e) => {
                let msg = e.to_string();
                if is_null_round_str(&msg) {
                    tracing::warn!(
                        height = h,
                        error = e.to_string(),
                        "look ahead height is a null round"
                    );
                    continue;
                } else {
                    return Err(Error::CannotQueryParent(msg, h));
                }
            }
        }
    }
    Err(Error::LookAheadLimitReached(start, end))
}
