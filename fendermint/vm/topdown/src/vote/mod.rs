// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod error;
pub mod gossip;
mod operation;
pub mod payload;
pub mod store;
mod tally;

use crate::observation::{CertifiedObservation, Observation};
use crate::syncer::TopDownSyncEvent;
use crate::vote::gossip::GossipClient;
use crate::vote::operation::{OperationMetrics, OperationStateMachine};
use crate::vote::payload::{PowerUpdates, Vote, VoteTallyState};
use crate::vote::store::VoteStore;
use crate::vote::tally::VoteTally;
use crate::BlockHeight;
use anyhow::anyhow;
use error::Error;
use fendermint_crypto::quorum::ECDSACertificate;
use fendermint_crypto::SecretKey;
use fendermint_vm_genesis::ValidatorKey;
use serde::Deserialize;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot};

pub type Weight = u64;

#[derive(Deserialize, Debug, Clone)]
pub struct VoteConfig {
    /// The reactor request channel buffer size
    pub req_channel_buffer_size: usize,
    /// The number of requests the reactor should process per run before handling other tasks
    pub req_batch_processing_size: usize,
    /// The number of vote recording requests the reactor should process per run before handling other tasks
    pub gossip_req_processing_size: usize,
    /// The time to sleep for voting loop if nothing happens
    pub voting_sleep_interval_millis: u64,
}

/// The client to interact with the vote reactor
#[derive(Clone)]
pub struct VoteReactorClient {
    tx: mpsc::Sender<VoteReactorRequest>,
}

impl VoteReactorClient {
    pub fn new(req_channel_buffer_size: usize) -> (Self, mpsc::Receiver<VoteReactorRequest>) {
        let (tx, rx) = mpsc::channel(req_channel_buffer_size);
        (Self { tx }, rx)
    }

    pub fn start_reactor<
        G: GossipClient + Send + Sync + 'static,
        V: VoteStore + Send + Sync + 'static,
    >(
        rx: mpsc::Receiver<VoteReactorRequest>,
        params: StartVoteReactorParams<G, V>,
    ) -> anyhow::Result<()> {
        let config = params.config;
        let vote_tally = VoteTally::new(
            params.power_table,
            params.last_finalized_height,
            params.vote_store,
        )?;

        let validator_key = params.validator_key;
        let internal_event_listener = params.internal_event_listener;
        let latest_child_block = params.latest_child_block;
        let gossip = params.gossip;

        tokio::spawn(async move {
            let sleep = Duration::from_millis(config.voting_sleep_interval_millis);

            let inner = VotingHandler {
                validator_key,
                req_rx: rx,
                internal_event_listener,
                vote_tally,
                latest_child_block,
                config,
                gossip,
            };
            let mut machine = OperationStateMachine::new(inner);
            loop {
                machine = machine.step().await;
                tokio::time::sleep(sleep).await;
            }
        });

        Ok(())
    }
}

pub struct StartVoteReactorParams<G, V> {
    pub config: VoteConfig,
    pub validator_key: SecretKey,
    pub power_table: PowerUpdates,
    pub last_finalized_height: BlockHeight,
    pub latest_child_block: BlockHeight,
    pub gossip: G,
    pub vote_store: V,
    pub internal_event_listener: broadcast::Receiver<TopDownSyncEvent>,
}

impl VoteReactorClient {
    async fn request<T, F: FnOnce(oneshot::Sender<T>) -> VoteReactorRequest>(
        &self,
        f: F,
    ) -> anyhow::Result<T> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(f(tx)).await?;
        let r = rx.await?;
        Ok(r)
    }

    /// Query the current operation mode of the vote tally state machine
    pub async fn query_operation_mode(&self) -> anyhow::Result<OperationMetrics> {
        self.request(VoteReactorRequest::QueryOperationMode).await
    }

    /// Query the current validator votes at the target block height
    pub async fn query_votes(
        &self,
        height: BlockHeight,
    ) -> anyhow::Result<Result<Vec<Vote>, Error>> {
        self.request(|tx| VoteReactorRequest::QueryVotes { height, tx })
            .await
    }

    /// Queries the vote tally to see if there are new quorum formed
    pub async fn find_quorum(&self) -> anyhow::Result<Option<ECDSACertificate<Observation>>> {
        self.request(VoteReactorRequest::FindQuorum).await
    }

    /// Get the current vote tally state variables in vote tally
    pub async fn query_vote_tally_state(&self) -> anyhow::Result<VoteTallyState> {
        self.request(VoteReactorRequest::QueryState).await
    }

    /// Update power of some validators. If the weight is zero, the validator is removed
    /// from the power table.
    pub async fn update_power_table(&self, updates: PowerUpdates) -> anyhow::Result<()> {
        self.request(|tx| VoteReactorRequest::UpdatePowerTable { updates, tx })
            .await
    }

    /// Completely over-write existing power table
    pub async fn set_power_table(&self, updates: PowerUpdates) -> anyhow::Result<()> {
        self.request(|tx| VoteReactorRequest::SetPowerTable { updates, tx })
            .await
    }

    /// Signals that a new quorum is finalized and executed in the interpreter
    pub async fn set_quorum_finalized(
        &self,
        height: BlockHeight,
    ) -> anyhow::Result<Result<(), Error>> {
        self.request(|tx| VoteReactorRequest::SetQuorumFinalized { height, tx })
            .await
    }

    pub async fn dump_votes(
        &self,
    ) -> anyhow::Result<Result<HashMap<BlockHeight, Vec<Vote>>, Error>> {
        self.request(VoteReactorRequest::DumpAllVotes).await
    }

    /// A new child/local block is mined
    pub async fn new_local_block_mined(&self, h: BlockHeight) -> anyhow::Result<()> {
        self.tx
            .send(VoteReactorRequest::NewLocalBlockMined(h))
            .await?;
        Ok(())
    }

    pub async fn check_quorum_cert(
        &self,
        cert: Box<ECDSACertificate<Observation>>,
    ) -> anyhow::Result<()> {
        let is_reached = self
            .request(|tx| VoteReactorRequest::CheckQuorumCert { tx, cert })
            .await?;
        if !is_reached {
            return Err(anyhow!("quorum not reached"));
        }
        Ok(())
    }
}

pub enum VoteReactorRequest {
    /// A new child subnet block is mined, this is the fendermint block
    NewLocalBlockMined(BlockHeight),
    /// Query the current operation mode of the vote tally state machine
    QueryOperationMode(oneshot::Sender<OperationMetrics>),
    /// Query the current validator votes at the target block height
    QueryVotes {
        height: BlockHeight,
        tx: oneshot::Sender<Result<Vec<Vote>, Error>>,
    },
    /// Dump all the votes that is currently stored in the vote tally.
    /// This is generally a very expensive operation, but good for debugging, use with care
    DumpAllVotes(oneshot::Sender<Result<HashMap<BlockHeight, Vec<Vote>>, Error>>),
    /// Get the current vote tally state variables in vote tally
    QueryState(oneshot::Sender<VoteTallyState>),
    CheckQuorumCert {
        cert: Box<ECDSACertificate<Observation>>,
        tx: oneshot::Sender<bool>,
    },
    /// Queries the vote tally to see if there are new quorum formed
    FindQuorum(oneshot::Sender<Option<ECDSACertificate<Observation>>>),
    /// Update power of some validators. If the weight is zero, the validator is removed
    /// from the power table.
    UpdatePowerTable {
        updates: PowerUpdates,
        tx: oneshot::Sender<()>,
    },
    /// Completely over-write existing power table
    SetPowerTable {
        updates: PowerUpdates,
        tx: oneshot::Sender<()>,
    },
    /// Signals that a new quorum is finalized and executed in the interpreter
    SetQuorumFinalized {
        height: BlockHeight,
        tx: oneshot::Sender<Result<(), Error>>,
    },
}

struct VotingHandler<Gossip, VoteStore> {
    /// The validator key that is used to sign proposal produced for broadcasting
    validator_key: SecretKey,
    /// Handles the requests targeting the vote reactor, could be querying the
    /// vote tally status and etc.
    req_rx: mpsc::Receiver<VoteReactorRequest>,
    /// Interface to gossip pub/sub for topdown voting
    gossip: Gossip,
    /// Listens to internal events and handles the events accordingly
    internal_event_listener: broadcast::Receiver<TopDownSyncEvent>,
    vote_tally: VoteTally<VoteStore>,
    latest_child_block: BlockHeight,
    config: VoteConfig,
}

impl<G, V> VotingHandler<G, V>
where
    G: GossipClient + Send + Sync + 'static,
    V: VoteStore + Send + Sync + 'static,
{
    fn handle_request(&mut self, req: VoteReactorRequest, metrics: &OperationMetrics) {
        match req {
            VoteReactorRequest::QueryOperationMode(tx) => {
                // ignore error
                let _ = tx.send(metrics.clone());
            }
            VoteReactorRequest::QueryVotes { height, tx } => {
                let _ = tx.send(self.vote_tally.get_votes_at_height(height));
            }
            VoteReactorRequest::UpdatePowerTable { updates, tx } => {
                self.vote_tally.update_power_table(updates);
                let _ = tx.send(());
            }
            VoteReactorRequest::FindQuorum(tx) => {
                let quorum = self
                    .vote_tally
                    .find_quorum()
                    .inspect_err(|e| tracing::error!(err = e.to_string(), "cannot find quorum"))
                    .unwrap_or_default();
                let _ = tx.send(quorum);
            }
            VoteReactorRequest::SetPowerTable { updates, tx } => {
                self.vote_tally.set_power_table(updates);
                let _ = tx.send(());
            }
            VoteReactorRequest::SetQuorumFinalized { height, tx } => {
                let _ = tx.send(self.vote_tally.set_finalized(height));
            }
            VoteReactorRequest::QueryState(tx) => {
                let _ = tx.send(VoteTallyState {
                    last_finalized_height: self.vote_tally.last_finalized_height(),
                    quorum_threshold: self.vote_tally.quorum_threshold(),
                    power_table: self.vote_tally.power_table().clone(),
                });
            }
            VoteReactorRequest::DumpAllVotes(tx) => {
                let _ = tx.send(self.vote_tally.dump_votes());
            }
            VoteReactorRequest::NewLocalBlockMined(n) => {
                self.latest_child_block = n;
            }
            VoteReactorRequest::CheckQuorumCert { cert, tx } => {
                if !self.vote_tally.check_quorum_cert(cert.borrow()) {
                    let _ = tx.send(false);
                } else {
                    let _ = tx.send(
                        self.vote_tally.last_finalized_height() + 1 == cert.payload().parent_height,
                    );
                }
            }
        }
    }

    async fn handle_event(&mut self, event: TopDownSyncEvent) {
        match event {
            TopDownSyncEvent::NewProposal(observation) => {
                let vote = match CertifiedObservation::sign(
                    *observation,
                    self.latest_child_block,
                    &self.validator_key,
                ) {
                    Ok(v) => Vote::v1(ValidatorKey::new(self.validator_key.public_key()), v),
                    Err(e) => {
                        tracing::error!(err = e.to_string(), "cannot sign received proposal");
                        return;
                    }
                };

                if let Err(e) = self.vote_tally.add_vote(vote.clone()) {
                    tracing::error!(err = e.to_string(), "cannot self vote to tally");
                    return;
                }

                match self.gossip.publish_vote(vote).await {
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
                }
                _ => break,
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
