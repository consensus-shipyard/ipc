// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Parent chain watcher for fetching F3 certificates

use anyhow::{Context, Result};
use ipc_api::subnet_id::SubnetID;
use ipc_provider::jsonrpc::JsonRpcClientImpl;
use ipc_provider::lotus::client::{DefaultLotusJsonRPCClient, LotusJsonRPCClient};
use ipc_provider::lotus::message::f3::F3CertificateResponse;
use ipc_provider::lotus::LotusClient; // Import trait for methods
use std::str::FromStr;
use std::sync::Arc;
use url::Url;

/// Watches the parent chain for new F3 certificates
pub struct ParentWatcher {
    /// Lotus RPC client
    lotus_client: Arc<DefaultLotusJsonRPCClient>,
}

impl ParentWatcher {
    /// Create a new parent watcher
    ///
    /// # Arguments
    /// * `parent_rpc_url` - RPC URL for the parent chain
    /// * `parent_subnet_id` - SubnetID of the parent chain (e.g., "/r314159" for calibration)
    pub fn new(parent_rpc_url: &str, parent_subnet_id: &str) -> Result<Self> {
        let url = Url::parse(parent_rpc_url).context("Failed to parse parent RPC URL")?;

        let subnet =
            SubnetID::from_str(parent_subnet_id).context("Failed to parse parent subnet ID")?;

        // Create the JSON-RPC client
        let rpc_client = JsonRpcClientImpl::new(url, None);

        // Wrap in Lotus client
        let lotus_client = Arc::new(LotusJsonRPCClient::new(rpc_client, subnet));

        Ok(Self { lotus_client })
    }

    /// Fetch the latest F3 certificate from the parent chain
    pub async fn fetch_latest_certificate(&self) -> Result<Option<F3CertificateResponse>> {
        tracing::debug!("Fetching latest F3 certificate from parent");

        let cert = self
            .lotus_client
            .f3_get_certificate()
            .await
            .context("Failed to fetch F3 certificate from parent")?;

        if let Some(ref c) = cert {
            tracing::debug!(
                instance_id = c.gpbft_instance,
                ec_chain_len = c.ec_chain.len(),
                "Received F3 certificate from parent"
            );
        } else {
            tracing::debug!("No F3 certificate available on parent yet");
        }

        Ok(cert)
    }

    /// Fetch F3 certificate for a SPECIFIC instance ID
    /// This is critical for sequential processing and gap recovery
    pub async fn fetch_certificate_by_instance(
        &self,
        instance_id: u64,
    ) -> Result<Option<F3CertificateResponse>> {
        tracing::debug!(instance_id, "Fetching F3 certificate for specific instance");

        let cert = self
            .lotus_client
            .f3_get_cert_by_instance(instance_id)
            .await
            .context("Failed to fetch F3 certificate by instance")?;

        if let Some(ref c) = cert {
            tracing::debug!(
                instance_id = c.gpbft_instance,
                ec_chain_len = c.ec_chain.len(),
                "Received F3 certificate for instance"
            );
        } else {
            tracing::debug!(
                instance_id,
                "Certificate not available for this instance yet"
            );
        }

        Ok(cert)
    }

    /// Get the current F3 instance ID from the latest certificate
    pub async fn get_latest_instance_id(&self) -> Result<Option<u64>> {
        let cert = self.fetch_latest_certificate().await?;
        Ok(cert.map(|c| c.gpbft_instance))
    }

    /// Fetch the F3 power table for a given instance
    pub async fn fetch_power_table(
        &self,
        instance_id: u64,
    ) -> Result<Vec<ipc_provider::lotus::message::f3::F3PowerEntry>> {
        tracing::debug!(instance_id, "Fetching F3 power table");

        let power_table = self
            .lotus_client
            .f3_get_power_table(instance_id)
            .await
            .context("Failed to fetch F3 power table")?;

        tracing::debug!(
            instance_id,
            entries = power_table.len(),
            "Received F3 power table"
        );

        Ok(power_table)
    }

    /// Get reference to the Lotus client (for proof generation)
    pub fn lotus_client(&self) -> &Arc<DefaultLotusJsonRPCClient> {
        &self.lotus_client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_creation() {
        // Valid URL and subnet
        let watcher = ParentWatcher::new("http://localhost:1234/rpc/v1", "/r314159");
        assert!(watcher.is_ok());

        // Invalid URL
        let watcher = ParentWatcher::new("not a url", "/r314159");
        assert!(watcher.is_err());

        // Invalid subnet ID
        let watcher = ParentWatcher::new("http://localhost:1234/rpc/v1", "invalid");
        assert!(watcher.is_err());
    }

    // Note: Integration tests with actual RPC calls would require
    // a running Lotus node, so we keep unit tests minimal
}
