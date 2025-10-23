// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! F3 client wrapper for certificate fetching and validation
//!
//! Wraps the F3 light client to provide:
//! - Certificate fetching from F3 RPC
//! - Full cryptographic validation (BLS signatures, quorum, chain continuity)
//! - Sequential state management for validated certificates

use crate::types::ValidatedCertificate;
use anyhow::{Context, Result};
use filecoin_f3_lightclient::{LightClient, LightClientState};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// F3 client for fetching and validating certificates
///
/// Uses the F3 light client for:
/// - Direct F3 RPC access
/// - Full cryptographic validation (BLS signatures, quorum, continuity)
/// - Stateful sequential validation
pub struct F3Client {
    /// Light client for F3 RPC and cryptographic validation
    /// Using Mutex to allow async methods
    light_client: Arc<Mutex<LightClient>>,

    /// Current validated state (instance, chain, power table)
    state: Arc<Mutex<LightClientState>>,
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
    /// * `power_table` - Initial trusted power table (from F3CertManager actor)
    pub fn new(
        rpc_endpoint: &str,
        network_name: &str,
        initial_instance: u64,
        power_table: filecoin_f3_gpbft::PowerEntries,
    ) -> Result<Self> {
        let light_client = LightClient::new(rpc_endpoint, network_name)
            .context("Failed to create F3 light client")?;

        // Initialize state with provided power table from actor
        let state = LightClientState {
            instance: initial_instance,
            chain: None,
            power_table,
        };

        info!(
            initial_instance,
            power_table_size = state.power_table.len(),
            network = network_name,
            rpc = rpc_endpoint,
            "Created F3 client with power table from F3CertManager actor"
        );

        Ok(Self {
            light_client: Arc::new(Mutex::new(light_client)),
            state: Arc::new(Mutex::new(state)),
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
            light_client: Arc::new(Mutex::new(light_client)),
            state: Arc::new(Mutex::new(state)),
        })
    }

    /// Fetch and validate an F3 certificate
    ///
    /// This performs full cryptographic validation including:
    /// - ✅ BLS signature correctness
    /// - ✅ Quorum requirements (>2/3 power)
    /// - ✅ Chain continuity (sequential instances)
    /// - ✅ Power table validity
    ///
    /// # Returns
    /// `ValidatedCertificate` containing the cryptographically verified certificate
    pub async fn fetch_and_validate(&self, instance: u64) -> Result<ValidatedCertificate> {
        debug!(instance, "Fetching and validating F3 certificate");

        // STEP 1: FETCH certificate from F3 RPC
        let f3_cert = self
            .light_client
            .lock()
            .await
            .get_certificate(instance)
            .await
            .context("Failed to fetch certificate from F3 RPC")?;

        debug!(
            instance,
            ec_chain_len = f3_cert.ec_chain.suffix().len(),
            "Fetched certificate from F3 RPC"
        );

        // STEP 2: CRYPTOGRAPHIC VALIDATION
        // The light client performs full validation: BLS signatures, quorum, continuity
        let new_state = {
            let mut client = self.light_client.lock().await;
            let state = self.state.lock().await.clone();
            client
                .validate_certificates(&state, &[f3_cert.clone()])
                .context("Certificate cryptographic validation failed")?
        };

        debug!(
            instance,
            new_instance = new_state.instance,
            power_table_size = new_state.power_table.len(),
            "Certificate cryptographically validated (BLS, quorum, continuity verified)"
        );

        // STEP 3: UPDATE validated state
        *self.state.lock().await = new_state;

        info!(
            instance,
            "Certificate validated with full cryptographic verification"
        );

        Ok(ValidatedCertificate {
            instance_id: instance,
            f3_cert,
        })
    }

    /// Get current instance
    pub async fn current_instance(&self) -> u64 {
        self.state.lock().await.instance
    }

    /// Get current validated state
    pub async fn get_state(&self) -> LightClientState {
        self.state.lock().await.clone()
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

        let result = F3Client::new("http://localhost:1234", "calibrationnet", 0, power_table);

        assert!(result.is_ok());
    }
}
