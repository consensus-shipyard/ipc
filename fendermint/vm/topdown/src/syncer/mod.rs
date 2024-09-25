// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::time::Duration;
use tokio::sync::mpsc;
use crate::BlockHeight;
use crate::vote::payload::Observation;

pub mod payload;
pub mod reactor;
pub mod store;
pub mod error;

#[derive(Clone, Debug)]
pub enum TopDownSyncEvent {
    /// The fendermint node is syncing with peers
    NodeSyncing,
    /// The parent view store is full, this will pause the parent syncer
    ParentViewStoreFull,
    NewProposal(Box<Observation>),
}

pub struct ParentSyncerConfig {
    pub request_channel_size: usize,
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
}

pub struct ParentSyncerReactorClient {
    tx: mpsc::Sender<()>,

}

pub fn start_parent_syncer(config: ParentSyncerConfig) -> anyhow::Result<ParentSyncerReactorClient> {
    let (tx, rx) = mpsc::channel(config.request_channel_size);

    tokio::spawn(async move {
    });
    Ok(ParentSyncerReactorClient { tx })
}
