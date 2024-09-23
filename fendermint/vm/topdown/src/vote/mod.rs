// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod error;
pub mod gossip;
mod operation;
pub mod payload;
pub mod store;
mod tally;

use crate::sync::TopDownSyncEvent;
use crate::vote::gossip::GossipClient;
use crate::vote::operation::{OperationMetrics, OperationStateMachine};
use crate::vote::payload::{PowerUpdates, Vote};
use crate::vote::store::VoteStore;
use crate::vote::tally::VoteTally;
use crate::BlockHeight;
use error::Error;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot};

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

/// The client to interact with the vote reactor
pub struct VoteReactorClient {
    tx: mpsc::Sender<VoteReactorRequest>,
}

pub fn start_vote_reactor<G: GossipClient + Send+ Sync + 'static, V: VoteStore + Send + Sync + 'static>(
    config: Config,
    power_table: PowerUpdates,
    last_finalized_height: BlockHeight,
    gossip: G,
    vote_store: V,
    internal_event_listener: broadcast::Receiver<TopDownSyncEvent>,
) -> anyhow::Result<VoteReactorClient> {
    let (tx, rx) = mpsc::channel(config.req_channel_buffer_size);
    let vote_tally = VoteTally::new(power_table, last_finalized_height, vote_store)?;

    tokio::spawn(async move {
        let sleep = Duration::new(config.voting_sleep_interval_sec, 0);

        let inner = VotingHandler {
            req_rx: rx,
            internal_event_listener,
            vote_tally,
            config,
            gossip,
        };
        let mut machine = OperationStateMachine::new(inner);
        loop {
            machine = machine.step().await;
            tokio::time::sleep(sleep).await;
        }
    });

    Ok(VoteReactorClient { tx })
}

impl VoteReactorClient {
    pub async fn query_operation_mode(&self) -> anyhow::Result<OperationMetrics> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(VoteReactorRequest::QueryOperationMode(tx))
            .await?;
        Ok(rx.await?)
    }

    pub async fn query_votes(
        &self,
        height: BlockHeight,
    ) -> anyhow::Result<Result<Vec<Vote>, Error>> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(VoteReactorRequest::QueryVotes {
                height,
                reply_tx: tx,
            })
            .await?;
        Ok(rx.await?)
    }

    pub async fn update_power_table(&self, updates: PowerUpdates) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(VoteReactorRequest::UpdatePowerTable {
                updates,
                reply_tx: tx,
            })
            .await?;
        Ok(rx.await?)
    }
}

enum VoteReactorRequest {
    QueryOperationMode(oneshot::Sender<OperationMetrics>),
    QueryVotes {
        height: BlockHeight,
        reply_tx: oneshot::Sender<Result<Vec<Vote>, Error>>,
    },
    UpdatePowerTable {
        updates: PowerUpdates,
        reply_tx: oneshot::Sender<()>,
    },
}

struct VotingHandler<Gossip, VoteStore> {
    /// Handles the requests targeting the vote reactor, could be querying the
    /// vote tally status and etc.
    req_rx: mpsc::Receiver<VoteReactorRequest>,
    /// Interface to gossip pub/sub for topdown voting
    gossip: Gossip,
    /// Listens to internal events and handles the events accordingly
    internal_event_listener: broadcast::Receiver<TopDownSyncEvent>,
    vote_tally: VoteTally<VoteStore>,

    config: Config,
}

impl<G, V> VotingHandler<G, V>
where
    G: GossipClient + Send + Sync + 'static,
    V: VoteStore + Send + Sync + 'static,
{
    fn handle_request(&mut self, req: VoteReactorRequest, metrics: &OperationMetrics) {
        match req {
            VoteReactorRequest::QueryOperationMode(req_tx) => {
                // ignore error
                let _ = req_tx.send(metrics.clone());
            }
            VoteReactorRequest::QueryVotes { height, reply_tx } => {
                let _ = reply_tx.send(self.vote_tally.get_votes_at_height(height));
            }
            VoteReactorRequest::UpdatePowerTable { updates, reply_tx } => {
                self.vote_tally.update_power_table(updates);
                let _ = reply_tx.send(());
            }
        }
    }

    async fn handle_event(&mut self, event: TopDownSyncEvent) {
        match event {
            TopDownSyncEvent::NewProposal(vote) => {
                if let Err(e) = self.vote_tally.add_vote(*vote.clone()) {
                    tracing::error!(err = e.to_string(), "cannot self vote to tally");
                    return;
                }

                match self.gossip.publish_vote(*vote).await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!(
                            err = e.to_string(),
                            "cannot send to gossip sender, tx dropped"
                        );

                        // when this happens, we still keep the vote tally going as
                        // we can still receive other peers's votes.
                    }
                }
            }
            _ => {
                // ignore events we are not interested in
            }
        };
    }

    /// Process external request, such as RPC queries for debugging and status tracking.
    fn process_external_request(&mut self, metrics: &OperationMetrics) -> usize {
        let mut n = 0;
        while n < self.config.req_batch_processing_size {
            match self.req_rx.try_recv() {
                Ok(req) => {
                    self.handle_request(req, metrics);
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
        let mut vote_processed = 0;
        while vote_processed < self.config.gossip_req_processing_size {
            match self.gossip.try_poll_vote() {
                Ok(Some(vote)) => {
                    if let Err(e) = self.vote_tally.add_vote(vote) {
                        tracing::error!(err = e.to_string(), "cannot add vote to tally");
                    } else {
                        vote_processed += 1;
                    }
                }
                Err(e) => {
                    tracing::warn!(err = e.to_string(), "cannot poll gossip vote");
                    break;
                },
                _ => {}
            }
        }
        vote_processed
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
