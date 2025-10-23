// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof bundle assembler
//!
//! Generates cryptographic proofs for parent chain finality using the
//! ipc-filecoin-proofs library. The assembler is only responsible for
//! proof generation - it has no knowledge of cache entries or storage.

use anyhow::{Context, Result};
use filecoin_f3_certs::FinalityCertificate;
use proofs::{
    client::LotusClient,
    proofs::{
        calculate_storage_slot, common::bundle::UnifiedProofBundle, generate_proof_bundle,
        EventProofSpec, StorageProofSpec,
    },
};
use url::Url;

/// Assembles proof bundles from F3 certificates and parent chain data
///
/// # Thread Safety
///
/// LotusClient from the proofs library uses Rc/RefCell internally, so it's not Send.
/// We store the URL and create clients on-demand instead of storing the client.
pub struct ProofAssembler {
    rpc_url: Url,
    gateway_actor_id: u64,
    subnet_id: String,
}

impl ProofAssembler {
    /// Create a new proof assembler
    pub fn new(rpc_url: String, gateway_actor_id: u64, subnet_id: String) -> Result<Self> {
        let url = Url::parse(&rpc_url).context("Failed to parse RPC URL")?;

        Ok(Self {
            rpc_url: url,
            gateway_actor_id,
            subnet_id,
        })
    }

    /// Create a LotusClient for making requests
    ///
    /// LotusClient is not Send, so we create it on-demand in each async function
    /// rather than storing it as a field.
    fn create_client(&self) -> LotusClient {
        LotusClient::new(self.rpc_url.clone(), None)
    }

    /// Generate proof bundle for a certificate
    ///
    /// Takes a certificate and tipsets, generates storage and event proofs.
    ///
    /// # Arguments
    /// * `certificate` - Cryptographically validated F3 certificate
    /// * `parent_tipset` - Parent tipset JSON
    /// * `child_tipset` - Child tipset JSON
    ///
    /// # Returns
    /// Typed unified proof bundle (storage + event proofs + witness blocks)
    pub async fn generate_proof_bundle(
        &self,
        certificate: &FinalityCertificate,
        parent_tipset: &serde_json::Value,
        child_tipset: &serde_json::Value,
    ) -> Result<UnifiedProofBundle> {
        let highest_epoch = certificate
            .ec_chain
            .suffix()
            .last()
            .map(|ts| ts.epoch)
            .context("No epochs in certificate")?;

        tracing::debug!(
            instance_id = certificate.gpbft_instance,
            highest_epoch,
            "Generating proof bundle"
        );

        // Deserialize tipsets from JSON
        let parent_api: proofs::client::types::ApiTipset =
            serde_json::from_value(parent_tipset.clone())
                .context("Failed to deserialize parent tipset")?;
        let child_api: proofs::client::types::ApiTipset =
            serde_json::from_value(child_tipset.clone())
                .context("Failed to deserialize child tipset")?;

        // Configure proof specs
        let storage_specs = vec![StorageProofSpec {
            actor_id: self.gateway_actor_id,
            slot: calculate_storage_slot(&self.subnet_id, 0),
        }];

        let event_specs = vec![EventProofSpec {
            event_signature: "NewTopDownMessage(bytes32,uint256)".to_string(),
            topic_1: self.subnet_id.clone(),
            actor_id_filter: Some(self.gateway_actor_id),
        }];

        tracing::debug!(
            highest_epoch,
            storage_specs_count = storage_specs.len(),
            event_specs_count = event_specs.len(),
            "Configured proof specs"
        );

        // Create LotusClient for this request (not stored due to Rc/RefCell)
        let lotus_client = self.create_client();

        // Generate proof bundle in blocking task (proofs library uses non-Send types)
        let bundle = tokio::task::spawn_blocking(move || {
            // Create a new tokio runtime for the blocking task
            let rt = tokio::runtime::Handle::current();
            rt.block_on(generate_proof_bundle(
                &lotus_client,
                &parent_api,
                &child_api,
                storage_specs,
                event_specs,
            ))
        })
        .await
        .context("Proof generation task panicked")?
        .context("Failed to generate proof bundle")?;

        tracing::info!(
            instance_id = certificate.gpbft_instance,
            highest_epoch,
            storage_proofs = bundle.storage_proofs.len(),
            event_proofs = bundle.event_proofs.len(),
            witness_blocks = bundle.blocks.len(),
            "Generated proof bundle"
        );

        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembler_creation() {
        let assembler = ProofAssembler::new(
            "http://localhost:1234".to_string(),
            1001,
            "test-subnet".to_string(),
        );
        assert!(assembler.is_ok());
    }

    #[test]
    fn test_invalid_url() {
        let assembler =
            ProofAssembler::new("not a url".to_string(), 1001, "test-subnet".to_string());
        assert!(assembler.is_err());
    }
}
