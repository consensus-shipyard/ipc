// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::sync::TopDownSyncEvent;
use crate::vote::gossip::GossipClient;
use crate::vote::operation::active::ActiveOperationMode;
use crate::vote::store::VoteStore;
use crate::vote::operation::{
    OperationMetrics, OperationMode, OperationStateMachine,
};
use crate::vote::VotingHandler;
use std::fmt::{Display, Formatter};
use tokio::select;

/// The paused operation mode handler.
///
/// Paused mode engages when we’re catching up with the subnet chain
/// (usually after a process interruption, restart, or first start).
/// Therefore, we still don’t know what the last committed topdown checkpoint is,
/// so we refrain from watching the parent chain, and from gossiping
/// any certified observations until we switch to active mode.
pub(crate) struct PausedOperationMode<G, S> {
    pub metrics: OperationMetrics,
    pub(crate) handler: VotingHandler<G, S>,
}

impl<G, S> Display for PausedOperationMode<G, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PAUSED")
    }
}

impl <G, S> PausedOperationMode<G, S> {
    fn into_active(mut self) -> OperationStateMachine<G, S> {
        self.metrics.mode_changed(OperationMode::Active);
        OperationStateMachine::Active(ActiveOperationMode {
            metrics: self.metrics,
            handler: self.handler,
        })
    }
}

impl <G: Send + Sync + GossipClient + 'static + Clone, S: VoteStore + Send + Sync + 'static> PausedOperationMode<G, S> {
    pub(crate) async fn advance(mut self) -> OperationStateMachine<G, S> {
        loop {
            select! {
                Some(req) = self.handler.req_rx.recv() => {
                    self.handler.handle_request(req, &self.metrics);
                },
                Ok(vote) = self.handler.gossip.recv_vote() => {
                    self.handler.record_vote(vote);
                },
                Ok(event) = self.handler.internal_event_listener.recv() => {
                    // top down is still syncing, pause everything
                    if !matches!(event, TopDownSyncEvent::NodeSyncing) {
                        return self.into_active();
                    }
                    self.handler.handle_event(event);
                }
            }
        }
    }
}
