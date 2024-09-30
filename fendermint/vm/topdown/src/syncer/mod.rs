// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::observation::{ObservationCommitment, ObservationConfig};
use crate::proxy::ParentQueryProxy;
use crate::syncer::poll::ParentPoll;
use crate::syncer::store::ParentViewStore;
use crate::{BlockHeight, Checkpoint};
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc;

pub mod error;
pub mod payload;
pub mod poll;
pub mod store;

#[derive(Clone, Debug)]
pub enum TopDownSyncEvent {
    /// The fendermint node is syncing with peers
    NodeSyncing,
    NewProposal(Box<ObservationCommitment>),
}

pub struct ParentSyncerConfig {
    pub request_channel_size: usize,
    /// The event broadcast channel buffer size
    pub broadcast_channel_size: usize,
    /// The number of blocks to delay before reporting a height as final on the parent chain.
    /// To propose a certain number of epochs delayed from the latest height, we see to be
    /// conservative and avoid other from rejecting the proposal because they don't see the
    /// height as final yet.
    pub chain_head_delay: BlockHeight,
    /// Parent syncing cron period, in seconds
    pub polling_interval: Duration,
    /// Top down exponential back off retry base
    pub exponential_back_off: Duration,
    /// The max number of retries for exponential backoff before giving up
    pub exponential_retry_limit: usize,
    /// Max number of un-finalized parent blocks that should be stored in the store
    pub max_store_blocks: BlockHeight,
    /// Attempts to sync as many block as possible till the finalized chain head
    pub sync_many: bool,

    pub observation: ObservationConfig,
}

#[derive(Clone)]
pub struct ParentSyncerReactorClient {
    tx: mpsc::Sender<ParentSyncerRequest>,
}

pub fn start_parent_syncer<P, S>(
    config: ParentSyncerConfig,
    proxy: P,
    store: S,
    last_finalized: Checkpoint,
) -> anyhow::Result<ParentSyncerReactorClient>
where
    S: ParentViewStore + Send + Sync + 'static,
    P: Send + Sync + 'static + ParentQueryProxy,
{
    let (tx, mut rx) = mpsc::channel(config.request_channel_size);

    tokio::spawn(async move {
        let polling_interval = config.polling_interval;
        let mut poller = ParentPoll::new(config, proxy, store, last_finalized);

        loop {
            select! {
                _ = tokio::time::sleep(polling_interval) => {
                    if let Err(e) = poller.try_poll().await {
                        tracing::error!(err = e.to_string(), "cannot sync with parent");
                    }
                }
                req = rx.recv() => {
                    let Some(req) = req else { break };
                    handle_request(req, &mut poller);
                }
            }
        }

        tracing::warn!("parent syncer stopped")
    });
    Ok(ParentSyncerReactorClient { tx })
}

impl ParentSyncerReactorClient {
    /// Marks the height as finalized.
    /// There is no need to wait for ack from the reactor
    pub async fn finalize_parent_height(&self, cp: Checkpoint) -> anyhow::Result<()> {
        self.tx.send(ParentSyncerRequest::Finalized(cp)).await?;
        Ok(())
    }
}

enum ParentSyncerRequest {
    /// A new parent height is finalized
    Finalized(Checkpoint),
}

fn handle_request<P, S>(req: ParentSyncerRequest, poller: &mut ParentPoll<P, S>)
where
    S: ParentViewStore + Send + Sync + 'static,
    P: Send + Sync + 'static + ParentQueryProxy,
{
    match req {
        ParentSyncerRequest::Finalized(c) => {
            let height = c.target_height();
            if let Err(e) = poller.finalize(c) {
                tracing::error!(height, err = e.to_string(), "cannot finalize parent viewer");
            }
        }
    }
}
