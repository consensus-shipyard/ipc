use std::sync::Arc;
use thiserror::Error;

use crate::fvm::state::ipc::GatewayCaller;
use crate::fvm::state::FvmExecState;
use crate::fvm::store::ReadOnlyBlockstore;
use crate::fvm::upgrades::UpgradeScheduler;

use crate::selector::{select_messages_by_gas_limit, select_messages_until_total_bytes};

use crate::check::check_nonce_and_sufficient_balance;
use crate::implicit_messages::{execute_cron_message, maybe_push_chain_metadata};
use crate::types::*;

use crate::verify::{IllegalMessage, VerifiableMessage};

use fendermint_vm_message::signed::SignedMessageError;

use anyhow::Context;

use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::IpcMessage;

use fvm_ipld_blockstore::Blockstore;

use crate::bottomup::BottomUpCheckpointResolver;
use crate::topdown::TopDownCheckpointResolver;

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("illegal message: {0}")]
    IllegalMessage(#[from] IllegalMessage),
    #[error("invalid message: {0}")]
    InvalidMessage(String),
    #[error("message verification error")]
    SignedMessageError(#[from] SignedMessageError),
}

struct Interpreter<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    bottom_up_resolver: BottomUpCheckpointResolver,
    top_down_resolver: TopDownCheckpointResolver,

    /// The parent finality provider for top down checkpoint
    /// TODO Karel - move these into separate module for top down
    // TODO Karel - consider using trait to make this testable
    gateway_caller: GatewayCaller<DB>,
    /// Upgrade scheduler stores all the upgrades to be executed at given heights.
    upgrade_scheduler: UpgradeScheduler<DB>,

    /// Indicate whether the chain metadata should be pushed into the ledger.
    push_chain_meta: bool,
    /// Maximum number of messages to allow in a block.
    max_msgs_per_block: usize,
    /// Should we reject proposals with malformed transactions we cannot parse.
    reject_malformed_proposal: bool,
}

impl<DB> Interpreter<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    pub fn new(
        bottom_up_resolver: BottomUpCheckpointResolver,
        top_down_resolver: TopDownCheckpointResolver,
        upgrade_scheduler: UpgradeScheduler<DB>,
        push_chain_meta: bool,
        max_msgs_per_block: usize,
        reject_malformed_proposal: bool,
    ) -> Self {
        Self {
            gateway_caller: GatewayCaller::default(),
            bottom_up_resolver,
            top_down_resolver,
            upgrade_scheduler,
            push_chain_meta,
            max_msgs_per_block,
            reject_malformed_proposal,
        }
    }

    pub async fn check(
        &self,
        state: FvmExecState<ReadOnlyBlockstore<Arc<DB>>>,
        msg: Vec<u8>,
        is_recheck: bool,
    ) -> anyhow::Result<FvmCheckRet> {
        let verifiable_msg = ipld_decode_signed_message(&msg)?;
        let fvm_msg = verifiable_msg.message();

        // Check that the message is valid
        fvm_msg
            .check()
            .map_err(|e| InterpreterError::InvalidMessage(e.to_string()))?;

        // For recheck, we don't need to check the signature or the nonce and balance
        if is_recheck {
            return Ok(FvmCheckRet::new_ok(&fvm_msg));
        }

        // Check that the signature is valid
        verifiable_msg.verify(&state.chain_id())?;

        let check_ret = check_nonce_and_sufficient_balance(&state, &fvm_msg)?;

        tracing::info!(
            exit_code = check_ret.exit_code.value(),
            from = fvm_msg.from.to_string(),
            to = fvm_msg.to.to_string(),
            method_num = fvm_msg.method_num,
            gas_limit = fvm_msg.gas_limit,
            info = check_ret.info.as_deref().unwrap_or(""),
            "check transaction"
        );

        Ok(check_ret)
    }

    pub async fn prepare(
        &self,
        state: FvmExecState<ReadOnlyBlockstore<Arc<DB>>>,
        msgs: Vec<Vec<u8>>,
        max_transaction_bytes: u64,
    ) -> anyhow::Result<(Vec<Vec<u8>>, usize)> {
        // Signed messages from the mempool submitted via transactions
        let signed_msgs = msgs
            .iter()
            .filter_map(|msg| match ipld_decode_signed_message(msg) {
                Ok(vm) => Some(vm),
                Err(e) => {
                    // This should never happen because messages that are not signed should not reach the mempool
                    tracing::warn!(error = %e, "failed to decode signable mempool message");
                    None
                }
            })
            .collect();

        // Select messages by block gas limit
        let total_gas_limit = state.block_gas_tracker().available();
        let signed_msgs = select_messages_by_gas_limit(signed_msgs, total_gas_limit);

        let signed_msgs = signed_msgs
            .into_iter()
            .map(Into::into)
            .collect::<Vec<ChainMessage>>();

        // Messages generated by the protocol (e.g. top down, bottom up checkpoints)
        // Add bottom up messages ready for execution directly
        let mut protocol_msgs = self
            .bottom_up_resolver
            .messages_from_resolved_checkpoints()
            .await;

        // Add top down message if parent checkpoint is available
        if let Some(top_down_message) = self
            .top_down_resolver
            .resolve_message_from_finality_and_quorum()
            .await
        {
            protocol_msgs.push(top_down_message);
        }

        // Add protocol message first before use messages
        // This ensures that protocol messages are always executed first
        let mut all_msgs: Vec<Vec<u8>> = ipld_encode_messages(protocol_msgs)?
            .into_iter()
            .chain(ipld_encode_messages(signed_msgs)?.into_iter())
            .collect();

        // Truncate messages if they exceed the maximum allowed count per block.
        if all_msgs.len() > self.max_msgs_per_block {
            tracing::warn!(
                max_msgs = self.max_msgs_per_block,
                total_msgs = all_msgs.len(),
                "truncating proposal due to message count limit"
            );
            all_msgs.truncate(self.max_msgs_per_block);
        }

        let input_msg_count = all_msgs.len();

        // Select messages until the total byte size reaches the limit.
        let (all_messages, total_bytes) =
            select_messages_until_total_bytes(all_msgs, max_transaction_bytes as usize);

        if all_messages.len() < input_msg_count {
            tracing::warn!(
                removed_msgs = input_msg_count - all_messages.len(),
                max_bytes = max_transaction_bytes,
                "some messages were removed from the proposal because they exceed the byte limit"
            );
        }

        Ok((all_messages, total_bytes))
    }

    pub async fn process(
        &self,
        state: FvmExecState<ReadOnlyBlockstore<Arc<DB>>>,
        msgs: Vec<Vec<u8>>,
    ) -> anyhow::Result<ProcessDecision> {
        // Check if there are too many messages.
        if msgs.len() > self.max_msgs_per_block {
            tracing::warn!(
                block_msgs = msgs.len(),
                "rejecting block: too many messages"
            );
            return Ok(ProcessDecision::Accept);
        }

        // Decode raw messages into ChainMessages.
        let mut chain_msgs = Vec::with_capacity(msgs.len());
        for msg in msgs {
            match fvm_ipld_encoding::from_slice::<ChainMessage>(&msg) {
                Ok(chain_msg) => chain_msgs.push(chain_msg),
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "failed to decode message in proposal as ChainMessage"
                    );
                    if self.reject_malformed_proposal {
                        return Ok(ProcessDecision::Reject);
                    }
                }
            }
        }

        // Process the chain messages: perform async checks and accumulate gas usage.
        let mut block_gas_usage = 0;
        for msg in chain_msgs {
            match msg {
                ChainMessage::Ipc(IpcMessage::BottomUpExec(checkpoint)) => {
                    if !self
                        .bottom_up_resolver
                        .check_checkpoint_resolved(checkpoint.into())
                        .await
                    {
                        return Ok(ProcessDecision::Reject);
                    }
                }
                ChainMessage::Ipc(IpcMessage::TopDownExec(finality)) => {
                    if !self.top_down_resolver.check_valid(finality).await {
                        return Ok(ProcessDecision::Reject);
                    }
                }
                ChainMessage::Signed(signed) => {
                    block_gas_usage += signed.message.gas_limit;
                }
                // Other variants are currently ignored.
                _ => {}
            }
        }

        // Ensure the total gas usage does not exceed the block's available gas.
        if block_gas_usage > state.block_gas_tracker().available() {
            return Ok(ProcessDecision::Reject);
        }

        Ok(ProcessDecision::Accept)
    }

    async fn begin(
        &self,
        mut state: FvmExecState<DB>,
    ) -> anyhow::Result<(FvmExecState<DB>, FvmApplyRet)> {
        // Block height (FVM epoch) as sequence is intentional
        let height = state.block_height() as u64;

        // Check for upgrades in the upgrade_scheduler
        self.maybe_perform_upgrade(&mut state)?;

        // Execute cron message in the cron actor
        let cron_apply_ret = execute_cron_message(&mut state, height)?;

        // Push the current block hash to the chainmetadata actor if possible
        if self.push_chain_meta {
            maybe_push_chain_metadata(&mut state, height)?;
        }

        Ok((state, cron_apply_ret))
    }

    /// Attempts to perform an upgrade if one is scheduled for the current block height,
    /// updating the provided state in-place.
    fn maybe_perform_upgrade(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<()> {
        let chain_id = state.chain_id();
        let block_height: u64 = state.block_height().try_into().unwrap();

        if let Some(upgrade) = self.upgrade_scheduler.get(chain_id, block_height) {
            tracing::info!(?chain_id, height = block_height, "executing an upgrade");

            // Execute the upgrade migration.
            let res = upgrade.execute(state).context("upgrade failed")?;
            if let Some(new_app_version) = res {
                // Update the application's version in the state.
                state.update_app_version(|app_version| *app_version = new_app_version);
                tracing::info!(app_version = state.app_version(), "upgraded app version");
            }
        }

        Ok(())
    }
}

/// Serializes a vector of messages into IPLD-encoded byte vectors.
/// Each message is converted using IPLD encoding.
fn ipld_encode_messages<T: serde::Serialize>(msgs: Vec<T>) -> anyhow::Result<Vec<Vec<u8>>> {
    msgs.into_iter()
        .map(|msg| {
            // Encode each message into IPLD bytes.
            fvm_ipld_encoding::to_vec(&msg).context("failed to encode message as IPLD")
        })
        .collect()
}

/// Decodes a raw IPLD-encoded message into a ChainMessage,
/// then converts it into a VerifiableMessage.
/// First, the raw bytes are deserialized into a ChainMessage.
/// Then, the ChainMessage is transformed into a VerifiableMessage.
fn ipld_decode_signed_message(msg: &[u8]) -> anyhow::Result<VerifiableMessage> {
    // Decode the raw bytes into a ChainMessage.
    let chain_msg = fvm_ipld_encoding::from_slice::<ChainMessage>(msg)
        .context("failed to decode message as ChainMessage")?;

    // Convert the ChainMessage into a VerifiableMessage.
    VerifiableMessage::try_from(chain_msg)
        .context("failed to convert ChainMessage into VerifiableMessage")
}
