// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod error;
pub mod sync;

pub mod proxy;

pub mod cache;
pub mod observe;

use crate::cache::{ParentViewPayload, TopdownViewContainer};
use async_trait::async_trait;
use ethers::utils::hex;
use fvm_shared::address::Address;
use fvm_shared::chainid::ChainID;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Duration;
use sync::syncer::LotusParentSyncer;
use sync::tendermint::TendermintAwareSyncer;
use tokio::sync::Mutex;

pub use crate::error::Error;
use crate::proxy::ParentQueryProxy;
use crate::sync::FendermintStateQuery;

pub type BlockHeight = u64;
pub type Bytes = Vec<u8>;
pub type BlockHash = Bytes;

/// The null round error message
pub(crate) const NULL_ROUND_ERR_MSG: &str = "requested epoch was a null round";
pub(crate) const DEFAULT_MAX_CACHE_BLOCK: BlockHeight = 500;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// The number of blocks to delay before reporting a height as final on the parent chain.
    /// To propose a certain number of epochs delayed from the latest height, we see to be
    /// conservative and avoid other from rejecting the proposal because they don't see the
    /// height as final yet.
    pub chain_head_delay: BlockHeight,
    /// Parent syncing cron period, in seconds
    pub polling_interval: Duration,
    /// Parent voting cron period, in seconds
    pub vote_interval: Duration,
    /// Max number of blocks that should be stored in cache
    pub max_cache_blocks: Option<BlockHeight>,
}

impl Config {
    pub fn new(
        chain_head_delay: BlockHeight,
        polling_interval: Duration,
        vote_interval: Duration,
    ) -> Self {
        Self {
            chain_head_delay,
            polling_interval,
            vote_interval,
            max_cache_blocks: None,
        }
    }

    pub fn with_max_cache_blocks(&mut self, blocks: BlockHeight) {
        self.max_cache_blocks = Some(blocks);
    }

    pub fn max_cache_blocks(&self) -> BlockHeight {
        self.max_cache_blocks.unwrap_or(DEFAULT_MAX_CACHE_BLOCK)
    }
}

/// The finality view for IPC parent at certain height.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParentState {
    /// The latest chain height
    pub height: BlockHeight,
    /// The block hash. For FVM, it is a Cid. For Evm, it is bytes32 as one can now potentially
    /// deploy a subnet on EVM.
    pub block_hash: BlockHash,
}

impl ParentState {
    pub fn new(height: ChainEpoch, hash: BlockHash) -> Self {
        Self {
            height: height as BlockHeight,
            block_hash: hash,
        }
    }
}

impl Display for ParentState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IPCParentFinality(height: {}, block_hash: {})",
            self.height,
            hex::encode(&self.block_hash)
        )
    }
}

/// checks if the error is a filecoin null round error
pub(crate) fn is_null_round_str(s: &str) -> bool {
    s.contains(NULL_ROUND_ERR_MSG)
}

/// Start the parent finality listener in the background
pub async fn run_topdown_voting<T, C, P, V>(
    validator: Address,
    config: Config,
    query: Arc<T>,
    parent_proxy: Arc<P>,
    tendermint_client: C,
    topdown_voter: V,
) where
    T: FendermintStateQuery + Send + Sync + 'static,
    C: tendermint_rpc::Client + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
    V: TopdownVoter + Send + Sync + 'static,
{
    let mut sync_interval = tokio::time::interval(config.polling_interval);
    sync_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut vote_interval = tokio::time::interval(config.vote_interval);
    vote_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let chain_id = loop {
        match query.get_chain_id() {
            Ok(chain_id) => break chain_id,
            Err(e) => {
                tracing::info!("app not up yet: {e}, sleep and retry");
                vote_interval.tick().await;
            }
        }
    };
    let latest_committed = query
        .get_latest_topdown_parent_state()
        .expect("app is up but state not available")
        .expect("latest committed parent state should be available, but non");
    let topdown_data_container = Arc::new(Mutex::new(TopdownViewContainer::new(latest_committed)));

    let lotus_syncer = LotusParentSyncer::new(config, parent_proxy.clone(), topdown_data_container)
        .expect("cannot create lotus parent syncer");
    let tendermint_syncer = Arc::new(TendermintAwareSyncer::new(lotus_syncer, tendermint_client));

    let syncer = tendermint_syncer.clone();
    tokio::spawn(async move {
        loop {
            sync_interval.tick().await;

            if let Err(e) = syncer.sync().await {
                tracing::error!(error = e.to_string(), "sync with parent encountered error");
                continue;
            }
        }
    });

    // setup voting loop
    tokio::spawn(async move {
        loop {
            vote_interval.tick().await;

            if let Err(e) = voting(
                &validator,
                &tendermint_syncer,
                &topdown_voter,
                &query,
                &parent_proxy,
                chain_id,
            )
            .await
            {
                tracing::error!(error = e.to_string(), "sync with parent encountered error");
                continue;
            }
        }
    });
}

async fn voting<T, C, P, V>(
    validator: &Address,
    syncer: &Arc<TendermintAwareSyncer<C, P>>,
    voter: &V,
    query: &Arc<T>,
    parent_proxy: &Arc<P>,
    chain_id: ChainID,
) -> anyhow::Result<()>
where
    T: FendermintStateQuery + Send + Sync + 'static,
    C: tendermint_rpc::Client + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
    V: TopdownVoter + Send + Sync + 'static,
{
    let finalized_checkpoint = sync::query_parent_state(query, parent_proxy).await?;
    syncer.set_committed(finalized_checkpoint).await;

    if query.has_voted(validator)? {
        tracing::debug!(
            validator = validator.to_string(),
            "validator has voted, skip this time"
        );
        return Ok(());
    }

    let Some((h, payload)) = syncer.fetched_first_non_null_block().await else {
        tracing::debug!("topdown syncer not fetched new data");
        return Ok(());
    };
    if payload.1.is_empty() && payload.0.is_empty() {
        tracing::debug!(
            height = h,
            "no topdown messages nor validator changes, skip"
        );
        return Ok(());
    }
    voter.vote(chain_id, h, payload).await
}

/// A trait that will be called by validators to
#[async_trait]
pub trait TopdownVoter {
    async fn vote(
        &self,
        chain_id: ChainID,
        height: observe::BlockHeight,
        parent_view_payload: ParentViewPayload,
    ) -> anyhow::Result<()>;
}
