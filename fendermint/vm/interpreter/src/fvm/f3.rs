// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{bail, Context};
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::GeneralisedTopDown;
use fvm_ipld_blockstore::Blockstore;
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::PowerChangeRequest;
use std::sync::Arc;

use crate::fvm::state::ipc::F3LightClientCaller;
use crate::fvm::state::FvmExecState;

/// F3 finality handler - handles all F3 proof-based finality logic
/// This module encapsulates all F3-specific concerns, keeping TopDownManager clean
#[derive(Clone)]
pub struct F3FinalityHandler {
    /// Proof cache for F3-based parent finality
    proof_cache: Arc<fendermint_vm_topdown_proof_service::ProofCache>,
    /// Subnet ID for ProofVerifier (needed for event filtering)
    subnet_id: String,
    /// F3 Light Client caller for querying F3 state
    f3_light_client_caller: F3LightClientCaller,
}

impl F3FinalityHandler {
    pub fn new(
        proof_cache: Arc<fendermint_vm_topdown_proof_service::ProofCache>,
        subnet_id: String,
    ) -> Self {
        Self {
            proof_cache,
            subnet_id,
            f3_light_client_caller: F3LightClientCaller::new(),
        }
    }

    /// Get reference to the proof cache
    pub fn proof_cache(&self) -> &Arc<fendermint_vm_topdown_proof_service::ProofCache> {
        &self.proof_cache
    }

    /// Check if we have a certificate in our local cache
    /// Used during attestation to avoid redundant F3 validation
    pub fn has_certificate_in_cache(&self, instance_id: u64) -> bool {
        self.proof_cache.contains_certificate(instance_id)
    }

    /// Query proof cache for next uncommitted proof and create a chain message with proof bundle.
    ///
    /// This is the v2 proof-based approach that replaces voting with cryptographic verification.
    ///
    /// Returns `None` if:
    /// - No proof available for next height
    /// - Cache is temporarily empty (graceful degradation)
    pub fn chain_message_from_proof_cache(&self) -> Option<ChainMessage> {
        // Get next uncommitted proof (epoch after last_committed)
        let entry = self.proof_cache.get_next_uncommitted()?;

        // Convert FinalityCertificate to get finalized epochs
        let finalized_epochs: Vec<fvm_shared::clock::ChainEpoch> = entry
            .certificate
            .ec_chain
            .iter()
            .map(|ts| ts.epoch)
            .collect();

        tracing::debug!(
            instance_id = entry.certificate.gpbft_instance,
            epoch = entry.epoch,
            epochs = ?finalized_epochs,
            "found proof in cache for proposal"
        );

        // Convert FinalityCertificate to SerializableF3Certificate for message
        let serializable_cert =
            fendermint_vm_topdown_proof_service::types::SerializableF3Certificate::from(
                &entry.certificate,
            );

        Some(ChainMessage::Ipc(
            fendermint_vm_message::ipc::IpcMessage::GeneralisedTopDown(GeneralisedTopDown {
                height: entry.epoch,
                certificate: fendermint_vm_message::ipc::Certificate::FilecoinF3(serializable_cert),
            }),
        ))
    }

    /// Attest a generalised top-down message during the attestation phase.
    ///
    /// This checks the certificate validity:
    /// 1. Get proof bundle from local cache using certificate instance ID
    /// 2. Check if certificate is in local cache (if yes, we're done - already validated)
    /// 3. If not in cache, verify certificate with F3 client and verify proof bundle
    ///
    /// All correct validators will reach the same decision (deterministic).
    /// Attestation must complete here, not defer to execution phase.
    pub async fn attest(&self, msg: &GeneralisedTopDown) -> anyhow::Result<()> {
        // Extract certificate from message
        let certificate = match &msg.certificate {
            fendermint_vm_message::ipc::Certificate::FilecoinF3(cert) => cert,
        };

        // Get proof bundle from local cache using certificate instance ID
        let proof_bundle = {
            // Get the epoch proof entry from cache
            let entry = self
                .proof_cache
                .get_epoch_proof_with_certificate(msg.height)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "proof bundle not found in local cache for height {}",
                        msg.height
                    )
                })?;

            // Verify the certificate instance matches
            if entry.certificate.gpbft_instance != certificate.gpbft_instance {
                bail!(
                    "Certificate instance mismatch: message has {}, cache has {}",
                    certificate.gpbft_instance,
                    entry.certificate.gpbft_instance
                );
            }

            entry.proof_bundle.clone()
        };

        // STEP 1: Check if certificate is in local cache
        let instance_id = certificate.gpbft_instance;
        if self.proof_cache.contains_certificate(instance_id) {
            tracing::debug!(
                instance = instance_id,
                "Certificate found in local cache - already validated by our F3 client"
            );
            return Ok(());
        }

        // STEP 2: Certificate not in cache - need to verify
        // Convert SerializableF3Certificate to FinalityCertificate for verification
        let finality_cert = certificate
            .clone()
            .try_into_certificate()
            .context("failed to convert SerializableF3Certificate to FinalityCertificate")?;

        // Create EpochProofWithCertificate for verification
        use fendermint_vm_topdown_proof_service::types::FinalizedTipsets;
        let finalized_tipsets = FinalizedTipsets::from(&finality_cert.ec_chain);

        let epoch_proof = fendermint_vm_topdown_proof_service::types::EpochProofWithCertificate {
            epoch: msg.height,
            proof_bundle: proof_bundle.clone(),
            certificate: finality_cert.clone(),
            finalized_tipsets,
        };

        // STEP 3: Verify proof bundle using ProofVerifier
        use fendermint_vm_topdown_proof_service::verifier::ProofVerifier;
        let verifier = ProofVerifier::new(self.subnet_id.clone());
        verifier
            .verify_epoch_proof(&epoch_proof)
            .context("proof bundle verification failed")?;

        tracing::debug!(
            instance = certificate.gpbft_instance,
            height = msg.height,
            "Proof bundle verified successfully (certificate not in cache)"
        );

        // Note: Full F3 certificate chain continuity validation happens during execution
        // when we have state access to query F3LightClientActor

        Ok(())
    }

    /// Get proof bundle for a given height from cache
    pub fn get_proof_bundle(
        &self,
        height: fvm_shared::clock::ChainEpoch,
        expected_instance_id: u64,
    ) -> anyhow::Result<proofs::proofs::common::bundle::UnifiedProofBundle> {
        let entry = self
            .proof_cache
            .get_epoch_proof_with_certificate(height)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "proof bundle not found in local cache for height {}",
                    height
                )
            })?;

        // Verify the certificate instance matches
        if entry.certificate.gpbft_instance != expected_instance_id {
            bail!(
                "Certificate instance mismatch: expected {}, cache has {}",
                expected_instance_id,
                entry.certificate.gpbft_instance
            );
        }

        Ok(entry.proof_bundle)
    }

    /// Extract topdown messages from proof bundle
    pub fn extract_topdown_messages(
        &self,
        proof_bundle: &proofs::proofs::common::bundle::UnifiedProofBundle,
    ) -> anyhow::Result<Vec<ipc_api::cross::IpcEnvelope>> {
        crate::fvm::event_extraction::extract_topdown_messages(proof_bundle)
    }

    /// Extract validator changes from proof bundle
    pub fn extract_validator_changes(
        &self,
        proof_bundle: &proofs::proofs::common::bundle::UnifiedProofBundle,
    ) -> anyhow::Result<Vec<ipc_api::staking::PowerChangeRequest>> {
        crate::fvm::event_extraction::extract_validator_changes(proof_bundle)
    }

    /// Get power table for a certificate instance from cache
    pub fn get_power_table(
        &self,
        instance_id: u64,
    ) -> Vec<fendermint_vm_actor_interface::f3_light_client::PowerEntry> {
        if let Some(cert_entry) = self.proof_cache.get_certificate(instance_id) {
            cert_entry
                .power_table
                .iter()
                .map(|pe| {
                    // Convert BigInt power to u64 (saturating if too large)
                    let (_sign, digits) = pe.power.to_u64_digits();
                    let power_u64 = if digits.is_empty() {
                        0
                    } else if digits.len() == 1 {
                        digits[0]
                    } else {
                        u64::MAX // Too large, saturate
                    };
                    fendermint_vm_actor_interface::f3_light_client::PowerEntry {
                        public_key: pe.pub_key.0.clone(),
                        power: power_u64,
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Mark epoch as committed in cache
    pub fn mark_committed(
        &self,
        epoch: fvm_shared::clock::ChainEpoch,
        instance_id: u64,
    ) -> anyhow::Result<()> {
        self.proof_cache
            .mark_committed(epoch, instance_id)
            .map_err(|e| {
                anyhow::anyhow!(
                    "failed to mark epoch {} as committed in cache: {}",
                    epoch,
                    e
                )
            })
    }

    /// Get F3 light client state
    pub fn get_f3_state<DB>(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<fendermint_vm_actor_interface::f3_light_client::GetStateResponse>
    where
        DB: Blockstore + Clone + 'static + Send + Sync,
    {
        self.f3_light_client_caller
            .get_state(state)
            .context("failed to get F3 light client state")
    }

    /// Update F3 light client state
    pub fn update_f3_state<DB>(
        &self,
        state: &mut FvmExecState<DB>,
        new_state: fendermint_vm_actor_interface::f3_light_client::LightClientState,
    ) -> anyhow::Result<()>
    where
        DB: Blockstore + Clone + 'static + Send + Sync,
    {
        self.f3_light_client_caller
            .update_state(state, new_state)
            .context("failed to update F3LightClientActor state")
    }

    /// Execute F3-specific logic for a generalised top-down message.
    /// Returns the topdown messages and validator changes to be processed by TopDownManager.
    pub fn execute<DB>(
        &self,
        state: &mut FvmExecState<DB>,
        msg: &GeneralisedTopDown,
    ) -> anyhow::Result<(Vec<IpcEnvelope>, Vec<PowerChangeRequest>)>
    where
        DB: Blockstore + Clone + 'static + Send + Sync,
    {
        // Extract certificate from message
        let certificate = match &msg.certificate {
            fendermint_vm_message::ipc::Certificate::FilecoinF3(cert) => cert,
        };

        tracing::debug!(
            instance = certificate.gpbft_instance,
            height = msg.height,
            "executing F3 generalised top-down"
        );

        // Step 1: Verify certificate chain continuity (check against F3LightClientActor state)
        let f3_state = self.get_f3_state(state)?;

        // Ensure certificate instance is sequential
        if certificate.gpbft_instance != f3_state.latest_instance_id + 1 {
            bail!(
                "Certificate instance ID {} is not sequential (expected {})",
                certificate.gpbft_instance,
                f3_state.latest_instance_id + 1
            );
        }

        // Step 2: Get proof bundle and extract topdown effects
        let proof_bundle = self.get_proof_bundle(msg.height, certificate.gpbft_instance)?;
        let msgs = self.extract_topdown_messages(&proof_bundle)?;
        let validator_changes = self.extract_validator_changes(&proof_bundle)?;

        tracing::debug!(
            message_count = msgs.len(),
            validator_changes_count = validator_changes.len(),
            "extracted topdown effects from proof bundle"
        );

        // Step 3: Update F3LightClientActor with new certificate state
        let power_table = self.get_power_table(certificate.gpbft_instance);
        let latest_finalized_height = Some(
            certificate
                .finalized_epochs()
                .iter()
                .max()
                .copied()
                .context("certificate has no finalized epochs")?,
        );

        let new_light_client_state =
            fendermint_vm_actor_interface::f3_light_client::LightClientState {
                latest_instance_id: certificate.gpbft_instance,
                latest_finalized_height,
                power_table,
            };

        self.update_f3_state(state, new_light_client_state)?;

        tracing::debug!(
            instance = certificate.gpbft_instance,
            "updated F3LightClientActor state"
        );

        // Step 4: Mark epoch as committed in cache
        if let Err(e) = self.mark_committed(msg.height, certificate.gpbft_instance) {
            tracing::warn!(
                error = %e,
                epoch = msg.height,
                instance = certificate.gpbft_instance,
                "failed to mark epoch as committed in cache"
            );
        } else {
            tracing::debug!(
                epoch = msg.height,
                instance = certificate.gpbft_instance,
                "marked epoch as committed in cache"
            );
        }

        Ok((msgs, validator_changes))
    }
}
