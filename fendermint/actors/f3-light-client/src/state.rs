// Copyright 2021-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! State management for the F3 Light Client actor.
//!
//! This module implements the actor's state, which consists of a single
//! LightClientState structure. The state is initialized at genesis and
//! updated as F3 finality progresses on the parent chain.

use crate::types::{LightClientState, PowerEntry};
use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::ActorError;
use serde::{Deserialize, Serialize};

/// State of the F3 light client actor.
///
/// The actor maintains a single light client state that tracks F3 finality
/// from the parent chain. This state is initialized at genesis and updated
/// via UpdateState calls when new finality information arrives.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct State {
    /// F3 Light Client State - initialized at construction, updated via state updates
    pub light_client_state: LightClientState,
}

impl State {
    /// Create a new F3 light client state
    pub fn new(
        instance_id: u64,
        power_table: Vec<PowerEntry>,
        finalized_epochs: Vec<fvm_shared::clock::ChainEpoch>,
    ) -> Result<State, ActorError> {
        let state = State {
            light_client_state: LightClientState {
                instance_id,
                finalized_epochs,
                power_table,
            },
        };
        Ok(state)
    }

    /// Update light client state
    pub fn update_state(
        &mut self,
        _rt: &impl Runtime,
        new_state: LightClientState,
    ) -> Result<(), ActorError> {
        // Validate finalized_epochs is not empty
        if new_state.finalized_epochs.is_empty() {
            return Err(ActorError::illegal_argument(
                "Finalized epochs cannot be empty".to_string(),
            ));
        }

        // Validate instance progression
        if new_state.instance_id == self.light_client_state.instance_id {
            // Same instance: highest epoch must advance
            let current_max = self
                .light_client_state
                .finalized_epochs
                .iter()
                .max()
                .copied()
                .unwrap_or(0);
            let new_max = *new_state
                .finalized_epochs
                .iter()
                .max()
                .expect("finalized_epochs validated as non-empty");
            if new_max <= current_max {
                return Err(ActorError::illegal_argument(format!(
                    "New finalized height {} must be greater than current {}",
                    new_max, current_max
                )));
            }
        } else if new_state.instance_id == self.light_client_state.instance_id + 1 {
            // Next instance: allowed (F3 protocol upgrade)
        } else {
            // Invalid progression (backward or skipping)
            return Err(ActorError::illegal_argument(format!(
                "Invalid instance progression: {} to {} (must increment by 0 or 1)",
                self.light_client_state.instance_id, new_state.instance_id
            )));
        }

        // Update state
        self.light_client_state = new_state;
        Ok(())
    }
}
