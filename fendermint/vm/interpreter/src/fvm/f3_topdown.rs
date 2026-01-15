// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{bail, Context};
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::GeneralisedTopDown;
use fvm_ipld_blockstore::Blockstore;
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::PowerChangeRequest;
use std::sync::Arc;

use fendermint_vm_message::ipc::IpcMessage;
use fendermint_vm_topdown_proof_service::types::SerializableF3Certificate;

use crate::fvm::state::ipc::F3LightClientCaller;
use crate::fvm::state::FvmExecState;

/// F3 finality handler - handles all F3 proof-based finality logic
/// This module encapsulates all F3-specific concerns, keeping TopDownManager clean
#[derive(Clone)]
pub struct F3TopDownHandler {
    /// Proof cache for F3-based parent finality
    proof_cache: Arc<fendermint_vm_topdown_proof_service::ProofCache>,
    /// F3 Light Client caller for querying F3 state
    f3_light_client_caller: F3LightClientCaller,
}

impl F3TopDownHandler {
    pub fn new(proof_cache: Arc<fendermint_vm_topdown_proof_service::ProofCache>) -> Self {
        Self {
            proof_cache,
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
        let epoch_with_cert = self.proof_cache.get_next_uncommitted_epoch_with_cert()?;

        tracing::debug!(
            instance_id = epoch_with_cert.certificate.gpbft_instance,
            epoch = epoch_with_cert.epoch,
            "found next uncommitted epoch with certificate in cache"
        );

        // Convert FinalityCertificate to SerializableF3Certificate for message
        let serializable_cert = SerializableF3Certificate::from(&epoch_with_cert.certificate);

        Some(ChainMessage::Ipc(IpcMessage::GeneralisedTopDown(
            GeneralisedTopDown {
                height: epoch_with_cert.epoch,
                certificate: fendermint_vm_message::ipc::Certificate::FilecoinF3(serializable_cert),
            },
        )))
    }

    /// Attest a generalised top-down message during the attestation phase.
    ///
    /// Cache-first attestation.
    ///
    /// We require that:
    /// - there is an epoch proof in the local cache for `msg.height`
    /// - the certificate attached to the message matches the certificate referenced by the cache entry
    ///
    /// Proof bundle validity is verified at proof generation time (before insertion into the cache).
    pub async fn attest<DB>(
        &self,
        state: &mut FvmExecState<DB>,
        msg: &GeneralisedTopDown,
    ) -> anyhow::Result<()>
    where
        DB: Blockstore + Clone + 'static + Send + Sync,
    {
        let msg_cert = match &msg.certificate {
            fendermint_vm_message::ipc::Certificate::FilecoinF3(cert) => cert,
        };

        let cached = self
            .proof_cache
            .get_epoch_proof_with_certificate(msg.height)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "proof bundle not found in local cache for height {}",
                    msg.height
                )
            })?;

        if cached.epoch != msg.height {
            bail!(
                "epoch mismatch: message has {}, cache entry has {}",
                msg.height,
                cached.epoch
            );
        }

        let cached_cert = SerializableF3Certificate::from(&cached.certificate);
        if &cached_cert != msg_cert {
            bail!(
                "certificate mismatch for epoch {} (message instance {}, cache instance {})",
                msg.height,
                msg_cert.gpbft_instance,
                cached_cert.gpbft_instance
            );
        }

        // Check on-chain continuity (this needs state access, hence in attestation).
        let f3_state = self.get_f3_state(state)?;
        let instance_id = cached.certificate.gpbft_instance;

        // Certificate instance must not go backwards; it either stays the same (multiple epochs can
        // be proven under the same certificate) or advances by exactly 1.
        if instance_id < f3_state.latest_instance_id {
            bail!(
                "certificate instance went backwards: {} < {}",
                instance_id,
                f3_state.latest_instance_id
            );
        }
        if instance_id > f3_state.latest_instance_id + 1 {
            bail!(
                "certificate instance jumped: {} > {}",
                instance_id,
                f3_state.latest_instance_id + 1
            );
        }

        // Epoch must advance by exactly 1 relative to the latest finalized epoch in state.
        // At genesis this is `None`; treat it as 0 baseline.
        let prev_finalized = f3_state.latest_finalized_height.unwrap_or(0);
        if msg.height != prev_finalized + 1 {
            bail!(
                "epoch is not sequential: message height {} != expected {}",
                msg.height,
                prev_finalized + 1
            );
        }

        Ok(())
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
    pub fn extract_messages_and_validator_changes<DB>(
        &self,
        _state: &mut FvmExecState<DB>,
        msg: &GeneralisedTopDown,
    ) -> anyhow::Result<(Vec<IpcEnvelope>, Vec<PowerChangeRequest>, u64)>
    where
        DB: Blockstore + Clone + 'static + Send + Sync,
    {
        // Cache is the source of truth: get the proof + certificate for this epoch.
        let cached = self
            .proof_cache
            .get_epoch_proof_with_certificate(msg.height)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "proof bundle not found in local cache for height {}",
                    msg.height
                )
            })?;

        // We don't validate the message certificate here; that happens during attestation.
        let instance_id = cached.certificate.gpbft_instance;

        tracing::debug!(
            instance = instance_id,
            height = msg.height,
            "executing F3 generalised top-down"
        );

        let msgs = self.extract_topdown_messages(&cached.proof_bundle)?;
        let validator_changes = self.extract_validator_changes(&cached.proof_bundle)?;

        tracing::debug!(
            message_count = msgs.len(),
            validator_changes_count = validator_changes.len(),
            "extracted topdown effects from proof bundle"
        );

        Ok((msgs, validator_changes, instance_id))
    }

    /// Finalize F3 execution after all top-down effects have been applied successfully.
    ///
    /// This updates the on-chain F3 light client state and marks the epoch as committed in the proof cache.
    pub fn finalize_after_execution<DB>(
        &self,
        state: &mut FvmExecState<DB>,
        epoch: fvm_shared::clock::ChainEpoch,
        instance_id: u64,
    ) -> anyhow::Result<()>
    where
        DB: Blockstore + Clone + 'static + Send + Sync,
    {
        // Update F3LightClientActor with new certificate state.
        let power_table = self.get_power_table(instance_id);
        let new_light_client_state =
            fendermint_vm_actor_interface::f3_light_client::LightClientState {
                latest_instance_id: instance_id,
                latest_finalized_height: Some(epoch),
                power_table,
            };

        self.update_f3_state(state, new_light_client_state)?;
        tracing::debug!(instance = instance_id, "updated F3LightClientActor state");

        // Mark epoch as committed in cache.
        if let Err(e) = self.mark_committed(epoch, instance_id) {
            tracing::warn!(
                error = %e,
                epoch,
                instance = instance_id,
                "failed to mark epoch as committed in cache"
            );
        } else {
            tracing::debug!(
                epoch,
                instance = instance_id,
                "marked epoch as committed in cache"
            );
        }

        Ok(())
    }
}
