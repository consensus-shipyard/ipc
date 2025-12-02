// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof bundle verification for block attestation
//!
//! Provides deterministic verification of proof bundles against F3 certificates.
//! Used by validators during block attestation to verify parent finality proofs.

use crate::assembler::{NEW_POWER_CHANGE_REQUEST_SIGNATURE, NEW_TOPDOWN_MESSAGE_SIGNATURE};
use anyhow::Result;
use cid::Cid;
use filecoin_f3_certs::FinalityCertificate;
use proofs::proofs::common::bundle::{UnifiedProofBundle, UnifiedVerificationResult};
use proofs::proofs::events::bundle::EventProofBundle;
use proofs::proofs::events::verifier::verify_event_proof;
use proofs::proofs::storage::verifier::verify_storage_proof;

use proofs::proofs::common::evm::{ascii_to_bytes32, extract_evm_log, hash_event_signature};

pub struct ProofsVerifier {
    events: Vec<Vec<[u8; 32]>>,
}

impl ProofsVerifier {
    pub fn new(subnet_id: String) -> Self {
        let events = vec![
            vec![
                hash_event_signature(NEW_TOPDOWN_MESSAGE_SIGNATURE),
                ascii_to_bytes32(&subnet_id),
            ],
            vec![hash_event_signature(NEW_POWER_CHANGE_REQUEST_SIGNATURE)],
        ];

        Self { events }
    }
}

impl ProofsVerifier {
    /// Verify a unified proof bundle against a certificate
    ///
    /// This performs deterministic verification of:
    /// - Storage proofs (contract state at parent height)
    /// - Event proofs (emitted events at parent height)
    ///
    /// # Arguments
    /// * `bundle` - The proof bundle to verify
    /// * `certificate` - The certificate containing finalized epochs
    pub fn verify_proof_bundle(
        &self,
        bundle: &UnifiedProofBundle,
        certificate: &FinalityCertificate,
    ) -> Result<UnifiedVerificationResult> {
        let tipset_verifier = |epoch: i64, cid: &Cid| -> bool {
            certificate
                .ec_chain
                .iter()
                .any(|ts| ts.epoch == epoch && ts.key == cid.to_bytes())
        };

        // Verify storage proofs
        let mut storage_results = Vec::new();
        for proof in &bundle.storage_proofs {
            let result = verify_storage_proof(proof, &bundle.blocks, &tipset_verifier)?;
            storage_results.push(result);
        }

        // Verify event proofs - need to create an EventProofBundle for the verifier
        let event_bundle = EventProofBundle {
            proofs: bundle.event_proofs.clone(),
            blocks: bundle.blocks.clone(),
        };

        let parent_tipset_verifier = |epoch: i64, cids: &[Cid]| -> bool {
            cids.iter().all(|cid| tipset_verifier(epoch, cid))
        };

        let event_results = verify_event_proof(
            &event_bundle,
            &parent_tipset_verifier,
            &tipset_verifier,
            Some(&self.create_event_filter()),
        )?;

        Ok(UnifiedVerificationResult {
            storage_results,
            event_results,
        })
    }

    fn create_event_filter(&self) -> impl Fn(&fvm_shared::event::ActorEvent) -> bool + '_ {
        |ev: &fvm_shared::event::ActorEvent| -> bool {
            if let Some(log) = extract_evm_log(ev) {
                self.events.iter().any(|expected_topics| {
                    log.topics.len() >= expected_topics.len()
                        && expected_topics
                            .iter()
                            .zip(log.topics.iter())
                            .all(|(expected, actual)| expected == actual)
                })
            } else {
                false
            }
        }
    }
}
