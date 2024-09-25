// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::sync::TopDownSyncEvent;
use crate::vote::gossip::GossipClient;
use crate::vote::operation::active::ActiveOperationMode;
use crate::vote::operation::{OperationMetrics, OperationStateMachine, ACTIVE, PAUSED};
use crate::vote::store::VoteStore;
use crate::vote::VotingHandler;
use std::fmt::{Display, Formatter};

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
        write!(f, "{}", PAUSED)
    }
}

impl<G, S> PausedOperationMode<G, S>
where
    G: GossipClient + Send + Sync + 'static,
    S: VoteStore + Send + Sync + 'static,
{
    pub(crate) async fn advance(mut self) -> OperationStateMachine<G, S> {
        let n = self.handler.process_external_request(&self.metrics);
        tracing::debug!(
            num = n,
            status = self.to_string(),
            "handled external requests"
        );

        if let Some(v) = self.handler.poll_internal_event() {
            // top down is still syncing, not doing anything for now
            if matches!(v, TopDownSyncEvent::NodeSyncing) {
                return OperationStateMachine::Paused(self);
            }

            // handle the polled event
            self.handler.handle_event(v).await;
        }

        self.metrics.mode_changed(ACTIVE);
        OperationStateMachine::Active(ActiveOperationMode {
            metrics: self.metrics,
            handler: self.handler,
        })
    }
}
