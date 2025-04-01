// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! A constant running process that fetch or listener to parent state

mod syncer;
mod tendermint;

use crate::proxy::ParentQueryProxy;
use crate::sync::syncer::LotusParentSyncer;
use crate::sync::tendermint::TendermintAwareSyncer;
use crate::{Config, ParentState};
use ethers::utils::hex;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use async_trait::async_trait;
use fvm_shared::chainid::ChainID;

use crate::finality::{ParentViewPayload, TopdownData};
pub use syncer::fetch_topdown_events;
use crate::observe::BlockHeight;

/// Query the parent finality from the child block chain state.
///
/// It returns `None` from queries until the ledger has been initialized.
pub trait ParentFinalityStateQuery {
    /// Get the latest committed finality from the state
    fn get_latest_topdown_parent_state(&self) -> anyhow::Result<Option<ParentState>>;

    fn get_chain_id(&self) -> anyhow::Result<ChainID>;
}

/// Queries the starting finality for polling. First checks the committed finality, if none, that
/// means the chain has just started, then query from the parent to get the genesis epoch.
async fn query_starting_finality<T, P>(
    query: &Arc<T>,
    parent_client: &Arc<P>,
) -> anyhow::Result<ParentState>
where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
{
    loop {
        let mut finality = match query.get_latest_topdown_parent_state() {
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

            finality = ParentState {
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

/// Start the parent finality listener in the background
pub async fn run_topdown_voting<T, C, P, V>(
    config: Config,
    query: Arc<T>,
    parent_proxy: Arc<P>,
    tendermint_client: C,
    topdown_voter: V,
) where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
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
    let latest_committed = query.get_latest_topdown_parent_state()
        .expect("app is up but state not available")
        .expect("latest committed parent state should be available, but non");
    let topdown_data_container =  Arc::new(Mutex::new(TopdownData::new(latest_committed)));

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

            if let Err(e) = voting(&tendermint_syncer, &topdown_voter, &query, &parent_proxy, chain_id).await {
                tracing::error!(error = e.to_string(), "sync with parent encountered error");
                continue;
            }
        }
    });
}

async fn voting<T, C, P, V>(
    syncer: &Arc<TendermintAwareSyncer<C, P>>,
    voter: &V,
    query: &Arc<T>,
    parent_proxy: &Arc<P>,
    chain_id: ChainID,
) -> anyhow::Result<()>
where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    C: tendermint_rpc::Client + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
    V: TopdownVoter + Send + Sync + 'static,
{
    let finalized_checkpoint = query_starting_finality(query, parent_proxy).await?;
    syncer.set_committed(finalized_checkpoint).await;

    let latest_height = syncer.latest_height().await;
    let Some((h, block)) = syncer.get_vote_below_height(latest_height).await else {
        tracing::debug!("topdown syncer not fetched new data");
        return Ok(());
    };
    voter.vote(chain_id, h, block).await
}

#[async_trait]
pub trait TopdownVoter {
    async fn vote(
        &self,
        chain_id: ChainID,
        height: BlockHeight,
        parent_view_payload: ParentViewPayload,
    ) -> anyhow::Result<()>;
}
