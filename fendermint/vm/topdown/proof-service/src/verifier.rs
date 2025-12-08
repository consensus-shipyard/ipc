// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof bundle verification for block attestation
//!
//! Provides deterministic verification of proof bundles against F3 certificates.
//! Used by validators during block attestation to verify parent finality proofs.
//!
//! # Verification Flow
//!
//! The verifier checks that witness blocks in the proof bundle are certified
//! by the F3 certificates. With the two-level cache design, proofs are verified
//! against pre-merged tipsets from both the parent and child certificates.

use crate::assembler::{NEW_POWER_CHANGE_REQUEST_SIGNATURE, NEW_TOPDOWN_MESSAGE_SIGNATURE};
use crate::types::{EpochProofWithCertificate, FinalizedTipsets};
use anyhow::Result;
use cid::Cid;
use proofs::proofs::common::bundle::{UnifiedProofBundle, UnifiedVerificationResult};
use proofs::proofs::events::bundle::EventProofBundle;
use proofs::proofs::events::verifier::verify_event_proof;
use proofs::proofs::storage::verifier::verify_storage_proof;

use proofs::proofs::common::evm::{ascii_to_bytes32, extract_evm_log, hash_event_signature};

pub struct ProofVerifier {
    events: Vec<Vec<[u8; 32]>>,
}

impl ProofVerifier {
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

    /// Verify a inclusion proof in the proof bundle using pre-merged tipsets from certificates
    ///
    /// This is the primary verification method. It verifies that all witness
    /// blocks in the proof bundle are certified by the provided tipsets.
    ///
    /// # Arguments
    /// * `bundle` - The proof bundle to verify
    /// * `merged_tipsets` - Pre-merged tipsets from parent and child certificates
    ///
    /// # Returns
    /// Verification results for storage and event inclusion proofs
    pub fn verify_proof_bundle_with_tipsets(
        &self,
        bundle: &UnifiedProofBundle,
        finalized_tipsets: &FinalizedTipsets,
    ) -> Result<UnifiedVerificationResult> {
        let tipset_verifier = |epoch: i64, cid: &Cid| -> bool {
            finalized_tipsets
                .iter()
                .any(|ts| ts.epoch == epoch && ts.block_cids == cid.to_bytes())
        };

        self.verify_with_verifier(bundle, &tipset_verifier)
    }

    /// Verify a proof bundle from a cache entry
    ///
    /// # Arguments
    /// * `entry` - The epoch proof entry with its certificates
    ///
    /// # Returns
    /// Verification results for storage and event proofs
    pub fn verify_epoch_proof(
        &self,
        entry: &EpochProofWithCertificate,
    ) -> Result<UnifiedVerificationResult> {
        self.verify_proof_bundle_with_tipsets(&entry.proof_bundle, &entry.finalized_tipsets)
    }

    /// Internal verification using a tipset verifier closure
    fn verify_with_verifier<F>(
        &self,
        bundle: &UnifiedProofBundle,
        tipset_verifier: &F,
    ) -> Result<UnifiedVerificationResult>
    where
        F: Fn(i64, &Cid) -> bool,
    {
        // Verify storage proofs
        let mut storage_results = Vec::new();
        for proof in &bundle.storage_proofs {
            let result = verify_storage_proof(proof, &bundle.blocks, tipset_verifier)?;
            storage_results.push(result);
        }

        // Verify event proofs
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
            tipset_verifier,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_creation() {
        let verifier = ProofVerifier::new("test-subnet".to_string());
        assert_eq!(verifier.events.len(), 2);
    }
}
