// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod active;
mod paused;

use crate::vote::operation::active::ActiveOperationMode;
use crate::vote::operation::paused::PausedOperationMode;
use crate::vote::VotingHandler;
use std::fmt::Display;

pub type OperationMode = &'static str;
pub const INITIALIZED: &str = "init";
pub const PAUSED: &str = "paused";
pub const ACTIVE: &str = "active";

/// The operation mode of voting reactor.
///
/// Active: Active publishing votes and aggregating votes normally
/// Paused: Stops voting reactor due to unknown or irrecoverable issues
///
/// State Diagram:
///   [*] --> Paused : Process start
///   Paused --> Active : Synced
///   Active --> Recovery : Checkpoints quiet
///   Recovery --> Active : Checkpoints observed
///   Active --> Paused : Stopping
///   Recovery --> Paused : Stopping
///
///   State: Recovery {
///     [*] --> SoftRecovery
///     SoftRecovery --> HardRecovery : Still no new topdown checkpoints
///     SoftRecovery --> [*] : New checkpoints
///     HardRecovery --> [*] : New checkpoints
///   }
/// TODO: Soft and Hard recovery mode to be added
pub enum OperationStateMachine {
    Paused(PausedOperationMode),
    Active(ActiveOperationMode),
}

/// Tracks the operation mdoe metrics for the voting system
pub(crate) struct OperationMetrics {
    current_mode: OperationMode,
    previous_mode: OperationMode,
}

pub(crate) trait OperationModeHandler: Display {
    fn advance(self) -> OperationStateMachine;
}

impl OperationStateMachine {
    /// Always start with Paused operation mode, one needs to know the exact status from syncer.
    pub fn new(handler: VotingHandler) -> OperationStateMachine {
        let metrics = OperationMetrics {
            current_mode: PAUSED,
            previous_mode: INITIALIZED,
        };
        Self::Paused(PausedOperationMode { metrics, handler })
    }

    pub fn step(self) -> Self {
        match self {
            OperationStateMachine::Paused(p) => p.advance(),
            OperationStateMachine::Active(p) => p.advance(),
        }
    }
}

impl OperationMetrics {
    pub fn mode_changed(&mut self, mode: OperationMode) {
        self.previous_mode = self.current_mode;
        self.current_mode = mode;
    }
}
