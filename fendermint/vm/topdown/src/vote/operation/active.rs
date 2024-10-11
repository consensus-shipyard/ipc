// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::vote::operation::paused::PausedOperationMode;
use crate::vote::operation::{
    OperationMetrics, OperationModeHandler, OperationStateMachine, PAUSED,
};
use crate::vote::TopDownSyncEvent;
use crate::vote::VotingHandler;
use std::fmt::{Display, Formatter};

/// In active mode, we observe a steady rate of topdown checkpoint commitments on chain.
/// Our lookahead buffer is sliding continuously. As we acquire new finalised parent blocks,
/// we broadcast individual signed votes for every epoch.
pub(crate) struct ActiveOperationMode {
    pub(crate) metrics: OperationMetrics,
    pub(crate) handler: VotingHandler,
}

impl Display for ActiveOperationMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "active")
    }
}

impl OperationModeHandler for ActiveOperationMode {
    fn advance(mut self) -> OperationStateMachine {
        let mut n = self.handler.process_external_request(&self.metrics);
        tracing::debug!(
            num = n,
            status = self.to_string(),
            "handled external requests"
        );

        n = self.handler.process_gossip_subscription_votes();
        tracing::debug!(num = n, status = self.to_string(), "handled gossip votes");

        if n == 0 {
            todo!("handle transition to soft recover")
        }

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
            self.handler.handle_event(v);
        }

        OperationStateMachine::Active(self)
    }
}
