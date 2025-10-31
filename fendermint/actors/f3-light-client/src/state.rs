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
    ///
    /// This method should only be called from consensus code path which
    /// contains the lightclient verifier. No additional validation is
    /// performed here as it's expected to be done by the verifier.
    pub fn update_state(
        &mut self,
        _rt: &impl Runtime,
        new_state: LightClientState,
    ) -> Result<(), ActorError> {
        self.light_client_state = new_state;
        Ok(())
    }
}
