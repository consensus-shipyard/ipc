// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod operation;
mod payload;
mod store;
mod tally;

use crate::sync::TopDownSyncEvent;
use crate::vote::operation::{OperationMetrics, OperationStateMachine};
use crate::vote::payload::Vote;
use crate::BlockHeight;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};

pub type Weight = u64;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// The reactor request channel buffer size
    req_channel_buffer_size: usize,
    /// The number of requests the reactor should process per run before handling other tasks
    req_batch_processing_size: usize,
    /// The number of vote recording requests the reactor should process per run before handling other tasks
    gossip_req_processing_size: usize,
    /// The time to sleep for voting loop if nothing happens
    voting_sleep_interval_sec: u64,
}

pub struct VoteReactorClient {
    tx: mpsc::Sender<VoteReactorRequest>,
}

pub fn start_vote_reactor(
    config: Config,
    gossip_rx: broadcast::Receiver<Vote>,
    gossip_tx: mpsc::Sender<Vote>,
    internal_event_listener: broadcast::Receiver<TopDownSyncEvent>,
) -> VoteReactorClient {
    let (tx, rx) = mpsc::channel(config.req_channel_buffer_size);

    tokio::spawn(async move {
        let sleep = Duration::new(config.voting_sleep_interval_sec, 0);

        let inner = VotingHandler {
            req_rx: rx,
            gossip_rx,
            gossip_tx,
            internal_event_listener,
            config,
        };
        let mut machine = OperationStateMachine::new(inner);
        loop {
            machine = machine.step();
            tokio::time::sleep(sleep).await;
        }
    });

    VoteReactorClient { tx }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("the last finalized block has not been set")]
    Uninitialized,

    #[error("failed to extend chain; height going backwards, current height {0}, got {1}")]
    UnexpectedBlock(BlockHeight, BlockHeight),

    #[error("validator unknown or has no power")]
    UnpoweredValidator,

    #[error("equivocation by validator")]
    Equivocation,

    #[error("validator vote is invalidated")]
    VoteCannotBeValidated,

    #[error("validator cannot sign vote")]
    CannotSignVote,
}

enum VoteReactorRequest {
    QueryOperationMode,
    QueryVotes(BlockHeight),
}

struct VotingHandler {
    /// Handles the requests targeting the vote reactor, could be querying the
    /// vote tally status and etc.
    req_rx: mpsc::Receiver<VoteReactorRequest>,
    /// Receiver from gossip pub/sub, mostly listening to incoming votes
    gossip_rx: broadcast::Receiver<Vote>,
    gossip_tx: mpsc::Sender<Vote>,
    /// Listens to internal events and handles the events accordingly
    internal_event_listener: broadcast::Receiver<TopDownSyncEvent>,
    config: Config,
}

impl VotingHandler {
    fn handle_request(&self, _req: VoteReactorRequest) {}

    fn record_vote(&self, _vote: Vote) {}

    fn handle_event(&self, _event: TopDownSyncEvent) {}

    /// Process external request, such as RPC queries for debugging and status tracking.
    fn process_external_request(&mut self, _metrics: &OperationMetrics) -> usize {
        let mut n = 0;
        while n < self.config.req_batch_processing_size {
            match self.req_rx.try_recv() {
                Ok(req) => {
                    self.handle_request(req);
                    n += 1
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    tracing::warn!("voting reactor tx closed unexpected");
                    break;
                }
                Err(mpsc::error::TryRecvError::Empty) => break,
            }
        }
        n
    }

    /// Handles vote tally gossip pab/sub incoming votes from other peers
    fn process_gossip_subscription_votes(&mut self) -> usize {
        let mut n = 0;
        while n < self.config.gossip_req_processing_size {
            match self.gossip_rx.try_recv() {
                Ok(vote) => {
                    self.record_vote(vote);
                    n += 1;
                }
                Err(broadcast::error::TryRecvError::Empty) => break,
                _ => {
                    tracing::warn!("gossip sender lagging or closed");
                    break;
                }
            }
        }
        n
    }

    /// Poll internal topdown syncer event broadcasted.
    fn poll_internal_event(&mut self) -> Option<TopDownSyncEvent> {
        match self.internal_event_listener.try_recv() {
            Ok(event) => Some(event),
            Err(broadcast::error::TryRecvError::Empty) => None,
            _ => {
                tracing::warn!("gossip sender lagging or closed");
                None
            }
        }
    }
}
