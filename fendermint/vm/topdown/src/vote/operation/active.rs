// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::vote::gossip::GossipClient;
use crate::vote::operation::paused::PausedOperationMode;
use crate::vote::store::VoteStore;
use crate::vote::operation::{
    OperationMetrics, OperationMode, OperationStateMachine,
};
use crate::vote::TopDownSyncEvent;
use crate::vote::VotingHandler;
use async_trait::async_trait;
use std::fmt::{Display, Formatter};
use tokio::select;

/// In active mode, we observe a steady rate of topdown checkpoint commitments on chain.
/// Our lookahead buffer is sliding continuously. As we acquire new finalised parent blocks,
/// we broadcast individual signed votes for every epoch.
pub(crate) struct ActiveOperationMode<G, S> {
    pub(crate) metrics: OperationMetrics,
    pub(crate) handler: VotingHandler<G, S>,
}

impl<G, S> Display for ActiveOperationMode<G, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "ACTIVE")
    }
}

impl <G, S> ActiveOperationMode<G, S> {
    fn into_paused(mut self) -> OperationStateMachine<G, S> {
        self.metrics.mode_changed(OperationMode::Paused);
        OperationStateMachine::Paused(PausedOperationMode {
            metrics: self.metrics,
            handler: self.handler,
        })
    }
}

impl <G: Send + Sync + 'static + GossipClient + Clone, S: VoteStore + Send + Sync + 'static> ActiveOperationMode<G, S> {
    pub(crate) async fn advance(mut self) -> OperationStateMachine<G, S> {
        loop {
            select! {
                Some(req) = self.handler.req_rx.recv() => {
                    self.handler.handle_request(req, &self.metrics);
                },
                Ok(vote) = self.handler.gossip.recv_vote() => {
                    self.handler.record_vote(vote);

                    // TODO: need to handle soft recovery transition
                },
                Ok(event) = self.handler.internal_event_listener.recv() => {
                    // top down is now syncing, pause everything
                    if matches!(event, TopDownSyncEvent::NodeSyncing) {
                        return self.into_paused();
                    }
                    self.handler.handle_event(event);
                }
            }
        }
    }
}
