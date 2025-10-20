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
    /// Latest F3 certificate
    pub latest_certificate: Option<F3Certificate>,
    /// Latest finalized height
    pub latest_finalized_height: ChainEpoch,
}

impl State {
    /// Create a new F3 certificate manager state
    pub fn new(
        genesis_instance_id: u64,
        genesis_power_table: Vec<PowerEntry>,
        genesis_certificate: Option<F3Certificate>,
    ) -> Result<State, ActorError> {
        let latest_finalized_height = genesis_certificate
            .as_ref()
            .map(|cert| cert.epoch)
            .unwrap_or(0);

        let state = State {
            genesis_instance_id,
            genesis_power_table,
            latest_certificate: genesis_certificate,
            latest_finalized_height,
        };
        Ok(state)
    }

    /// Update the latest F3 certificate
    pub fn update_certificate(
        &mut self,
        _rt: &impl Runtime,
        certificate: F3Certificate,
    ) -> Result<(), ActorError> {
        // Validate that the certificate advances the finalized height
        if certificate.epoch <= self.latest_finalized_height {
            return Err(ActorError::illegal_argument(format!(
                "Certificate epoch {} is not greater than current finalized height {}",
                certificate.epoch, self.latest_finalized_height
            )));
        }

        // Update state - the transaction will handle persisting this
        self.latest_finalized_height = certificate.epoch;
        self.latest_certificate = Some(certificate);

        Ok(())
    }

    /// Get the latest certificate
    pub fn get_latest_certificate(&self) -> Option<&F3Certificate> {
        self.latest_certificate.as_ref()
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
