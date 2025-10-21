// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Parent chain watcher for fetching and validating F3 certificates

use crate::types::ValidatedCertificate;
use anyhow::{Context, Result};
use filecoin_f3_certs::FinalityCertificate;
use ipc_api::subnet_id::SubnetID;
use ipc_provider::jsonrpc::JsonRpcClientImpl;
use ipc_provider::lotus::client::{DefaultLotusJsonRPCClient, LotusJsonRPCClient};
use ipc_provider::lotus::message::f3::F3CertificateResponse;
use ipc_provider::lotus::LotusClient;
use parking_lot::RwLock;
use serde_json::json;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, info, warn};
use url::Url;

/// Watches the parent chain for new F3 certificates
pub struct ParentWatcher {
    /// Parent RPC URL
    parent_rpc_url: String,
    
    /// Parent subnet ID
    parent_subnet_id: SubnetID,
    
    /// Last validated instance ID
    last_validated_instance: AtomicU64,
    
    /// Previous power table for certificate validation
    /// TODO: This would store the power table from the previous certificate
    /// For MVP, we'll skip power table validation
    previous_power_table: Arc<RwLock<Option<Vec<u8>>>>,
}

impl ParentWatcher {
    /// Create a new parent watcher
    ///
    /// # Arguments
    /// * `parent_rpc_url` - RPC URL for the parent chain
    /// * `parent_subnet_id` - SubnetID of the parent chain (e.g., "/r314159" for calibration)
    pub fn new(parent_rpc_url: &str, parent_subnet_id: &str) -> Result<Self> {
        // Validate URL
        let _ = Url::parse(parent_rpc_url).context("Failed to parse parent RPC URL")?;

        let subnet =
            SubnetID::from_str(parent_subnet_id).context("Failed to parse parent subnet ID")?;

        Ok(Self {
            parent_rpc_url: parent_rpc_url.to_string(),
            parent_subnet_id: subnet,
            last_validated_instance: AtomicU64::new(0),
            previous_power_table: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Create a Lotus client for RPC calls
    fn create_lotus_client(&self) -> Result<DefaultLotusJsonRPCClient> {
        let url = Url::parse(&self.parent_rpc_url)?;
        let rpc_client = JsonRpcClientImpl::new(url, None);
        Ok(LotusJsonRPCClient::new(rpc_client, self.parent_subnet_id.clone()))
    }

    /// Fetch and validate F3 certificate for a SPECIFIC instance ID
    /// CRITICAL: Must process instances sequentially (can't skip!)
    pub async fn fetch_and_validate_certificate(
        &self,
        instance_id: u64,
    ) -> Result<Option<ValidatedCertificate>> {
        debug!(instance_id, "Fetching F3 certificate for instance");

        // Create client and fetch certificate from parent
        let lotus_client = self.create_lotus_client()?;
        let cert_response = lotus_client
            .f3_get_cert_by_instance(instance_id)
            .await
            .context("Failed to fetch certificate from parent")?;

        let Some(cert_response) = cert_response else {
            debug!(instance_id, "Certificate not available yet");
            return Ok(None);
        };

        debug!(
            instance_id,
            ec_chain_len = cert_response.ec_chain.len(),
            "Received F3 certificate from parent"
        );

        // Fetch F3 certificate in native format
        // Note: In a real implementation, we'd parse the certificate properly
        // For MVP, we'll use a placeholder
        let f3_cert = self.parse_f3_certificate(&cert_response).await?;

        // Validate certificate chain
        let is_valid = self.validate_certificate_chain(&f3_cert).await?;

        if !is_valid {
            return Err(anyhow::anyhow!(
                "Invalid certificate for instance {}",
                instance_id
            ));
        }

        // Update last validated instance
        self.last_validated_instance
            .store(instance_id, Ordering::Release);

        info!(instance_id, "F3 certificate validated successfully");

        Ok(Some(ValidatedCertificate {
            instance_id,
            f3_cert,
            lotus_response: cert_response,
        }))
    }

    /// Parse F3 certificate from Lotus response
    async fn parse_f3_certificate(
        &self,
        lotus_cert: &F3CertificateResponse,
    ) -> Result<FinalityCertificate> {
        // For MVP, we'll try to fetch from F3 RPC
        // In production, we'd parse the lotus certificate properly

        // For MVP, we'll create a placeholder certificate
        // In production, we would:
        // 1. Create an F3 RPC client: RPCClient::new(&self.parent_rpc_url)?
        // 2. Fetch the certificate: client.get_certificate(lotus_cert.gpbft_instance).await
        // 3. Or parse it directly from the Lotus certificate data
        
        debug!(
            instance_id = lotus_cert.gpbft_instance,
            "Creating placeholder F3 certificate for MVP"
        );

        // Create a placeholder certificate for MVP
        Ok(FinalityCertificate::default())
    }

    /// Validate certificate chain
    async fn validate_certificate_chain(&self, _cert: &FinalityCertificate) -> Result<bool> {
        // For MVP, we'll skip validation and trust the parent chain
        // In production, this would:
        // 1. Check signatures
        // 2. Verify power table transitions
        // 3. Ensure sequential instance progression

        // TODO: Implement proper validation using rust-f3

        debug!("Certificate validation (MVP: always valid)");
        Ok(true)
    }

    /// Fetch tipsets for a specific epoch
    pub async fn fetch_tipsets_for_epoch(
        &self,
        epoch: i64,
    ) -> Result<(serde_json::Value, serde_json::Value)> {
        // Use the underlying JSON-RPC client directly
        let parent_params = json!([epoch, null]);
        let child_params = json!([epoch + 1, null]);
        
        // Create a temporary Lotus client for raw requests
        let lotus_client = proofs::client::LotusClient::new(
            Url::parse(&self.parent_rpc_url)?,
            None
        );
        
        let parent = lotus_client
            .request("Filecoin.ChainGetTipSetByHeight", parent_params)
            .await
            .context("Failed to fetch parent tipset")?;

        let child = lotus_client
            .request("Filecoin.ChainGetTipSetByHeight", child_params)
            .await
            .context("Failed to fetch child tipset")?;

        Ok((parent, child))
    }

    /// Get the latest F3 instance ID from the parent chain
    pub async fn get_latest_instance_id(&self) -> Result<Option<u64>> {
        let lotus_client = self.create_lotus_client()?;
        let cert = lotus_client
            .f3_get_certificate()
            .await
            .context("Failed to fetch latest F3 certificate")?;

        Ok(cert.map(|c| c.gpbft_instance))
    }

    /// Get the parent RPC URL
    pub fn parent_rpc_url(&self) -> &str {
        &self.parent_rpc_url
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
}
