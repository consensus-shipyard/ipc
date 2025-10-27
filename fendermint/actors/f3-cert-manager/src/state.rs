// Copyright 2021-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::types::{F3Certificate, PowerEntry};
use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::ActorError;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

/// State of the F3 certificate manager actor
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct State {
    /// Genesis F3 instance ID
    pub genesis_instance_id: u64,
    /// Genesis power table for F3 consensus
    pub genesis_power_table: Vec<PowerEntry>,
    /// Current F3 instance ID (updated via certificates)
    pub current_instance_id: u64,
    /// Latest finalized height
    pub latest_finalized_height: ChainEpoch,
}

impl State {
    /// Create a new F3 certificate manager state
    pub fn new(
        genesis_instance_id: u64,
        genesis_power_table: Vec<PowerEntry>,
    ) -> Result<State, ActorError> {
        let state = State {
            genesis_instance_id,
            genesis_power_table,
            current_instance_id: genesis_instance_id,
            latest_finalized_height: 0,
        };
        Ok(state)
    }

    /// Update state from F3 certificate (without storing the certificate)
    pub fn update_certificate(
        &mut self,
        _rt: &impl Runtime,
        certificate: F3Certificate,
    ) -> Result<(), ActorError> {
        // Validate finalized_epochs is not empty
        if certificate.finalized_epochs.is_empty() {
            return Err(ActorError::illegal_argument(
                "Certificate must have at least one finalized epoch".to_string(),
            ));
        }

        // Validate instance progression
        if certificate.instance_id == self.current_instance_id {
            // Same instance: highest epoch must advance
            let new_highest = certificate
                .finalized_epochs
                .iter()
                .max()
                .expect("finalized_epochs validated as non-empty");
            if *new_highest <= self.latest_finalized_height {
                return Err(ActorError::illegal_argument(format!(
                    "Certificate highest epoch {} must be greater than current finalized height {}",
                    new_highest, self.latest_finalized_height
                )));
            }
        } else if certificate.instance_id == self.current_instance_id + 1 {
            // Next instance: allowed (F3 protocol upgrade)
            self.current_instance_id = certificate.instance_id;
        } else {
            // Invalid progression (backward or skipping)
            return Err(ActorError::illegal_argument(format!(
                "Invalid instance progression: {} to {} (must increment by 0 or 1)",
                self.current_instance_id, certificate.instance_id
            )));
        }

        // Update state - set latest_finalized_height to the highest epoch
        self.latest_finalized_height = *certificate
            .finalized_epochs
            .iter()
            .max()
            .expect("finalized_epochs validated as non-empty");

        Ok(())
    }

    /// Get the current F3 instance ID
    pub fn get_current_instance_id(&self) -> u64 {
        self.current_instance_id
    }

    /// Get the genesis F3 instance ID
    pub fn get_genesis_instance_id(&self) -> u64 {
        self.genesis_instance_id
    }

    /// Get the genesis power table
    pub fn get_genesis_power_table(&self) -> &[PowerEntry] {
        &self.genesis_power_table
    }

    /// Get the latest finalized height
    pub fn get_latest_finalized_height(&self) -> ChainEpoch {
        self.latest_finalized_height
    }
}
