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

use crate::fvm::state::ipc::GatewayCaller;
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

        // Extract highest epoch as the finality height
        let height = entry.highest_epoch()? as ChainEpoch;

        // Extract block hash from the proof bundle
        // The proof bundle contains the parent tipset information
        // For now, we use an empty block hash as a placeholder
        // TODO: Extract actual block hash from certificate or proof bundle
        let block_hash = vec![];

        Some(ChainMessage::Ipc(IpcMessage::ParentFinalityWithProof(
            fendermint_vm_message::ipc::ParentFinalityProofBundle {
                finality: ParentFinality { height, block_hash },
                certificate: entry.certificate,
                proof_bundle: entry.proof_bundle,
            },
        )))
    }

    /// Deterministically verify a proof bundle against F3 certificate.
    ///
    /// This performs cryptographic verification of:
    /// 1. Storage proofs (contract state at parent height - completeness via topDownNonce)
    /// 2. Event proofs (emitted events at parent height)
    /// 3. Certificate chain continuity (validates against F3CertManager state)
    ///
    /// All correct validators will reach the same decision (deterministic).
    pub async fn verify_proof_bundle(
        &self,
        bundle: &fendermint_vm_message::ipc::ParentFinalityProofBundle,
    ) -> anyhow::Result<()> {
        use fendermint_vm_topdown_proof_service::verify_proof_bundle;

        // Step 1: Verify cryptographic proofs (storage + events)
        verify_proof_bundle(&bundle.proof_bundle, &bundle.certificate)
            .context("proof bundle cryptographic verification failed")?;

        // Step 2: TODO - Verify certificate chain continuity
        // Query F3CertManager for last committed instance
        // Ensure bundle.certificate.instance_id == last_committed + 1
        // This requires querying the F3CertManager actor state

        tracing::debug!(
            instance_id = bundle.certificate.instance_id,
            height = bundle.finality.height,
            "proof bundle verified successfully"
        );

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
    /// 1. Commit parent finality to gateway
    /// 2. Update F3CertManager actor with new certificate (TODO)
    /// 3. Extract and execute topdown effects (messages + validator changes)
    /// 4. Mark instance as committed in cache
    /// 5. Update local state (provider + votes)
    pub async fn execute_proof_based_topdown(
        &self,
        state: &mut FvmExecState<DB>,
        bundle: fendermint_vm_message::ipc::ParentFinalityProofBundle,
    ) -> anyhow::Result<AppliedMessage> {
        if !self.provider.is_enabled() {
            bail!("cannot execute IPC top-down message: parent provider disabled");
        }

        // Convert to IPCParentFinality
        let finality =
            IPCParentFinality::new(bundle.finality.height, bundle.finality.block_hash.clone());

        tracing::debug!(
            finality = finality.to_string(),
            instance = bundle.certificate.instance_id,
            "executing proof-based topdown finality"
        );

        // Step 1: Commit parent finality (same as v1)
        let (prev_height, _prev_finality) = self
            .commit_finality(state, finality.clone())
            .await
            .context("failed to commit finality")?;

        tracing::debug!(
            previous_height = prev_height,
            current_height = finality.height,
            "committed parent finality"
        );

        // Step 2: TODO - Update F3CertManager actor
        // self.update_f3_cert_manager(state, &bundle.certificate)?;

        // Step 3: Execute topdown effects
        // For now, we use the existing v1 path to fetch messages/changes from the provider
        // TODO: Extract from proof bundle instead
        let (execution_fr, execution_to) = (prev_height + 1, finality.height);

        let validator_changes = self
            .provider
            .validator_changes_from(execution_fr, execution_to)
            .await
            .context("failed to fetch validator changes")?;

        tracing::debug!(
            from = execution_fr,
            to = execution_to,
            change_count = validator_changes.len(),
            "fetched validator changes"
        );

        self.gateway_caller
            .store_validator_changes(state, validator_changes)
            .context("failed to store validator changes")?;

        let msgs = self
            .provider
            .top_down_msgs_from(execution_fr, execution_to)
            .await
            .context("failed to fetch top down messages")?;

        tracing::debug!(
            message_count = msgs.len(),
            start = execution_fr,
            end = execution_to,
            "fetched topdown messages"
        );

        let ret = self
            .execute_topdown_msgs(state, msgs)
            .await
            .context("failed to execute top down messages")?;

        tracing::debug!("applied topdown messages");

        // Step 4: Mark instance as committed in cache
        if let Some(cache) = &self.proof_cache {
            cache.mark_committed(bundle.certificate.instance_id);
            tracing::debug!(
                instance = bundle.certificate.instance_id,
                "marked instance as committed in cache"
            );
        }

        // Step 5: Update state (same as v1)
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

        tracing::info!(
            instance = bundle.certificate.instance_id,
            height = finality.height,
            "proof-based topdown finality executed successfully"
        );

        Ok(ret)
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
