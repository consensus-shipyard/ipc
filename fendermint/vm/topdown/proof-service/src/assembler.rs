// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof bundle assembler

use crate::types::{CacheEntry, ProofBundlePlaceholder};
use anyhow::{Context, Result};
use cid::Cid;
use fendermint_actor_f3_cert_manager::types::F3Certificate;
use fvm_shared::clock::ChainEpoch;
use ipc_provider::lotus::message::f3::F3CertificateResponse;
use std::str::FromStr;
use std::time::SystemTime;

/// Assembles proof bundles from F3 certificates and parent chain data
pub struct ProofAssembler {
    /// Source RPC URL (for metadata)
    source_rpc: String,
}

impl ProofAssembler {
    /// Create a new proof assembler
    pub fn new(source_rpc: String) -> Self {
        Self { source_rpc }
    }

    /// Assemble a complete proof bundle for an F3 certificate
    ///
    /// This will eventually:
    /// 1. Extract tipsets from ECChain
    /// 2. Call ipc-filecoin-proofs::generate_proof_bundle()
    /// 3. Build complete CacheEntry
    ///
    /// For now, we create a placeholder bundle
    pub async fn assemble_proof(&self, lotus_cert: &F3CertificateResponse) -> Result<CacheEntry> {
        tracing::debug!(
            instance_id = lotus_cert.gpbft_instance,
            "Assembling proof bundle"
        );

        // Extract finalized epochs from ECChain
        let finalized_epochs: Vec<ChainEpoch> = lotus_cert
            .ec_chain
            .iter()
            .map(|entry| entry.epoch)
            .collect();

        if finalized_epochs.is_empty() {
            anyhow::bail!("F3 certificate has empty ECChain");
        }

        tracing::debug!(
            instance_id = lotus_cert.gpbft_instance,
            epochs = ?finalized_epochs,
            "Extracted epochs from certificate"
        );

        // Convert Lotus certificate to actor format
        let actor_cert = self.convert_lotus_to_actor_cert(lotus_cert)?;

        // TODO: Generate actual proof bundle using ipc-filecoin-proofs
        // For now, create a placeholder
        let highest_epoch = finalized_epochs.iter().max().copied().unwrap();
        let bundle = ProofBundlePlaceholder {
            parent_height: highest_epoch as u64,
            data: vec![],
        };

        let entry = CacheEntry {
            instance_id: lotus_cert.gpbft_instance,
            finalized_epochs,
            bundle,
            actor_certificate: actor_cert,
            generated_at: SystemTime::now(),
            source_rpc: self.source_rpc.clone(),
        };

        tracing::info!(
            instance_id = entry.instance_id,
            epochs_count = entry.finalized_epochs.len(),
            "Assembled proof bundle"
        );

        Ok(entry)
    }

    /// Convert Lotus F3 certificate to actor certificate format
    fn convert_lotus_to_actor_cert(
        &self,
        lotus_cert: &F3CertificateResponse,
    ) -> Result<F3Certificate> {
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
        // CIDMap.cid is Option<String>, need to parse it
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
        // This preserves the entire ECChain for verification
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
    use cid::Cid;
    use ipc_provider::lotus::message::f3::{ECChainEntry, F3CertificateResponse, SupplementalData};
    use ipc_provider::lotus::message::CIDMap;
    use multihash_codetable::{Code, MultihashDigest};

    fn create_test_lotus_cert(instance: u64, epochs: Vec<i64>) -> F3CertificateResponse {
        let power_table_cid = Cid::new_v1(0x55, Code::Blake2b256.digest(b"test"));
        let cid_map = CIDMap {
            cid: Some(power_table_cid.to_string()),
        };

        let ec_chain: Vec<ECChainEntry> = epochs
            .into_iter()
            .map(|epoch| ECChainEntry {
                key: vec![],
                epoch,
                power_table: cid_map.clone(),
                commitments: String::new(),
            })
            .collect();

        F3CertificateResponse {
            gpbft_instance: instance,
            ec_chain,
            supplemental_data: SupplementalData {
                commitments: String::new(),
                power_table: cid_map,
            },
            signers: vec![],
            signature: {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(b"test_signature")
            },
        }
    }

    #[tokio::test]
    async fn test_assemble_proof() {
        let assembler = ProofAssembler::new("http://test".to_string());

        let lotus_cert = create_test_lotus_cert(100, vec![500, 501, 502, 503]);

        let result = assembler.assemble_proof(&lotus_cert).await;
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.instance_id, 100);
        assert_eq!(entry.finalized_epochs, vec![500, 501, 502, 503]);
        assert_eq!(entry.highest_epoch(), Some(503));
        assert_eq!(entry.actor_certificate.instance_id, 100);
    }

    #[tokio::test]
    async fn test_assemble_proof_empty_ec_chain() {
        let assembler = ProofAssembler::new("http://test".to_string());

        let lotus_cert = create_test_lotus_cert(100, vec![]);

        let result = assembler.assemble_proof(&lotus_cert).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_lotus_to_actor_cert() {
        let assembler = ProofAssembler::new("http://test".to_string());

        let lotus_cert = create_test_lotus_cert(42, vec![100, 101, 102]);

        let result = assembler.convert_lotus_to_actor_cert(&lotus_cert);
        assert!(result.is_ok());

        let actor_cert = result.unwrap();
        assert_eq!(actor_cert.instance_id, 42);
        assert_eq!(actor_cert.finalized_epochs, vec![100, 101, 102]);
        assert!(!actor_cert.signature.is_empty());
        assert!(!actor_cert.certificate_data.is_empty());
    }
}
