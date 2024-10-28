// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::observation::{Observation, ObservationConfig};
use crate::syncer::payload::ParentBlockView;
use crate::{BlockHeight, Checkpoint};
use anyhow::anyhow;
use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;
use tokio::select;
use tokio::sync::{broadcast, mpsc, oneshot};

pub mod error;
pub mod payload;
pub mod poll;
pub mod store;

#[derive(Clone, Debug)]
pub enum TopDownSyncEvent {
    /// The fendermint node is syncing with peers
    NodeSyncing,
    NewProposal(Box<Observation>),
}

#[derive(Debug, Clone, Deserialize)]
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

impl ParentSyncerReactorClient {
    pub fn new(request_channel_size: usize) -> (Self, mpsc::Receiver<ParentSyncerRequest>) {
        let (tx, rx) = mpsc::channel(request_channel_size);
        (Self { tx }, rx)
    }

    pub fn start_reactor<P: ParentPoller + Send + Sync + 'static>(
        mut rx: mpsc::Receiver<ParentSyncerRequest>,
        mut poller: P,
        config: ParentSyncerConfig,
    ) {
        tokio::spawn(async move {
            let polling_interval = config.polling_interval;

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
    }
}

/// Polls the parent block view
#[async_trait]
pub trait ParentPoller {
    fn subscribe(&self) -> broadcast::Receiver<TopDownSyncEvent>;

    /// The previous checkpoint committed
    fn last_checkpoint(&self) -> &Checkpoint;

    /// The target block height is finalized, purge all the parent view before the target height
    fn finalize(&mut self, checkpoint: Checkpoint) -> anyhow::Result<()>;

    /// Try to poll the next parent height
    async fn try_poll(&mut self) -> anyhow::Result<()>;

    /// Dump the parent block view from the height after the last committed checkpoint to the `to` height
    fn dump_parent_block_views(
        &self,
        to: BlockHeight,
    ) -> anyhow::Result<Vec<Option<ParentBlockView>>>;
}

impl ParentSyncerReactorClient {
    /// Marks the height as finalized.
    /// There is no need to wait for ack from the reactor
    pub async fn finalize_parent_height(&self, cp: Checkpoint) -> anyhow::Result<()> {
        self.tx.send(ParentSyncerRequest::Finalized(cp)).await?;
        Ok(())
    }

    pub async fn query_parent_block_view(
        &self,
        to: BlockHeight,
    ) -> anyhow::Result<anyhow::Result<Vec<Option<ParentBlockView>>>> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ParentSyncerRequest::QueryParentBlockViews { to, tx })
            .await?;
        Ok(rx.await?)
    }
}

pub enum ParentSyncerRequest {
    /// A new parent height is finalized
    Finalized(Checkpoint),
    QueryParentBlockViews {
        to: BlockHeight,
        tx: oneshot::Sender<anyhow::Result<Vec<Option<ParentBlockView>>>>,
    },
}

fn handle_request<P: Send + Sync + 'static + ParentPoller>(
    req: ParentSyncerRequest,
    poller: &mut P,
) {
    match req {
        ParentSyncerRequest::Finalized(c) => {
            let height = c.target_height();
            if let Err(e) = poller.finalize(c) {
                tracing::error!(height, err = e.to_string(), "cannot finalize parent viewer");
            }
        }
        ParentSyncerRequest::QueryParentBlockViews { to, tx } => {
            let r = poller.dump_parent_block_views(to).map_err(|e| {
                tracing::error!(
                    height = to,
                    err = e.to_string(),
                    "cannot query parent block view"
                );
                anyhow!("cannot read parent block view: {}", e)
            });
            let _ = tx.send(r);
        }
    }
}
