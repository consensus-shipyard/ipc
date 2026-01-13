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
use fvm_shared::clock::ChainEpoch;
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
        latest_instance_id: u64,
        latest_finalized_height: Option<ChainEpoch>,
        power_table: Vec<PowerEntry>,
    ) -> Result<State, ActorError> {
        let state = State {
            light_client_state: LightClientState {
                latest_instance_id,
                latest_finalized_height,
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
        self.light_client_state = new_state;
        Ok(())
    }
}
