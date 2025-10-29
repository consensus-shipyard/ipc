// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use async_stm::atomically;
use fendermint_tracing::emit;
use fendermint_vm_event::ParentFinalityMissingQuorum;
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::IpcMessage;
use fendermint_vm_message::ipc::ParentFinality;
use fendermint_vm_topdown::proxy::IPCProviderProxyWithLatency;
use fendermint_vm_topdown::voting::ValidatorKey;
use fendermint_vm_topdown::voting::VoteTally;
use fendermint_vm_topdown::{
    BlockHeight, CachedFinalityProvider, IPCParentFinality, ParentFinalityProvider,
    ParentViewProvider, Toggle,
};
use fvm_shared::clock::ChainEpoch;
use std::sync::Arc;

use crate::fvm::state::ipc::{F3LightClientCaller, GatewayCaller};
use crate::fvm::state::FvmExecState;
use anyhow::{bail, Context};
use fvm_ipld_blockstore::Blockstore;

use crate::fvm::end_block_hook::PowerUpdates;
use crate::fvm::state::ipc::tokens_to_mint;
use crate::types::AppliedMessage;
use ipc_api::cross::IpcEnvelope;

type TopDownFinalityProvider = Arc<Toggle<CachedFinalityProvider<IPCProviderProxyWithLatency>>>;

#[derive(Clone)]
pub struct TopDownManager<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    provider: TopDownFinalityProvider,
    votes: VoteTally,
    // Gateway caller for IPC gateway interactions
    gateway_caller: GatewayCaller<DB>,
    // F3 Light Client caller for querying F3 state
    f3_light_client_caller: F3LightClientCaller,
    // Proof cache for F3-based parent finality (optional for gradual rollout)
    proof_cache: Option<std::sync::Arc<fendermint_vm_topdown_proof_service::ProofCache>>,
}

impl<DB> TopDownManager<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    pub fn new(
        provider: TopDownFinalityProvider,
        votes: VoteTally,
        proof_cache: Option<std::sync::Arc<fendermint_vm_topdown_proof_service::ProofCache>>,
    ) -> Self {
        Self {
            provider,
            votes,
            gateway_caller: GatewayCaller::default(),
            f3_light_client_caller: F3LightClientCaller::default(),
            proof_cache,
        }
    }

    pub async fn is_finality_valid(&self, finality: ParentFinality) -> bool {
        let prop = IPCParentFinality {
            height: finality.height as u64,
            block_hash: finality.block_hash,
        };
        atomically(|| self.provider.check_proposal(&prop)).await
    }

    /// Prepares a top-down execution message based on the current parent's finality proposal and quorum.
    ///
    /// This function first pauses incoming votes to prevent interference during processing. It then atomically retrieves
    /// both the next parent's proposal and the quorum of votes. If either the parent's proposal or the quorum is missing,
    /// the function returns `None`. When both are available, it selects the finality with the lower block height and wraps
    /// it into a `ChainMessage` for top-down execution.
    pub async fn chain_message_from_finality_or_quorum(&self) -> Option<ChainMessage> {
        // Prepare top down proposals.
        // Before we try to find a quorum, pause incoming votes. This is optional but if there are lots of votes coming in it might hold up proposals.
        atomically(|| self.votes.pause_votes_until_find_quorum()).await;

        // The pre-requisite for proposal is that there is a quorum of gossiped votes at that height.
        // The final proposal can be at most as high as the quorum, but can be less if we have already,
        // hit some limits such as how many blocks we can propose in a single step.
        let (parent, quorum) = atomically(|| {
            let parent = self.provider.next_proposal()?;

            let quorum = self
                .votes
                .find_quorum()?
                .map(|(height, block_hash)| IPCParentFinality { height, block_hash });

            Ok((parent, quorum))
        })
        .await;

        // If there is no parent proposal, exit early.
        let parent = parent?;

        // Require a quorum; if it's missing, log and exit.
        let quorum = if let Some(quorum) = quorum {
            quorum
        } else {
            emit!(
                DEBUG,
                ParentFinalityMissingQuorum {
                    block_height: parent.height,
                    block_hash: &hex::encode(&parent.block_hash),
                }
            );
            return None;
        };

        // Choose the lower height between the parent's proposal and the quorum.
        let finality = if parent.height <= quorum.height {
            parent
        } else {
            quorum
        };

        Some(ChainMessage::Ipc(IpcMessage::TopDownExec(ParentFinality {
            height: finality.height as ChainEpoch,
            block_hash: finality.block_hash,
        })))
    }

    /// Query proof cache for next uncommitted proof and create a chain message with proof bundle.
    ///
    /// This is the v2 proof-based approach that replaces voting with cryptographic verification.
    ///
    /// Returns `None` if:
    /// - Proof cache is not configured
    /// - No proof available for next height
    /// - Cache is temporarily empty (graceful degradation)
    pub async fn chain_message_from_proof_cache(&self) -> Option<ChainMessage> {
        let cache = self.proof_cache.as_ref()?;

        // Get next uncommitted proof (instance after last_committed)
        let entry = cache.get_next_uncommitted()?;

        tracing::debug!(
            instance_id = entry.instance_id,
            epochs = ?entry.finalized_epochs,
            "found proof in cache for proposal"
        );

        Some(ChainMessage::Ipc(IpcMessage::TopDownWithProof(
            fendermint_vm_message::ipc::TopDownProofBundle {
                certificate: entry.certificate,
                proof_bundle: entry.proof_bundle,
            },
        )))
    }

    /// Deterministically verify a proof bundle against F3 certificate (read-only attestation).
    ///
    /// This performs cryptographic verification of:
    /// 1. Storage proofs (contract state at parent height - completeness via topDownNonce)
    /// 2. Event proofs (emitted events at parent height)
    ///
    /// All correct validators will reach the same decision (deterministic).
    /// Full verification including state queries happens during execution.
    pub fn verify_proof_bundle_attestation(
        &self,
        bundle: &fendermint_vm_message::ipc::TopDownProofBundle,
    ) -> anyhow::Result<()> {
        use fendermint_vm_topdown_proof_service::verify_proof_bundle;

        // Verify cryptographic proofs (storage + events)
        verify_proof_bundle(&bundle.proof_bundle, &bundle.certificate)
            .context("proof bundle cryptographic verification failed")?;

        tracing::debug!(
            instance_id = bundle.certificate.instance_id,
            "proof bundle verified successfully (attestation)"
        );

        Ok(())
    }

    /// Verify proof bundle with full state validation (during execution).
    ///
    /// This performs:
    /// 1. Certificate chain continuity check (validates against F3LightClientActor state)
    /// 2. Cryptographic proof verification
    fn verify_proof_bundle_with_state(
        &self,
        state: &mut FvmExecState<DB>,
        bundle: &fendermint_vm_message::ipc::TopDownProofBundle,
    ) -> anyhow::Result<()> {
        // Step 1: Verify certificate chain continuity
        // Query F3LightClientActor for last committed instance
        let f3_state = self
            .f3_light_client_caller
            .get_state(state)
            .context("failed to query F3LightClientActor state")?;

        // Ensure bundle.certificate.instance_id == last_committed + 1
        if bundle.certificate.instance_id != f3_state.instance_id + 1 {
            bail!(
                "Certificate instance ID {} is not sequential (expected {})",
                bundle.certificate.instance_id,
                f3_state.instance_id + 1
            );
        }

        tracing::debug!(
            current_instance = f3_state.instance_id,
            new_instance = bundle.certificate.instance_id,
            "verified certificate chain continuity"
        );

        // Step 2: Verify cryptographic proofs (already done in attestation, but verify again)
        self.verify_proof_bundle_attestation(bundle)?;

        // Step 3: TODO - Verify F3 certificate cryptographically using F3Client
        // This requires:
        // 1. Initialize F3Client with power table from f3_state
        // 2. Call f3_client.fetch_and_validate(bundle.certificate.instance_id)
        // 3. Verify BLS signatures, quorum, and chain continuity
        // For now, we skip this and trust the certificate

        Ok(())
    }

    pub async fn update_voting_power_table(&self, power_updates: &PowerUpdates) {
        let power_updates_mapped: Vec<_> = power_updates
            .0
            .iter()
            .map(|v| (ValidatorKey::from(v.public_key.0), v.power.0))
            .collect();

        atomically(|| self.votes.update_power_table(power_updates_mapped.clone())).await
    }

    /// Execute proof-based topdown finality (v2).
    ///
    /// Steps:
    /// 1. Extract topdown messages from proof bundle (via ABI decoding)
    /// 2. Extract validator changes from proof bundle (via ABI decoding)
    /// 3. Commit parent finality to gateway (use highest epoch from certificate)
    /// 4. Store validator changes in gateway
    /// 5. Execute topdown messages
    /// 6. Update F3LightClientActor with new certificate state
    /// 7. Mark instance as committed in cache
    pub async fn execute_proof_based_topdown(
        &self,
        state: &mut FvmExecState<DB>,
        bundle: fendermint_vm_message::ipc::TopDownProofBundle,
    ) -> anyhow::Result<AppliedMessage> {
        if !self.provider.is_enabled() {
            bail!("cannot execute IPC top-down message: parent provider disabled");
        }

        tracing::debug!(
            instance = bundle.certificate.instance_id,
            "executing proof-based topdown finality"
        );

        // Step 0: Verify proof bundle with state (chain continuity check)
        self.verify_proof_bundle_with_state(state, &bundle)
            .context("proof bundle verification with state failed")?;

        // Step 1 & 2: Extract topdown effects from proof bundle
        let msgs = self.extract_topdown_messages_from_bundle(&bundle.proof_bundle)?;
        let validator_changes = self.extract_validator_changes_from_bundle(&bundle.proof_bundle)?;

        tracing::debug!(
            message_count = msgs.len(),
            validator_changes_count = validator_changes.len(),
            "extracted topdown effects from proof bundle"
        );

        // Step 3: Commit parent finality to gateway
        // Use the highest finalized epoch from the certificate
        let highest_epoch = bundle
            .certificate
            .finalized_epochs
            .iter()
            .max()
            .copied()
            .context("certificate has no finalized epochs")?;
        let finality = IPCParentFinality::new(highest_epoch as BlockHeight, vec![]);

        let (prev_height, _prev_finality) = self
            .commit_finality(state, finality.clone())
            .await
            .context("failed to commit finality")?;

        tracing::debug!(
            previous_height = prev_height,
            current_height = finality.height,
            "committed parent finality"
        );

        // Step 4: Store validator changes in gateway
        self.gateway_caller
            .store_validator_changes(state, validator_changes)
            .context("failed to store validator changes")?;

        // Step 5: Execute topdown messages
        let ret = self
            .execute_topdown_msgs(state, msgs)
            .await
            .context("failed to execute top down messages")?;

        tracing::debug!("applied topdown messages");

        // Step 6: Update F3LightClientActor with new certificate state
        // Convert power table from proof service format to actor format
        let power_table: Vec<fendermint_vm_actor_interface::f3_light_client::PowerEntry> = bundle
            .certificate
            .power_table
            .iter()
            .map(
                |pe| fendermint_vm_actor_interface::f3_light_client::PowerEntry {
                    public_key: pe.public_key.clone(),
                    power: pe.power,
                },
            )
            .collect();

        let new_light_client_state =
            fendermint_vm_actor_interface::f3_light_client::LightClientState {
                instance_id: bundle.certificate.instance_id,
                finalized_epochs: bundle.certificate.finalized_epochs.clone(),
                power_table,
            };

        self.f3_light_client_caller
            .update_state(state, new_light_client_state)
            .context("failed to update F3LightClientActor state")?;

        tracing::debug!(
            instance = bundle.certificate.instance_id,
            "updated F3LightClientActor state"
        );

        // Step 7: Mark instance as committed in cache
        if let Some(cache) = &self.proof_cache {
            cache.mark_committed(bundle.certificate.instance_id);
            tracing::debug!(
                instance = bundle.certificate.instance_id,
                "marked instance as committed in cache"
            );
        }

        tracing::info!(
            instance = bundle.certificate.instance_id,
            height = finality.height,
            "proof-based topdown finality executed successfully"
        );

        Ok(ret)
    }

    /// Extract topdown messages from proof bundle event proofs.
    ///
    /// Decodes `NewTopDownMessage` events from the proof bundle using ABI decoding.
    ///
    /// Event signature: `NewTopDownMessage(address indexed subnet, IpcEnvelope message, bytes32 indexed id)`
    fn extract_topdown_messages_from_bundle(
        &self,
        proof_bundle: &proofs::proofs::common::bundle::UnifiedProofBundle,
    ) -> anyhow::Result<Vec<IpcEnvelope>> {
        use ethers::abi::{Abi, RawLog};
        use ethers::types as et;

        // NewTopDownMessage event signature
        // event NewTopDownMessage(address indexed subnet, IpcEnvelope message, bytes32 indexed id)
        let event_signature = et::H256::from_slice(&ethers::utils::keccak256(
            "NewTopDownMessage(address,IpcEnvelope,bytes32)",
        ));

        let mut messages = Vec::new();

        // Iterate through event proofs in the bundle
        for event_proof in &proof_bundle.event_proofs {
            // TODO: Decode event proof structure to extract logs
            // The event_proof contains:
            // - Receipt with logs
            // - Merkle proof for the receipt
            // Each log has: address, topics[], data
            //
            // For each log:
            // 1. Check if topics[0] == event_signature
            // 2. If yes, decode topics and data:
            //    - topics[1] = subnet address (indexed)
            //    - topics[2] = message id (indexed)
            //    - data = ABI-encoded IpcEnvelope
            // 3. Use contract-bindings or ethabi to decode IpcEnvelope from data
            // 4. Convert to ipc_api::cross::IpcEnvelope and add to messages

            tracing::debug!("TODO: Decode event proof for NewTopDownMessage events");
        }

        tracing::warn!(
            "extract_topdown_messages_from_bundle not yet implemented - returning empty",
        );

        Ok(messages)
    }

    /// Extract validator changes from proof bundle event proofs.
    ///
    /// Decodes `NewPowerChangeRequest` events from the proof bundle using ABI decoding.
    ///
    /// Event signature: `NewPowerChangeRequest(uint8 op, address validator, bytes payload, uint64 configurationNumber)`
    fn extract_validator_changes_from_bundle(
        &self,
        proof_bundle: &proofs::proofs::common::bundle::UnifiedProofBundle,
    ) -> anyhow::Result<Vec<ipc_api::staking::PowerChangeRequest>> {
        use ethers::abi::{Abi, RawLog};
        use ethers::types as et;

        // NewPowerChangeRequest event signature
        // event NewPowerChangeRequest(uint8 op, address validator, bytes payload, uint64 configurationNumber)
        let event_signature = et::H256::from_slice(&ethers::utils::keccak256(
            "NewPowerChangeRequest(uint8,address,bytes,uint64)",
        ));

        let mut changes = Vec::new();

        // Iterate through event proofs in the bundle
        for event_proof in &proof_bundle.event_proofs {
            // TODO: Decode event proof structure to extract logs
            // The event_proof contains:
            // - Receipt with logs
            // - Merkle proof for the receipt
            // Each log has: address, topics[], data
            //
            // For each log:
            // 1. Check if topics[0] == event_signature
            // 2. If yes, decode topics and data:
            //    - data = ABI-encoded (uint8 op, address validator, bytes payload, uint64 configurationNumber)
            // 3. Use contract-bindings or ethabi to decode into PowerChangeRequest
            // 4. Convert to ipc_api::staking::PowerChangeRequest and add to changes

            tracing::debug!("TODO: Decode event proof for NewPowerChangeRequest events");
        }

        tracing::warn!(
            "extract_validator_changes_from_bundle not yet implemented - returning empty",
        );

        Ok(changes)
    }

    // TODO Karel - separate this huge function and clean up
    pub async fn execute_topdown_msg(
        &self,
        state: &mut FvmExecState<DB>,
        finality: ParentFinality,
    ) -> anyhow::Result<AppliedMessage> {
        if !self.provider.is_enabled() {
            bail!("cannot execute IPC top-down message: parent provider disabled");
        }

        // commit parent finality first
        let finality = IPCParentFinality::new(finality.height, finality.block_hash);
        tracing::debug!(
            finality = finality.to_string(),
            "chain interpreter received topdown exec proposal",
        );

        let (prev_height, prev_finality) = self
            .commit_finality(state, finality.clone())
            .await
            .context("failed to commit finality")?;

        tracing::debug!(
            previous_committed_height = prev_height,
            previous_committed_finality = prev_finality
                .as_ref()
                .map(|f| format!("{f}"))
                .unwrap_or_else(|| String::from("None")),
            "chain interpreter committed topdown finality",
        );

        // The height range we pull top-down effects from. This _includes_ the proposed
        // finality, as we assume that the interface we query publishes only fully
        // executed blocks as the head of the chain. This is certainly the case for
        // Ethereum-compatible JSON-RPC APIs, like Filecoin's. It should be the case
        // too for future Filecoin light clients.
        //
        // Another factor to take into account is the chain_head_delay, which must be
        // non-zero. So even in the case where deferred execution leaks through our
        // query mechanism, it should not be problematic because we're guaranteed to
        // be _at least_ 1 height behind.
        let (execution_fr, execution_to) = (prev_height + 1, finality.height);

        // error happens if we cannot get the validator set from ipc agent after retries
        let validator_changes = self
            .provider
            .validator_changes_from(execution_fr, execution_to)
            .await
            .context("failed to fetch validator changes")?;

        tracing::debug!(
            from = execution_fr,
            to = execution_to,
            msgs = validator_changes.len(),
            "chain interpreter received total validator changes"
        );

        self.gateway_caller
            .store_validator_changes(state, validator_changes)
            .context("failed to store validator changes")?;

        // error happens if we cannot get the cross messages from ipc agent after retries
        let msgs = self
            .provider
            .top_down_msgs_from(execution_fr, execution_to)
            .await
            .context("failed to fetch top down messages")?;

        tracing::debug!(
            number_of_messages = msgs.len(),
            start = execution_fr,
            end = execution_to,
            "chain interpreter received topdown msgs",
        );

        let ret = self
            .execute_topdown_msgs(state, msgs)
            .await
            .context("failed to execute top down messages")?;

        tracing::debug!("chain interpreter applied topdown msgs");

        let local_block_height = state.block_height() as u64;
        let proposer = state
            .block_producer()
            .map(|id| hex::encode(id.serialize_compressed()));
        let proposer_ref = proposer.as_deref();

        atomically(|| {
            self.provider.set_new_finality(finality.clone())?;

            self.votes.set_finalized(
                finality.height,
                finality.block_hash.clone(),
                proposer_ref,
                Some(local_block_height),
            )?;

            Ok(())
        })
        .await;

        tracing::debug!(
            finality = finality.to_string(),
            "chain interpreter has set new"
        );

        Ok(ret)
    }

    /// Commit the parent finality. Returns the height that the previous parent finality is committed and
    /// the committed finality itself. If there is no parent finality committed, genesis epoch is returned.
    async fn commit_finality(
        &self,
        state: &mut FvmExecState<DB>,
        finality: IPCParentFinality,
    ) -> anyhow::Result<(BlockHeight, Option<IPCParentFinality>)> {
        let (prev_height, prev_finality) = if let Some(prev_finality) = self
            .gateway_caller
            .commit_parent_finality(state, finality)?
        {
            (prev_finality.height, Some(prev_finality))
        } else {
            (self.provider.genesis_epoch()?, None)
        };

        tracing::debug!(
            "commit finality parsed: prev_height {prev_height}, prev_finality: {prev_finality:?}"
        );

        Ok((prev_height, prev_finality))
    }

    /// Execute the top down messages implicitly. Before the execution, mint to the gateway of the funds
    /// transferred in the messages, and increase the circulating supply with the incoming value.
    async fn execute_topdown_msgs(
        &self,
        state: &mut FvmExecState<DB>,
        messages: Vec<IpcEnvelope>,
    ) -> anyhow::Result<AppliedMessage> {
        let minted_tokens = tokens_to_mint(&messages);
        tracing::debug!(token = minted_tokens.to_string(), "tokens to mint in child");

        if !minted_tokens.is_zero() {
            self.gateway_caller
                .mint_to_gateway(state, minted_tokens.clone())
                .context("failed to mint to gateway")?;

            state.update_circ_supply(|circ_supply| {
                *circ_supply += minted_tokens;
            });
        }

        self.gateway_caller.apply_cross_messages(state, messages)
    }
}
