// Copyright 2021-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! State management for the F3 Light Client actor.
//!
//! This module implements the actor's state, which consists of a single
//! LightClientState structure. The state is initialized at genesis and
//! updated as F3 finality progresses on the parent chain.

use crate::types::{LightClientState, PowerEntry};
use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{ActorError, Map2, DEFAULT_HAMT_CONFIG};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
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

/// Stored HAMT value for power table entries.
///
/// The key of the HAMT is the validator ID, so storing `id` in the value would be redundant.
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub(crate) struct PowerEntryValue {
    pub public_key: Vec<u8>,
    pub power_be: Vec<u8>,
}

pub(crate) type PowerTable<BS> = Map2<BS, u64, PowerEntryValue>;

impl State {
    /// Create a new F3 light client state
    pub fn new<BS: Blockstore>(
        store: &BS,
        latest_instance_id: u64,
        latest_finalized_height: ChainEpoch,
        power_table: Vec<PowerEntry>,
    ) -> Result<State, ActorError> {
        let power_table_root = {
            let mut m = PowerTable::empty(store, DEFAULT_HAMT_CONFIG, "f3_power_table");
            for pe in power_table {
                let id = pe.id;
                m.set(
                    &id,
                    PowerEntryValue {
                        public_key: pe.public_key,
                        power_be: pe.power_be,
                    },
                )?;
            }
            m.flush()?
        };

        let state = State {
            light_client_state: LightClientState {
                latest_instance_id,
                latest_finalized_height,
                power_table_root,
            },
        };
        Ok(state)
    }

    /// Update light client state
    pub fn update_state(
        &mut self,
        rt: &impl Runtime,
        latest_instance_id: u64,
        latest_finalized_height: ChainEpoch,
        power_table: Vec<PowerEntry>,
    ) -> Result<(), ActorError> {
        let power_table_root = {
            let mut m = PowerTable::empty(rt.store(), DEFAULT_HAMT_CONFIG, "f3_power_table");
            for pe in power_table {
                let id = pe.id;
                m.set(
                    &id,
                    PowerEntryValue {
                        public_key: pe.public_key,
                        power_be: pe.power_be,
                    },
                )?;
            }
            m.flush()?
        };

        self.light_client_state.latest_instance_id = latest_instance_id;
        self.light_client_state.latest_finalized_height = latest_finalized_height;
        self.light_client_state.power_table_root = power_table_root;
        Ok(())
    }
}
