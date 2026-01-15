// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::ParentFinality;
use fendermint_vm_topdown::{BlockHeight, IPCParentFinality};

use crate::fvm::state::ipc::GatewayCaller;
use crate::fvm::state::FvmExecState;
use anyhow::{bail, Context};
use fvm_ipld_blockstore::Blockstore;

use crate::fvm::end_block_hook::PowerUpdates;
use crate::fvm::f3_topdown::F3TopDownHandler;
use crate::fvm::legacy_topdown::LegacyTopDownHandler;
use crate::fvm::state::ipc::tokens_to_mint;
use crate::types::AppliedMessage;
use ipc_api::cross::IpcEnvelope;

#[derive(Clone)]
pub enum TopDownFinalityHandler {
    Disabled,
    Legacy(LegacyTopDownHandler),
    F3(F3TopDownHandler),
}

#[derive(Clone)]
pub struct TopDownManager<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    finality: TopDownFinalityHandler,
    // Gateway caller for IPC gateway interactions
    gateway_caller: GatewayCaller<DB>,
}

impl<DB> TopDownManager<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    pub fn new(finality: TopDownFinalityHandler) -> Self {
        Self {
            finality,
            gateway_caller: GatewayCaller::default(),
        }
    }

    pub fn disabled() -> Self {
        Self::new(TopDownFinalityHandler::Disabled)
    }

    pub fn legacy(handler: LegacyTopDownHandler) -> Self {
        Self::new(TopDownFinalityHandler::Legacy(handler))
    }

    pub fn f3(handler: F3TopDownHandler) -> Self {
        Self::new(TopDownFinalityHandler::F3(handler))
    }

    pub async fn attest_legacy(&self, finality: ParentFinality) -> bool {
        match &self.finality {
            TopDownFinalityHandler::Legacy(h) => h.attest(finality).await,
            TopDownFinalityHandler::F3(_) | TopDownFinalityHandler::Disabled => false,
        }
    }

    /// Get the chain message for parent finality proposal.
    ///
    /// This method encapsulates the decision of which finality mechanism to use:
    /// - If configured for legacy: use legacy voting-based finality
    /// - If configured for F3: use F3 proof-based finality (no fallback)
    ///
    /// The caller doesn't need to know which mechanism is being used.
    pub async fn chain_message_for_proposal(&self) -> Option<ChainMessage> {
        match &self.finality {
            TopDownFinalityHandler::Disabled => None,
            TopDownFinalityHandler::Legacy(h) => h.chain_message_for_proposal().await,
            TopDownFinalityHandler::F3(f3) => {
                let proof_msg = f3.chain_message_from_proof_cache()?;
                tracing::info!("using F3 proof-based parent finality in proposal");
                Some(proof_msg)
            }
        }
    }

    /// Attest a generalised top-down message during the attestation phase.
    ///
    /// Delegates to F3 handler if F3 is configured, otherwise returns error.
    pub async fn attest_generalised(
        &self,
        msg: &fendermint_vm_message::ipc::GeneralisedTopDown,
    ) -> anyhow::Result<()> {
        match &self.finality {
            TopDownFinalityHandler::F3(f3) => f3.attest(msg).await,
            TopDownFinalityHandler::Legacy(_) | TopDownFinalityHandler::Disabled => Err(
                anyhow::anyhow!("F3 not configured - cannot attest generalised top-down message"),
            ),
        }
    }

    pub async fn update_voting_power_table(&self, power_updates: &PowerUpdates) {
        if let TopDownFinalityHandler::Legacy(h) = &self.finality {
            h.update_voting_power_table(power_updates).await
        }
    }

    /// Execute generalised top-down message.
    /// Delegates F3-specific logic to F3 module, handles common top-down execution.
    pub async fn execute_generalised(
        &self,
        state: &mut FvmExecState<DB>,
        msg: fendermint_vm_message::ipc::GeneralisedTopDown,
    ) -> anyhow::Result<AppliedMessage> {
        let f3 = match &self.finality {
            TopDownFinalityHandler::F3(f3) => f3,
            TopDownFinalityHandler::Legacy(_) | TopDownFinalityHandler::Disabled => {
                bail!("F3 not configured - cannot execute without F3 handler")
            }
        };

        // Execute F3-specific logic (certificate validation, proof extraction, state updates)
        let (msgs, validator_changes) = f3.execute(state, &msg)?;

        // Commit parent finality to gateway
        let finality = IPCParentFinality::new(msg.height as i64, vec![]);
        let (prev_height, _prev_finality) = self
            .commit_finality(state, finality.clone(), 0)
            .await
            .context("failed to commit finality")?;

        tracing::debug!(
            previous_height = prev_height,
            current_height = finality.height,
            "committed parent finality"
        );

        // Store validator changes in gateway
        self.gateway_caller
            .store_validator_changes(state, validator_changes)
            .context("failed to store validator changes")?;

        // Execute topdown messages
        let ret = self
            .execute_topdown_msgs(state, msgs)
            .await
            .context("failed to execute top down messages")?;

        tracing::info!(
            height = msg.height,
            "generalised top-down executed successfully"
        );

        Ok(ret)
    }

    // TODO Karel - separate this huge function and clean up
    pub async fn execute_legacy(
        &self,
        state: &mut FvmExecState<DB>,
        finality: ParentFinality,
    ) -> anyhow::Result<AppliedMessage> {
        let legacy = match &self.finality {
            TopDownFinalityHandler::Legacy(h) => h,
            TopDownFinalityHandler::F3(_) => bail!("cannot execute legacy top-down: F3 enabled"),
            TopDownFinalityHandler::Disabled => {
                bail!("cannot execute IPC top-down message: parent provider disabled")
            }
        };
        if !legacy.is_enabled() {
            bail!("cannot execute IPC top-down message: parent provider disabled");
        }

        // commit parent finality first
        let finality = IPCParentFinality::new(finality.height, finality.block_hash);
        tracing::debug!(
            finality = finality.to_string(),
            "chain interpreter received topdown exec proposal",
        );

        let (prev_height, prev_finality) = self
            .commit_finality(state, finality.clone(), legacy.genesis_epoch()?)
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
        let validator_changes = legacy
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
        let msgs = legacy
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

        legacy
            .on_finality_executed(finality.clone(), proposer_ref, local_block_height)
            .await
            .context("failed to record new finality")?;

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
        genesis_epoch: BlockHeight,
    ) -> anyhow::Result<(BlockHeight, Option<IPCParentFinality>)> {
        let (prev_height, prev_finality) = if let Some(prev_finality) = self
            .gateway_caller
            .commit_parent_finality(state, finality)?
        {
            (prev_finality.height, Some(prev_finality))
        } else {
            (genesis_epoch, None)
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
