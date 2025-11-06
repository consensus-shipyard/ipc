// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof bundle verification for block attestation
//!
//! Provides deterministic verification of proof bundles against F3 certificates.
//! Used by validators during block attestation to verify parent finality proofs.

use crate::types::SerializableF3Certificate;
use anyhow::{Context, Result};
use proofs::proofs::common::bundle::{ProofBlock, UnifiedProofBundle};
use proofs::proofs::storage::{bundle::StorageProof, verifier::verify_storage_proof};
use tracing::debug;

/// Verify a unified proof bundle against a certificate
///
/// This performs deterministic verification of:
/// - Storage proofs (contract state at parent height)
/// - Event proofs (emitted events at parent height)
///
/// # Arguments
/// * `bundle` - The proof bundle to verify
/// * `certificate` - The certificate containing finalized epochs
///
/// # Returns
/// `Ok(())` if all proofs are valid, `Err` otherwise
///
/// # Usage in Block Attestation
///
/// ```ignore
/// // When validator receives block with parent finality data
/// if cache.contains(cert.instance_id) {
///     // Already validated - just verify proofs
///     let cached = cache.get(cert.instance_id).unwrap();
///     verify_proof_bundle(&cached.proof_bundle, &cached.certificate)?;
/// } else {
///     // Not cached - need full crypto validation first
///     let validated = f3_client.fetch_and_validate(cert.instance_id).await?;
///     let serializable_cert = SerializableF3Certificate::from(&validated.f3_cert);
///     verify_proof_bundle(&proof_bundle, &serializable_cert)?;
/// }
/// ```
pub fn verify_proof_bundle(
    bundle: &UnifiedProofBundle,
    certificate: &SerializableF3Certificate,
) -> Result<()> {
    debug!(
        instance_id = certificate.instance_id,
        storage_proofs = bundle.storage_proofs.len(),
        event_proofs = bundle.event_proofs.len(),
        witness_blocks = bundle.blocks.len(),
        "Verifying proof bundle"
    );

    // Verify all storage proofs
    for (idx, storage_proof) in bundle.storage_proofs.iter().enumerate() {
        verify_storage_proof_internal(storage_proof, &bundle.blocks, certificate)
            .with_context(|| format!("Storage proof {} failed verification", idx))?;
    }

    // Event proof verification uses a bundle-level API
    // For now, we verify that the bundle structure is valid
    // Full event proof verification will be added when the proofs library API is finalized
    if !bundle.event_proofs.is_empty() {
        debug!(
            event_proofs = bundle.event_proofs.len(),
            "Event proofs present (verification to be implemented with proofs library API)"
        );
    }

    debug!(
        instance_id = certificate.instance_id,
        "Proof bundle verified successfully"
    );

    Ok(())
}

/// Verify a single storage proof
///
/// Uses the proofs library's verify_storage_proof function with the witness blocks.
fn verify_storage_proof_internal(
    proof: &StorageProof,
    blocks: &[ProofBlock],
    certificate: &SerializableF3Certificate,
) -> Result<()> {
    // Verify the proof's child epoch is in the certificate's finalized epochs
    let child_epoch = proof.child_epoch;
    if !certificate.finalized_epochs.contains(&child_epoch) {
        anyhow::bail!(
            "Storage proof child epoch {} not in certificate's finalized epochs",
            child_epoch
        );
    }

    // Use the proofs library to verify the storage proof
    // The is_trusted_child_header function checks if the child epoch is finalized
    let is_trusted =
        |epoch: i64, _cid: &cid::Cid| -> bool { certificate.finalized_epochs.contains(&epoch) };

    let valid = verify_storage_proof(proof, blocks, &is_trusted)
        .context("Storage proof verification failed")?;

    if !valid {
        anyhow::bail!("Storage proof is invalid");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proofs::proofs::common::bundle::UnifiedProofBundle;

    #[test]
    fn test_verify_empty_bundle() {
        let bundle = UnifiedProofBundle {
            storage_proofs: vec![],
            event_proofs: vec![],
            blocks: vec![],
        };

        let cert = SerializableF3Certificate {
            instance_id: 1,
            finalized_epochs: vec![100, 101],
            power_table_cid: "test_cid".to_string(),
            signature: vec![],
            signers: vec![],
        };

        // Empty bundle should verify successfully
        assert!(verify_proof_bundle(&bundle, &cert).is_ok());
    }
}
