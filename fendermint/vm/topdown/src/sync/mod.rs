// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! A constant running process that fetch or listener to parent state

mod syncer;
mod tendermint;
mod voter;

use crate::proxy::ParentQueryProxy;
use crate::sync::syncer::LotusParentSyncer;
use crate::sync::tendermint::TendermintAwareSyncer;
use crate::{Config, IPCParentFinality};
use anyhow::anyhow;
use ethers::utils::hex;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub use syncer::fetch_topdown_events;
use crate::finality::{FinalityWithNull, ParentViewPayload};
use crate::sync::voter::TopdownVoter;

/// Query the parent finality from the child block chain state.
///
/// It returns `None` from queries until the ledger has been initialized.
pub trait ParentFinalityStateQuery {
    /// Get the latest committed finality from the state
    fn get_latest_committed_finality(&self) -> anyhow::Result<Option<IPCParentFinality>>;
}

/// Queries the starting finality for polling. First checks the committed finality, if none, that
/// means the chain has just started, then query from the parent to get the genesis epoch.
async fn query_starting_finality<T, P>(
    query: &Arc<T>,
    parent_client: &Arc<P>,
) -> anyhow::Result<IPCParentFinality>
where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
{
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

/// Start the parent finality listener in the background
pub fn launch_topdown_process<T, C, P>(
    config: Config,
    view_provider: Arc<Mutex<FinalityWithNull>>,
    parent_proxy: Arc<P>,
    query: Arc<T>,
    tendermint_client: C,
) where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    C: tendermint_rpc::Client + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
{
    let mut sync_interval = tokio::time::interval(config.polling_interval);
    sync_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut vote_interval = tokio::time::interval(config.vote_interval);
    vote_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let lotus_syncer =
        LotusParentSyncer::new(config, parent_proxy, view_provider, query)
            .expect("cannot create lotus parent syncer");
    let tendermint_syncer = Arc::new(TendermintAwareSyncer::new(lotus_syncer, tendermint_client));

    let syncer = tendermint_syncer.clone();
    tokio::spawn(async move {
        loop {
            sync_interval.tick().await;

            if let Err(e) =  syncer.sync().await {
                tracing::error!(error = e.to_string(), "sync with parent encountered error");
                continue;
            }
        }

    });

    let voter = TopdownVoter{};

    // setup voting loop
    tokio::spawn(async move {
        loop {
            vote_interval.tick().await;

            if let Err(e) = voting(&tendermint_syncer, &voter).await {
                tracing::error!(error = e.to_string(), "sync with parent encountered error");
                continue;
            }
        }

    });
}

async fn voting<T, C, P>(
    syncer: &Arc<TendermintAwareSyncer<T, C, P>>,
    voter: &TopdownVoter,
) -> anyhow::Result<()> where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    C: tendermint_rpc::Client + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
{
    let finalized_checkpoint = voter.latest_finalized_checkpoint().await?;
    syncer.set_committed(finalized_checkpoint).await;

    let voting_liveness_period = voter.livesness_period()?;

    let Some(block) = syncer.get_vote_below_height(voting_liveness_period).await else {
        tracing::debug!("topdown syncer not fetched new data");
        return try_vote_for_liveness(syncer, voter).await;
    };
    vote_with_payload(syncer, voter, block).await
}

async fn try_vote_for_liveness<T, C, P>(
    syncer: &Arc<TendermintAwareSyncer<T, C, P>>,
    voter: &TopdownVoter,
) -> anyhow::Result<()> where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    C: tendermint_rpc::Client + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
{
    todo!()
}

async fn vote_with_payload<T, C, P>(
    syncer: &Arc<TendermintAwareSyncer<T, C, P>>,
    voter: &TopdownVoter,
    payload: ParentViewPayload,
) -> anyhow::Result<()> where
    T: ParentFinalityStateQuery + Send + Sync + 'static,
    C: tendermint_rpc::Client + Send + Sync + 'static,
    P: ParentQueryProxy + Send + Sync + 'static,
{
    todo!()
}