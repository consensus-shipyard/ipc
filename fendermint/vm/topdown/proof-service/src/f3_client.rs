// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! F3 client wrapper for certificate fetching and validation
//!
//! Wraps the F3 light client to provide:
//! - Certificate fetching from F3 RPC
//! - Full cryptographic validation (BLS signatures, quorum, chain continuity)
//! - Sequential state management for validated certificates

use crate::observe::{F3CertificateFetched, F3CertificateValidated, OperationStatus};
use anyhow::{Context, Result};
use filecoin_f3_certs::FinalityCertificate;
use filecoin_f3_gpbft::PowerEntries;
use filecoin_f3_lightclient::{LightClient, LightClientState};
use ipc_observability::emit;
use tracing::{debug, error, info};

/// F3 client for fetching and validating certificates
///
/// Uses the F3 light client for:
/// - Direct F3 RPC access
/// - Full cryptographic validation (BLS signatures, quorum, continuity)
/// - Stateful sequential validation
///
/// This client is designed to be owned by a single service and accessed
/// sequentially. Methods that mutate state take `&mut self`.
pub struct F3Client {
    /// Light client for F3 RPC and cryptographic validation
    light_client: LightClient,

    /// Current validated state (instance, chain, power table)
    state: LightClientState,

    /// F3 RPC endpoint
    rpc_endpoint: String,
}

impl F3Client {
    /// Create a new F3 client with provided power table (PRODUCTION USE)
    ///
    /// This is the primary constructor for production use. The power table and
    /// initial instance should come from the F3CertManager actor on-chain.
    ///
    /// # Arguments
    /// * `rpc_endpoint` - F3 RPC endpoint
    /// * `network_name` - Network name (e.g., "calibrationnet", "mainnet")
    /// * `initial_instance` - F3 instance to bootstrap from (from F3CertManager actor)
    /// * `initial_power_table` - Initial trusted power table (from F3CertManager actor)
    pub fn new(
        rpc_endpoint: &str,
        network_name: &str,
        initial_instance: u64,
        initial_power_table: filecoin_f3_gpbft::PowerEntries,
    ) -> Result<Self> {
        let light_client = LightClient::new(rpc_endpoint, network_name)
            .context("Failed to create F3 light client")?;

        // Initialize state with provided power table from actor
        let state = LightClientState {
            instance: initial_instance,
            chain: None,
            power_table: initial_power_table.clone(),
        };

        info!(
            initial_instance,
            power_table_size = initial_power_table.len(),
            network = network_name,
            rpc = rpc_endpoint,
            "Created F3 client with power table from F3CertManager actor"
        );

        Ok(Self {
            light_client,
            state,
            rpc_endpoint: rpc_endpoint.to_string(),
        })
    }

    /// Create F3 client by fetching power table from RPC (TESTING ONLY)
    ///
    /// For testing/development. In production, use `new()` with power table from
    /// the F3CertManager actor on-chain.
    ///
    /// # Arguments
    /// * `rpc_endpoint` - F3 RPC endpoint
    /// * `network_name` - Network name (e.g., "calibrationnet", "mainnet")
    /// * `initial_instance` - F3 instance to bootstrap from
    #[doc(hidden)]
    pub async fn new_from_rpc(
        rpc_endpoint: &str,
        network_name: &str,
        initial_instance: u64,
    ) -> Result<Self> {
        let mut light_client = LightClient::new(rpc_endpoint, network_name)
            .context("Failed to create F3 light client")?;

        // Fetch initial power table from RPC (for testing)
        let state = light_client
            .initialize(initial_instance)
            .await
            .context("Failed to initialize light client with power table from RPC")?;

        info!(
            initial_instance,
            power_table_size = state.power_table.len(),
            network = network_name,
            "Created F3 client with power table from RPC (testing mode)"
        );

        Ok(Self {
            light_client,
            state,
            rpc_endpoint: rpc_endpoint.to_string(),
        })
    }

    /// Fetch and validate an F3 certificate
    ///
    /// This performs full cryptographic validation including:
    /// - BLS signature correctness
    /// - Quorum requirements (>2/3 power)
    /// - Chain continuity (sequential instances)
    /// - Power table validity
    ///
    /// # Returns
    /// `FinalityCertificate` that has been cryptographically verified
    pub async fn fetch_and_validate(&mut self) -> Result<(FinalityCertificate, PowerEntries)> {
        // The light client state tracks the next instance expected by validation.
        // Fetch that exact instance from RPC; fetching `+1` causes continuity failures
        // like "expected instance N, found instance N+1".
        let instance = self.state.instance;

        debug!(instance, "Starting F3 certificate fetch and validation");

        // Fetch certificate from F3 RPC first
        let certificate = self.fetch_certificate(instance).await?;

        // Then validate the certificate cryptography
        debug!(instance, "Validating certificate cryptography");
        let new_state = self.validate_certificate(&certificate)?;
        let power_table = new_state.power_table.clone();

        debug!(
            instance,
            current_instance = self.state.instance,
            power_table_entries = self.state.power_table.len(),
            "Current F3 validator state"
        );

        // Update the state with the new validated state
        self.state = new_state;

        debug!(instance, "Certificate validation complete");

        Ok((certificate, power_table))
    }

    /// Snapshot the current validated state.
    ///
    /// Used by the proof-service to provide *all-or-nothing* semantics:
    /// if proof generation fails after fetching/validating a certificate, we can roll back
    /// and retry the same certificate on the next tick.
    pub fn checkpoint_state(&self) -> LightClientState {
        self.state.clone()
    }

    /// Restore a previously checkpointed validated state.
    pub fn restore_state(&mut self, state: LightClientState) {
        self.state = state;
    }

    async fn fetch_certificate(&self, instance: u64) -> Result<FinalityCertificate> {
        match self.light_client.get_certificate(instance).await {
            Ok(cert) => {
                emit(F3CertificateFetched {
                    instance,
                    ec_chain_len: cert.ec_chain.suffix().len(),
                    status: OperationStatus::Success,
                    latency: 0.0,
                });
                debug!(
                    instance,
                    ec_chain_len = cert.ec_chain.suffix().len(),
                    "Fetched certificate from F3 RPC"
                );
                Ok(cert)
            }
            Err(e) => {
                emit(F3CertificateFetched {
                    instance,
                    ec_chain_len: 0,
                    status: OperationStatus::Failure,
                    latency: 0.0,
                });
                error!(
                    instance,
                    error = %e,
                    "Failed to fetch certificate from F3 RPC"
                );
                Err(e).context("Failed to fetch certificate from F3 RPC")
            }
        }
    }

    fn validate_certificate(
        &mut self,
        certificate: &FinalityCertificate,
    ) -> Result<LightClientState> {
        let instance = certificate.gpbft_instance;

        match self
            .light_client
            .validate_certificates(&self.state, &[certificate.clone()])
        {
            Ok(new_state) => {
                emit(F3CertificateValidated {
                    instance,
                    new_instance: new_state.instance,
                    power_table_size: new_state.power_table.len(),
                    status: OperationStatus::Success,
                    latency: 0.0,
                });
                info!(
                    instance,
                    new_instance = new_state.instance,
                    power_table_size = new_state.power_table.len(),
                    "Certificate validated (BLS signatures, quorum, continuity)"
                );
                Ok(new_state)
            }
            Err(e) => {
                emit(F3CertificateValidated {
                    instance,
                    new_instance: self.state.instance,
                    power_table_size: self.state.power_table.len(),
                    status: OperationStatus::Failure,
                    latency: 0.0,
                });
                error!(
                    instance,
                    error = %e,
                    current_instance = self.state.instance,
                    power_table_entries = self.state.power_table.len(),
                    "Certificate validation failed"
                );
                Err(e).context("Certificate cryptographic validation failed")
            }
        }
    }

    /// Get current instance
    pub fn current_instance(&self) -> u64 {
        self.state.instance
    }

    /// Get current validated state
    pub fn get_state(&self) -> &LightClientState {
        &self.state
    }

    /// Get F3 RPC endpoint
    pub fn rpc_endpoint(&self) -> &str {
        &self.rpc_endpoint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f3_client_creation() {
        use filecoin_f3_gpbft::PowerEntries;

        // Creating a client requires actual RPC endpoint
        // Real test would need integration test with live network
        let power_table = PowerEntries(vec![]);

        let result = F3Client::new("http://localhost:1234", "calibrationnet2", 0, power_table);

        assert!(result.is_ok());
    }
}
