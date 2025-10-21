// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof bundle assembler

use crate::types::{CacheEntry, ValidatedCertificate};
use anyhow::{Context, Result};
use fendermint_actor_f3_light_client::types::F3Certificate;
use fvm_shared::clock::ChainEpoch;
use proofs::{
    client::LotusClient,
    proofs::{calculate_storage_slot, generate_proof_bundle, EventProofSpec, StorageProofSpec},
};
use serde_json::json;
use std::time::SystemTime;
use url::Url;

/// Assembles proof bundles from F3 certificates and parent chain data
pub struct ProofAssembler {
    rpc_url: String,
    gateway_actor_id: u64,
    subnet_id: String,
}

impl ProofAssembler {
    /// Create a new proof assembler
    pub fn new(rpc_url: String, gateway_actor_id: u64, subnet_id: String) -> Result<Self> {
        // Validate URL
        let _ = Url::parse(&rpc_url)?;
        Ok(Self {
            rpc_url,
            gateway_actor_id,
            subnet_id,
        })
    }
    
    /// Create a Lotus client for requests
    fn create_client(&self) -> Result<LotusClient> {
        Ok(LotusClient::new(Url::parse(&self.rpc_url)?, None))
    }

    /// Generate proof for a specific epoch
    pub async fn generate_proof_for_epoch(&self, epoch: i64) -> Result<Vec<u8>> {
        tracing::debug!(epoch, "Generating proof for epoch");
        
        // Create client for this request
        let lotus_client = self.create_client()?;

        // Fetch tipsets
        let parent = lotus_client
            .request("Filecoin.ChainGetTipSetByHeight", json!([epoch, null]))
            .await
            .context("Failed to fetch parent tipset")?;

        let child = lotus_client
            .request("Filecoin.ChainGetTipSetByHeight", json!([epoch + 1, null]))
            .await
            .context("Failed to fetch child tipset")?;

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
            epoch,
            storage_specs_count = storage_specs.len(),
            event_specs_count = event_specs.len(),
            "Configured proof specs"
        );

        // Generate proof bundle
        let bundle = generate_proof_bundle(
            &lotus_client,
            &parent,
            &child,
            storage_specs,
            event_specs,
        )
        .await
        .context("Failed to generate proof bundle")?;

        // Serialize the bundle to bytes
        let bundle_bytes =
            fvm_ipld_encoding::to_vec(&bundle).context("Failed to serialize proof bundle")?;

        tracing::info!(
            epoch,
            bundle_size = bundle_bytes.len(),
            "Generated proof bundle"
        );

        Ok(bundle_bytes)
    }

    /// Create a cache entry from a validated certificate
    pub async fn create_cache_entry_for_certificate(
        &self,
        validated: &ValidatedCertificate,
    ) -> Result<CacheEntry> {
        // Extract epochs from certificate
        let finalized_epochs: Vec<ChainEpoch> = validated
            .lotus_response
            .ec_chain
            .iter()
            .map(|entry| entry.epoch)
            .collect();

        if finalized_epochs.is_empty() {
            anyhow::bail!("Certificate has empty ECChain");
        }

        let highest_epoch = *finalized_epochs
            .iter()
            .max()
            .context("No epochs in certificate")?;

        tracing::debug!(
            instance_id = validated.instance_id,
            highest_epoch,
            epochs_count = finalized_epochs.len(),
            "Processing certificate for proof generation"
        );

        // Generate proof bundle for the highest epoch
        let proof_bundle_bytes = self
            .generate_proof_for_epoch(highest_epoch)
            .await
            .context("Failed to generate proof for epoch")?;

        // For MVP, we'll store empty bytes since F3Certificate doesn't implement Serialize
        // In production, we'd store the raw certificate data
        let f3_certificate_bytes = vec![];

        // Convert to actor certificate format
        let actor_cert = self.convert_to_actor_cert(&validated.lotus_response)?;

        Ok(CacheEntry {
            instance_id: validated.instance_id,
            finalized_epochs,
            proof_bundle_bytes,
            f3_certificate_bytes,
            actor_certificate: actor_cert,
            generated_at: SystemTime::now(),
            source_rpc: self.rpc_url.clone(),
        })
    }

    /// Convert Lotus F3 certificate to actor certificate format
    fn convert_to_actor_cert(
        &self,
        lotus_cert: &ipc_provider::lotus::message::f3::F3CertificateResponse,
    ) -> Result<F3Certificate> {
        use cid::Cid;
        use std::str::FromStr;

        // Extract all epochs from ECChain
        let finalized_epochs: Vec<ChainEpoch> = lotus_cert
            .ec_chain
            .iter()
            .map(|entry| entry.epoch)
            .collect();

        if finalized_epochs.is_empty() {
            anyhow::bail!("Empty ECChain in certificate");
        }

        // Power table CID from last entry in ECChain
        let power_table_cid_str = lotus_cert
            .ec_chain
            .last()
            .context("Empty ECChain")?
            .power_table
            .cid
            .as_ref()
            .context("PowerTable CID is None")?;

        let power_table_cid =
            Cid::from_str(power_table_cid_str).context("Failed to parse power table CID")?;

        // Decode signature from base64
        use base64::Engine;
        let signature = base64::engine::general_purpose::STANDARD
            .decode(&lotus_cert.signature)
            .context("Failed to decode certificate signature")?;

        // Encode full Lotus certificate as CBOR
        let certificate_data =
            fvm_ipld_encoding::to_vec(lotus_cert).context("Failed to encode certificate data")?;

        Ok(F3Certificate {
            instance_id: lotus_cert.gpbft_instance,
            finalized_epochs,
            power_table_cid,
            signature,
            certificate_data,
        })
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
}
