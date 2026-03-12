// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::ParentFinality;
use fendermint_vm_topdown::{BlockHeight, IPCParentFinality};

use crate::fvm::state::ipc::GatewayCaller;
use crate::fvm::state::FvmExecState;
use anyhow::{bail, Context};
use fvm_ipld_blockstore::Blockstore;
use std::sync::{Arc, OnceLock};

use crate::fvm::end_block_hook::PowerUpdates;
use crate::fvm::f3_topdown::{F3TopDownError, F3TopDownHandler};
use crate::fvm::legacy_topdown::LegacyTopDownHandler;
use crate::fvm::observe::{F3CacheWaitRecovered, F3CacheWaitStuck};
use crate::fvm::state::ipc::tokens_to_mint;
use crate::types::AppliedMessage;
use ipc_api::cross::IpcEnvelope;
use ipc_observability::emit;

#[derive(Clone, Debug)]
pub struct F3ExecutionCacheRetryConfig {
    pub backoff_initial: std::time::Duration,
    pub backoff_max: std::time::Duration,
    /// After this much waiting, emit an error-severity event/log to surface that block execution
    /// is blocked on a missing local proof-cache entry. Execution will still keep retrying.
    pub critical_after: std::time::Duration,
    pub error_after: std::time::Duration,
}

impl Default for F3ExecutionCacheRetryConfig {
    fn default() -> Self {
        Self {
            backoff_initial: std::time::Duration::from_millis(200),
            backoff_max: std::time::Duration::from_secs(5),
            critical_after: std::time::Duration::from_secs(10 * 60),
            error_after: std::time::Duration::from_secs(2 * 60),
        }
    }
}

#[derive(Clone)]
pub enum TopDownFinalityHandler {
    Disabled,
    Legacy(LegacyTopDownHandler),
    F3(F3TopDownHandler),
}

struct TopDownManagerInner<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    legacy: Option<LegacyTopDownHandler>,
    f3: OnceLock<F3Runtime>,
    // Gateway caller for IPC gateway interactions
    gateway_caller: GatewayCaller<DB>,
}

#[derive(Clone)]
struct F3Runtime {
    handler: F3TopDownHandler,
    retry: F3ExecutionCacheRetryConfig,
}

#[derive(Clone)]
pub struct TopDownManager<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    inner: Arc<TopDownManagerInner<DB>>,
}

impl<DB> TopDownManager<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    fn is_cache_miss(err: &anyhow::Error) -> bool {
        err.chain().any(|cause| {
            matches!(
                cause.downcast_ref::<F3TopDownError>(),
                Some(F3TopDownError::CacheMiss { .. })
            )
        })
    }

    /// Extract top-down effects, retrying on local proof-cache misses up to a bounded timeout.
    ///
    /// This is used during block execution (catch-up): a node might not have had the local cache
    /// entry during attestation (so it didn't vote), but it still needs to be able to apply the
    /// committed block once the proof-service catches up.
    async fn extract_top_down_effects_retry_cache_miss(
        retry: &F3ExecutionCacheRetryConfig,
        f3: &F3TopDownHandler,
        msg: &fendermint_vm_message::ipc::ParentFinalityWithCert,
    ) -> anyhow::Result<crate::fvm::f3_topdown::ExtractedTopDownEffects> {
        use tokio::time::sleep;

        // Tuning:
        // - critical_after controls when we start emitting an error-severity signal.
        // - error_after controls how often we repeat that signal once we're in the critical state.
        let mut backoff = retry.backoff_initial;
        let max_backoff = retry.backoff_max;
        let critical_after = retry.critical_after;
        let error_after = retry.error_after;
        let mut next_error_log_at = critical_after;
        let mut waited = std::time::Duration::ZERO;
        let mut saw_cache_miss = false;
        let mut entered_critical = false;

        loop {
            match f3.extract_top_down_effects(msg) {
                Ok(v) => {
                    if saw_cache_miss {
                        emit(F3CacheWaitRecovered {
                            epoch: msg.height as u64,
                            waited_secs: waited.as_secs_f64(),
                        });
                    }
                    return Ok(v);
                }
                Err(e) if Self::is_cache_miss(&e) => {
                    saw_cache_miss = true;
                    // Don't abort execution on cache wait: keep retrying forever.
                    // Once we cross `critical_after`, switch into a "critical" state where we emit
                    // an error-severity signal periodically.
                    if waited >= critical_after && !entered_critical {
                        entered_critical = true;
                        tracing::error!(
                            height = msg.height,
                            waited = ?waited,
                            critical_after = ?critical_after,
                            "still missing local proof cache entry after critical_after; continuing to wait"
                        );
                    } else if waited >= next_error_log_at {
                        if entered_critical {
                            tracing::error!(
                                height = msg.height,
                                waited = ?waited,
                                "still missing local proof cache entry; node cannot execute parent-finality-with-cert yet"
                            );
                            emit(F3CacheWaitStuck {
                                epoch: msg.height as u64,
                                waited_secs: waited.as_secs_f64(),
                            });
                            next_error_log_at += error_after;
                        }
                    } else {
                        tracing::warn!(
                            height = msg.height,
                            waited = ?waited,
                            retry_in = ?backoff,
                            "missing local proof cache entry; waiting for proof-service to fill cache"
                        );
                    }
                    sleep(backoff).await;
                    waited += backoff;
                    backoff = std::cmp::min(backoff * 2, max_backoff);
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn new(finality: TopDownFinalityHandler) -> Self {
        let (legacy, f3) = match finality {
            TopDownFinalityHandler::Disabled => (None, None),
            TopDownFinalityHandler::Legacy(h) => (Some(h), None),
            TopDownFinalityHandler::F3(h) => (
                None,
                Some(F3Runtime {
                    handler: h,
                    retry: Default::default(),
                }),
            ),
        };
        let f3_cell = OnceLock::new();
        if let Some(runtime) = f3 {
            let _ = f3_cell.set(runtime);
        }
        Self {
            inner: Arc::new(TopDownManagerInner {
                legacy,
                f3: f3_cell,
                gateway_caller: GatewayCaller::default(),
            }),
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

    pub fn f3_with_retry_config(
        handler: F3TopDownHandler,
        retry: F3ExecutionCacheRetryConfig,
    ) -> Self {
        let f3_cell = OnceLock::new();
        let _ = f3_cell.set(F3Runtime { handler, retry });
        Self {
            inner: Arc::new(TopDownManagerInner {
                legacy: None,
                f3: f3_cell,
                gateway_caller: GatewayCaller::default(),
            }),
        }
    }

    /// Activate F3 exactly once at runtime.
    ///
    /// Intended for first-boot lifecycle where the node starts before committed state is queryable.
    pub fn activate_f3_once(
        &self,
        handler: F3TopDownHandler,
        retry: F3ExecutionCacheRetryConfig,
    ) -> anyhow::Result<()> {
        if self.inner.legacy.is_some() {
            bail!("cannot activate F3: legacy topdown is configured");
        }
        self.inner
            .f3
            .set(F3Runtime { handler, retry })
            .map_err(|_| anyhow::anyhow!("cannot activate F3: already active"))?;
        Ok(())
    }

    pub async fn attest_legacy(&self, finality: ParentFinality) -> bool {
        match &self.inner.legacy {
            Some(h) => h.attest(finality).await,
            None => false,
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
        if let Some(f3) = self.inner.f3.get() {
            let proof_msg = f3.handler.chain_message_from_proof_cache();
            if proof_msg.is_some() {
                tracing::info!("Including F3 proof-based parent-finality message in proposal");
            } else {
                tracing::info!("F3 enabled but no proposal parent-finality message available");
            }
            return proof_msg;
        }
        if let Some(h) = &self.inner.legacy {
            tracing::debug!("Using legacy top-down proposal path");
            return h.chain_message_for_proposal().await;
        }
        tracing::debug!("Top-down disabled; proposal includes only mempool messages");
        None
    }

    /// Attest a parent-finality-with-cert message during the attestation phase.
    ///
    /// Delegates to F3 handler if F3 is configured, otherwise returns error.
    pub async fn attest_parent_finality_with_cert<BS>(
        &self,
        state: &mut FvmExecState<BS>,
        msg: &fendermint_vm_message::ipc::ParentFinalityWithCert,
    ) -> anyhow::Result<()>
    where
        BS: Blockstore + Clone + 'static + Send + Sync,
    {
        if let Some(f3) = self.inner.f3.get() {
            return f3.handler.attest(state, msg).await;
        }
        Err(anyhow::anyhow!(
            "F3 not configured - cannot attest parent-finality-with-cert message"
        ))
    }

    pub async fn update_voting_power_table(&self, power_updates: &PowerUpdates) {
        if let Some(h) = &self.inner.legacy {
            h.update_voting_power_table(power_updates).await
        }
    }

    /// Execute parent-finality-with-cert message.
    /// Delegates F3-specific logic to F3 module, handles common top-down execution.
    pub async fn execute_parent_finality_with_cert(
        &self,
        state: &mut FvmExecState<DB>,
        msg: fendermint_vm_message::ipc::ParentFinalityWithCert,
    ) -> anyhow::Result<AppliedMessage> {
        let Some(f3) = self.inner.f3.get() else {
            bail!("F3 not configured - cannot execute without F3 handler");
        };

        // Execute F3-specific logic (certificate validation, proof extraction, state updates).
        //
        // This path may be hit during catch-up for a node that did not have the local proof cache
        // entry during attestation. In that case, wait for the cache to be filled by the proof-service.
        let extracted =
            Self::extract_top_down_effects_retry_cache_miss(&f3.retry, &f3.handler, &msg).await?;

        // Commit parent finality to gateway.
        //
        // The gateway expects a fixed `bytes32 blockHash`, so for Filecoin we commit the FEVM
        // (Ethereum-view) block hash corresponding to this epoch, derived deterministically from
        // the cached tipset key bytes for this epoch (see `F3TopDownHandler`).
        let finality = IPCParentFinality::new(msg.height, extracted.parent_eth_block_hash.to_vec());
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
        self.inner
            .gateway_caller
            .store_validator_changes(state, extracted.validator_changes)
            .context("failed to store validator changes")?;

        // Execute topdown messages
        let ret = self
            .execute_topdown_msgs(state, extracted.topdown_msgs)
            .await
            .context("failed to execute top down messages")?;

        // Finalize F3 execution only after all effects were applied successfully.
        f3.handler
            .finalize_after_execution(state, msg.height, extracted.instance_id)
            .context("failed to finalize F3 execution")?;

        tracing::info!(
            height = msg.height,
            "parent finality with cert executed successfully"
        );

        Ok(ret)
    }

    // TODO Karel - separate this huge function and clean up
    pub async fn execute_legacy(
        &self,
        state: &mut FvmExecState<DB>,
        finality: ParentFinality,
    ) -> anyhow::Result<AppliedMessage> {
        if self.inner.f3.get().is_some() {
            bail!("cannot execute legacy top-down: F3 enabled");
        }
        let Some(legacy) = &self.inner.legacy else {
            bail!("cannot execute IPC top-down message: parent provider disabled");
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

        self.inner
            .gateway_caller
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
            .inner
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
            self.inner
                .gateway_caller
                .mint_to_gateway(state, minted_tokens.clone())
                .context("failed to mint to gateway")?;

            state.update_circ_supply(|circ_supply| {
                *circ_supply += minted_tokens;
            });
        }

        self.inner
            .gateway_caller
            .apply_cross_messages(state, messages)
    }
}
