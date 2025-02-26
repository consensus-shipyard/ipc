// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::state::ipc::GatewayCaller;
use crate::fvm::PowerUpdates;
use async_stm::atomically;
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::{BottomUpCheckpoint, CertifiedMessage, IpcMessage};
use fendermint_vm_resolver::pool::{ResolveKey, ResolvePool};
use tendermint_rpc::Client;

use fendermint_crypto::{PublicKey, SecretKey};
use fvm_ipld_blockstore::Blockstore;
use ipc_observability::{emit, observe::TracingError, Traceable};

use crate::fvm::observe::CheckpointFinalized;
// TODO Karel - this should be moved here.
use crate::fvm::ValidatorContext;

use anyhow::Context;

use crate::fvm::broadcast::Broadcaster;
use crate::fvm::checkpoint::{
    broadcast_incomplete_signatures, emit_trace_if_check_checkpoint_finalized,
    maybe_create_checkpoint, unsigned_checkpoints,
};
use crate::fvm::state::FvmExecState;

use crate::types::BlockEndEvents;

use fvm_shared::address::Address;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum CheckpointPoolItem {
    /// BottomUp checkpoints to be resolved from the originating subnet or the current one.
    BottomUp(CertifiedMessage<BottomUpCheckpoint>),
    // We can extend this to include top-down checkpoints as well, with slightly
    // different resolution semantics (resolving it from a trusted parent, and
    // awaiting finality before declaring it available).
}

impl From<CertifiedMessage<BottomUpCheckpoint>> for CheckpointPoolItem {
    fn from(value: CertifiedMessage<BottomUpCheckpoint>) -> Self {
        CheckpointPoolItem::BottomUp(value)
    }
}

impl From<&CheckpointPoolItem> for ResolveKey {
    fn from(value: &CheckpointPoolItem) -> Self {
        match value {
            CheckpointPoolItem::BottomUp(cp) => {
                (cp.message.subnet_id.clone(), cp.message.bottom_up_messages)
            }
        }
    }
}

// TODO Karel - clean this up. This should probably not be here and also the bottom up check ABI should not leak here.
pub struct CheckpointOutcome {
    pub checkpoint: ipc_actors_abis::checkpointing_facet::BottomUpCheckpoint,
    pub power_updates: PowerUpdates,
    pub block_end_events: BlockEndEvents,
}

pub struct BottomUpManager<DB, C>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
    C: Client + Clone + Send + Sync + 'static,
{
    pool: ResolvePool<CheckpointPoolItem>,

    /// Tendermint client for querying the RPC.
    tendermint_client: C,
    /// If this is a validator node, this should be the key we can use to sign transactions.
    validator_ctx: Option<ValidatorContext<C>>,

    // Gateway caller for IPC gateway interactions
    gateway_caller: GatewayCaller<DB>,
}

impl<DB, C> BottomUpManager<DB, C>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
    C: Client + Clone + Send + Sync + 'static,
{
    pub fn new(
        resolve_pool: ResolvePool<CheckpointPoolItem>,
        tendermint_client: C,
        validator_ctx: Option<ValidatorContext<C>>,
    ) -> Self {
        Self {
            tendermint_client,
            pool: resolve_pool,
            validator_ctx,
            // TODO Karel - no default - better make it mockable?
            gateway_caller: GatewayCaller::default(),
        }
    }

    // Checks if the bottom up checkpoint is already resolved
    pub async fn is_checkpoint_resolved(&self, item: CheckpointPoolItem) -> bool {
        // We can just look in memory because when we start the application, we should retrieve any
        // pending checkpoints (relayed but not executed) from the ledger, so they should be there.
        // We don't have to validate the checkpoint here, because
        // 1) we validated it when it was relayed, and
        // 2) if a validator proposes something invalid, we can make them pay during execution.
        let is_resolved = atomically(|| match self.pool.get_status(&item)? {
            None => Ok(false),
            Some(status) => status.is_resolved(),
        })
        .await;
        is_resolved
    }

    pub async fn add_checkpoint(&self, item: CheckpointPoolItem) {
        atomically(|| self.pool.add(item.clone(), false)).await;
    }

    // Checks the bottom up checkpoint pool and returns the messages that represent the checkpoints for execution
    pub async fn messages_from_resolved_checkpoints(&self) -> Vec<ChainMessage> {
        let resolved = atomically(|| self.pool.collect_resolved()).await;
        resolved
            .into_iter()
            .map(|checkpoint| match checkpoint {
                CheckpointPoolItem::BottomUp(checkpoint) => {
                    ChainMessage::Ipc(IpcMessage::BottomUpExec(checkpoint))
                }
            })
            .collect()
    }

    pub fn create_checkpoint_if_needed(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<Option<CheckpointOutcome>> {
        let mut block_end_events = BlockEndEvents::default();

        // Emit trace; errors here are logged but not fatal.
        let _ = emit_trace_if_check_checkpoint_finalized(&self.gateway_caller, state).inspect_err(
            |e| {
                emit(TracingError {
                    affected_event: CheckpointFinalized::name(),
                    reason: e.to_string(),
                });
            },
        );

        let maybe_result =
            maybe_create_checkpoint(&self.gateway_caller, state, &mut block_end_events)
                .context("failed to create checkpoint")?;

        if let Some((checkpoint, power_updates)) = maybe_result {
            Ok(Some(CheckpointOutcome {
                checkpoint,
                power_updates,
                block_end_events,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn cast_validator_signatures_for_incomplete_checkpoints(
        &self,
        current_checkpoint: ipc_actors_abis::checkpointing_facet::BottomUpCheckpoint,
        state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<()> {
        // Exit early if there's no validator context.
        let validator_ctx = match self.validator_ctx.as_ref() {
            Some(ctx) => ctx,
            None => return Ok(()),
        };

        // If we're currently syncing, do not resend past signatures.
        if self.syncing().await {
            return Ok(());
        }

        // Retrieve incomplete checkpoints synchronously (state cannot be shared across threads).
        let incomplete_checkpoints =
            unsigned_checkpoints(&self.gateway_caller, state, validator_ctx.public_key)
                .context("failed to fetch incomplete checkpoints")?;

        // Ensure that the current checkpoint exists among the incomplete ones.
        debug_assert!(
            incomplete_checkpoints.iter().any(|checkpoint| {
                checkpoint.block_height == current_checkpoint.block_height
                    && checkpoint.block_hash == current_checkpoint.block_hash
            }),
            "the current checkpoint is incomplete"
        );

        // Clone the necessary values to move into the asynchronous task.
        let client = self.tendermint_client.clone();
        let gateway = self.gateway_caller.clone();
        let chain_id = state.chain_id();
        let height = current_checkpoint.block_height;
        let validator_ctx = validator_ctx.clone();

        // Spawn an asynchronous task to broadcast incomplete checkpoint signatures.
        tokio::spawn(async move {
            if let Err(e) = broadcast_incomplete_signatures(
                &client,
                &validator_ctx,
                &gateway,
                chain_id,
                incomplete_checkpoints,
            )
            .await
            {
                tracing::error!(error = ?e, height = height.as_u64(), "error broadcasting checkpoint signature");
            }
        });

        Ok(())
    }

    /// Indicate that the node is syncing with the rest of the network and hasn't caught up with the tip yet.
    async fn syncing(&self) -> bool {
        match self.tendermint_client.status().await {
            Ok(status) => status.sync_info.catching_up,
            Err(e) => {
                // CometBFT often takes a long time to boot, e.g. while it's replaying blocks it won't
                // respond to JSON-RPC calls. Let's treat this as an indication that we are syncing.
                tracing::warn!(error =? e, "failed to get CometBFT sync status");
                true
            }
        }
    }
}
