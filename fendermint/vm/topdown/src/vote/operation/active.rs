// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::vote::gossip::GossipClient;
use crate::vote::operation::paused::PausedOperationMode;
use crate::vote::operation::{OperationMetrics, OperationStateMachine, ACTIVE, PAUSED};
use crate::vote::store::VoteStore;
use crate::syncer::TopDownSyncEvent;
use crate::vote::VotingHandler;
use std::fmt::{Display, Formatter};

/// In active mode, we observe a steady rate of topdown checkpoint commitments on chain.
/// Our lookahead buffer is sliding continuously. As we acquire new finalised parent blocks,
/// we broadcast individual signed votes for every epoch.
pub(crate) struct ActiveOperationMode<G, S> {
    pub(crate) metrics: OperationMetrics,
    pub(crate) handler: VotingHandler<G, S>,
}

impl<G, S> Display for ActiveOperationMode<G, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ACTIVE)
    }
}

impl<G, S> ActiveOperationMode<G, S>
where
    G: GossipClient + Send + Sync + 'static,
    S: VoteStore + Send + Sync + 'static,
{
    pub(crate) async fn advance(mut self) -> OperationStateMachine<G, S> {
        let mut n = self.handler.process_external_request(&self.metrics);
        tracing::debug!(
            num = n,
            status = self.to_string(),
            "handled external requests"
        );

        n = self.handler.process_gossip_subscription_votes();
        tracing::debug!(num = n, status = self.to_string(), "handled gossip votes");

        while let Some(v) = self.handler.poll_internal_event() {
            // top down is now syncing, pause everything
            if matches!(v, TopDownSyncEvent::NodeSyncing) {
                self.metrics.mode_changed(PAUSED);
                return OperationStateMachine::Paused(PausedOperationMode {
                    metrics: self.metrics,
                    handler: self.handler,
                });
            }

            // handle the polled event
            self.handler.handle_event(v).await;
        }

        OperationStateMachine::Active(self)
    }
}
