// Copyright 2021-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::types::{F3Certificate, PowerEntry};
use cid::Cid;
use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::CborStore;
use fvm_shared::clock::ChainEpoch;
use multihash::Code;
use serde::{Deserialize, Serialize};

/// State of the F3 certificate manager actor
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct State {
    /// Genesis F3 instance ID
    pub genesis_instance_id: u64,
    /// Genesis power table for F3 consensus (stored in blockstore)
    pub genesis_power_table: Cid,
    /// Latest F3 certificate (stored in blockstore)
    pub latest_certificate: Option<Cid>,
    /// Latest finalized height
    pub latest_finalized_height: ChainEpoch,
}

impl State {
    /// Create a new F3 certificate manager state
    pub fn new<BS: Blockstore>(
        store: &BS,
        genesis_instance_id: u64,
        genesis_power_table: Vec<PowerEntry>,
        genesis_certificate: Option<F3Certificate>,
    ) -> Result<State, ActorError> {
        let latest_finalized_height = genesis_certificate
            .as_ref()
            .map(|cert| cert.epoch)
            .unwrap_or(0);

        // Store genesis power table in blockstore
        let genesis_power_table_cid = store
            .put_cbor(&genesis_power_table, Code::Blake2b256)
            .map_err(|e| {
                ActorError::illegal_state(format!("Failed to store genesis power table: {}", e))
            })?;

        // Store genesis certificate in blockstore if provided
        let latest_certificate_cid = if let Some(cert) = &genesis_certificate {
            Some(store.put_cbor(cert, Code::Blake2b256).map_err(|e| {
                ActorError::illegal_state(format!("Failed to store genesis certificate: {}", e))
            })?)
        } else {
            None
        };

        let state = State {
            genesis_instance_id,
            genesis_power_table: genesis_power_table_cid,
            latest_certificate: latest_certificate_cid,
            latest_finalized_height,
        };
        Ok(state)
    }

    /// Update the latest F3 certificate
    pub fn update_certificate(
        &mut self,
        rt: &impl Runtime,
        certificate: F3Certificate,
    ) -> Result<(), ActorError> {
        // Validate that the certificate advances the finalized height
        if certificate.epoch <= self.latest_finalized_height {
            return Err(ActorError::illegal_argument(format!(
                "Certificate epoch {} is not greater than current finalized height {}",
                certificate.epoch, self.latest_finalized_height
            )));
        }

        // Store certificate in blockstore
        let certificate_cid = rt
            .store()
            .put_cbor(&certificate, Code::Blake2b256)
            .map_err(|e| {
                ActorError::illegal_state(format!("Failed to store certificate: {}", e))
            })?;

        // Update state
        self.latest_finalized_height = certificate.epoch;
        self.latest_certificate = Some(certificate_cid);

        Ok(())
    }

    /// Get the latest certificate
    pub fn get_latest_certificate(
        &self,
        rt: &impl Runtime,
    ) -> Result<Option<F3Certificate>, ActorError> {
        if let Some(cid) = &self.latest_certificate {
            let cert = rt
                .store()
                .get_cbor(cid)
                .map_err(|e| {
                    ActorError::illegal_state(format!("Failed to load certificate: {}", e))
                })?
                .ok_or_else(|| {
                    ActorError::illegal_state("Certificate not found in blockstore".to_string())
                })?;
            Ok(Some(cert))
        } else {
            Ok(None)
        }
    }

    /// Get the genesis F3 instance ID
    pub fn get_genesis_instance_id(&self) -> u64 {
        self.genesis_instance_id
    }

    /// Get the genesis power table
    pub fn get_genesis_power_table(
        &self,
        rt: &impl Runtime,
    ) -> Result<Vec<PowerEntry>, ActorError> {
        let power_table = rt
            .store()
            .get_cbor(&self.genesis_power_table)
            .map_err(|e| {
                ActorError::illegal_state(format!("Failed to load genesis power table: {}", e))
            })?
            .ok_or_else(|| {
                ActorError::illegal_state("Genesis power table not found in blockstore".to_string())
            })?;
        Ok(power_table)
    }

    /// Get the latest finalized height
    pub fn get_latest_finalized_height(&self) -> ChainEpoch {
        self.latest_finalized_height
    }
}
