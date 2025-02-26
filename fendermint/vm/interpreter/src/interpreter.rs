use ethers::core::k256::elliptic_curve::rand_core::le;
use ethers::etherscan::verify;
use std::sync::Arc;
use thiserror::Error;

use crate::fvm::state::ipc::GatewayCaller;
use crate::fvm::state::FvmExecState;
use crate::fvm::store::ReadOnlyBlockstore;
use crate::fvm::upgrades::UpgradeScheduler;
use tendermint_rpc::Client;

use crate::selector::{select_messages_by_gas_limit, select_messages_until_total_bytes};
use fendermint_vm_topdown::voting::ValidatorKey;

use crate::check::check_nonce_and_sufficient_balance;
use crate::implicit_messages::{execute_cron_message, push_block_to_chainmeta_actor_if_possible};
use crate::types::*;

use crate::fvm::activity::ValidatorActivityTracker;
use crate::fvm::{FvmApplyRet, PowerUpdates};
use crate::verify::{IllegalMessage, VerifiableMessage};

use crate::fvm::observe::{MsgExec, MsgExecPurpose};
use fendermint_vm_message::signed::SignedMessageError;

use anyhow::Context;

use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::IpcMessage;

use fendermint_vm_actor_interface::system as system_actor;

use fvm_ipld_blockstore::Blockstore;

use crate::bottomup::BottomUpManager;
use crate::topdown::TopDownManager;

use ipc_observability::{emit, measure_time, observe::TracingError, Traceable};

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("illegal message: {0}")]
    IllegalMessage(#[from] IllegalMessage),
    #[error("invalid message: {0}")]
    InvalidMessage(String),
    #[error("message verification error")]
    SignedMessageError(#[from] SignedMessageError),
}

struct Interpreter<DB, C>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
    C: Client + Clone + Send + Sync + 'static,
{
    bottom_up_manager: BottomUpManager<DB, C>,
    top_down_manager: TopDownManager<DB>,

    /// Upgrade scheduler stores all the upgrades to be executed at given heights.
    upgrade_scheduler: UpgradeScheduler<DB>,

    /// Indicate whether some block metadata should be pushed to chainmetadata actor.
    push_block_data_to_chainmeta_actor: bool,
    /// Maximum number of messages to allow in a block.
    max_msgs_per_block: usize,
    /// Should we reject proposals with malformed transactions we cannot parse.
    reject_malformed_proposal: bool,
}

impl<DB, C> Interpreter<DB, C>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
    C: Client + Clone + Send + Sync + 'static,
{
    pub fn new(
        bottom_up_resolver: BottomUpManager<DB, C>,
        top_down_resolver: TopDownManager<DB>,
        upgrade_scheduler: UpgradeScheduler<DB>,
        push_block_data_to_chainmeta_actor: bool,
        max_msgs_per_block: usize,
        reject_malformed_proposal: bool,
    ) -> Self {
        Self {
            bottom_up_manager: bottom_up_resolver,
            top_down_manager: top_down_resolver,
            upgrade_scheduler,
            push_block_data_to_chainmeta_actor: push_block_data_to_chainmeta_actor,
            max_msgs_per_block,
            reject_malformed_proposal,
        }
    }

    /// Check that the message is valid for inclusion in a mempool
    pub async fn check_message(
        &self,
        state: FvmExecState<ReadOnlyBlockstore<Arc<DB>>>,
        msg: Vec<u8>,
        is_recheck: bool,
    ) -> anyhow::Result<CheckResponse> {
        let verifiable_msg = ipld_decode_signed_message(&msg)?;
        let fvm_msg = verifiable_msg.message();

        // Check that the message is valid
        fvm_msg
            .check()
            .map_err(|e| InterpreterError::InvalidMessage(e.to_string()))?;

        // For recheck, we don't need to check the signature or the nonce and balance
        if is_recheck {
            return Ok(CheckResponse::new_ok(&fvm_msg));
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

    /// Prepare messages for inclusion in a block
    pub async fn prepare_messages(
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
            .bottom_up_manager
            .messages_from_resolved_checkpoints()
            .await;

        // Add top down message if parent checkpoint is available
        if let Some(top_down_message) = self
            .top_down_manager
            .message_from_finality_or_quorum()
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

    /// Process messages prepared messages to check they can be included in a block
    pub async fn process_messages(
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
                        .bottom_up_manager
                        .is_checkpoint_resolved(checkpoint.into())
                        .await
                    {
                        return Ok(ProcessDecision::Reject);
                    }
                }
                ChainMessage::Ipc(IpcMessage::TopDownExec(finality)) => {
                    if !self.top_down_manager.is_finality_valid(finality).await {
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

    async fn begin_block(
        &self,
        mut state: FvmExecState<DB>,
    ) -> anyhow::Result<(FvmExecState<DB>, ApplyResponse)> {
        // Block height (FVM epoch) as sequence is intentional
        let height = state.block_height() as u64;

        // Check for upgrades in the upgrade_scheduler
        self.perform_upgrade_if_needed(&mut state)?;

        // Execute cron message in the cron actor
        let cron_apply_ret = execute_cron_message(&mut state, height)?;

        // Push the current block hash to the chainmetadata actor if possible
        if self.push_block_data_to_chainmeta_actor {
            push_block_to_chainmeta_actor_if_possible(&mut state, height)?;
        }

        Ok((state, cron_apply_ret))
    }

    async fn end_block(
        &self,
        mut state: FvmExecState<DB>,
    ) -> anyhow::Result<(FvmExecState<DB>, EndBlockResponse)> {
        // Record the block commitment if a block producer exists.
        if let Some(pubkey) = state.block_producer() {
            state.activity_tracker().record_block_committed(pubkey)?;
        }

        // Attempt to create a bottom-up checkpoint if needed.
        let checkpoint_outcome = self
            .bottom_up_manager
            .create_checkpoint_if_needed(&mut state)?;

        // Process the checkpoint outcome, casting signatures if applicable.
        let (power_updates, block_end_events) = if let Some(outcome) = checkpoint_outcome {
            // Broadcast signatures asynchronously for validators.
            self.bottom_up_manager
                .cast_validator_signatures_for_incomplete_checkpoints(
                    outcome.checkpoint,
                    &mut state,
                )
                .await?;
            (outcome.power_updates, outcome.block_end_events)
        } else {
            (PowerUpdates::default(), BlockEndEvents::default())
        };

        // Finalize the gas market.
        let next_gas_market = state.finalize_gas_market()?;

        // Update any component that needs to know about changes in the power table.
        if !power_updates.0.is_empty() {
            self.top_down_manager
                .update_voting_power_table(&power_updates)
                .await;
        }

        // Assemble and return the response.
        let response = EndBlockResponse {
            power_updates,
            gas_market: next_gas_market,
            events: block_end_events,
        };
        Ok((state, response))
    }

    async fn deliver_message(
        &self,
        mut state: FvmExecState<DB>,
        msg: Vec<u8>,
    ) -> anyhow::Result<(FvmExecState<DB>, ApplyResponse)> {
        let chain_msg = match fvm_ipld_encoding::from_slice::<ChainMessage>(&msg) {
            Ok(msg) => msg,
            Err(e) => {
                // If decoding fails, log a warning if we are configured to reject malformed proposals.
                if self.reject_malformed_proposal {
                    tracing::warn!(
                        error = e.to_string(),
                        "failed to decode delivered message as ChainMessage; This may indicate a node issue."
                    );
                }

                return Err(InterpreterError::InvalidMessage(e.to_string()).into());
            }
        };

        match chain_msg {
            ChainMessage::Signed(msg) => {
                let verifiable_message = VerifiableMessage::Signed(msg);
                verifiable_message.verify(&state.chain_id())?;

                let response = self
                    .execute_signed_message(&mut state, verifiable_message)
                    .await?;

                Ok((state, response))
            }
            ChainMessage::Ipc(msg) => match msg {
                IpcMessage::BottomUpResolve(msg) => {
                    let certified_msg = msg.message.message.clone();
                    let verifiable_message = VerifiableMessage::BottomUp(msg);
                    verifiable_message.verify(&state.chain_id())?;

                    let response = self
                        .execute_signed_message(&mut state, verifiable_message)
                        .await?;

                    // TODO Karel - this might not be necessary
                    // If successful, add the CID to the background resolution pool.
                    let is_success = response.apply_ret.msg_receipt.exit_code.is_success();
                    if is_success {
                        // For now try to get it from the child subnet. If the same comes up for execution, include own.
                        self.bottom_up_manager
                            .add_checkpoint(certified_msg.into())
                            .await;
                    }

                    Ok((state, response))
                }
                IpcMessage::BottomUpExec(_) => {
                    todo!("#197: implement BottomUp checkpoint execution")
                }
                IpcMessage::TopDownExec(p) => {
                    let response = self
                        .top_down_manager
                        .execute_topdown_msg(&mut state, p)
                        .await?;

                    Ok((state, response))
                }
            },
        }
    }

    async fn execute_signed_message(
        &self,
        mut state: &mut FvmExecState<DB>,
        msg: VerifiableMessage,
    ) -> anyhow::Result<ApplyResponse> {
        let msg = msg.message();

        // Execute the message and measure execution time.
        let (apply_ret, emitters, execution_time) = if msg.from == system_actor::SYSTEM_ACTOR_ADDR {
            // For the system actor, use the implicit execution path.
            let (execution_result, execution_time) =
                measure_time(|| state.execute_implicit(msg.clone()));
            let (apply_ret, emitters) = execution_result?;

            (apply_ret, emitters, execution_time)
        } else {
            // For other actors, ensure sufficient gas and then use the explicit execution path.
            if let Err(err) = state.block_gas_tracker().ensure_sufficient_gas(&msg) {
                // This is panic-worthy, but we suppress it to avoid liveness issues.
                // Consider maybe record as evidence for the validator slashing?
                tracing::warn!("insufficient block gas; continuing to avoid halt, but this should've not happened: {}", err);
            }

            let (execution_result, execution_time) =
                measure_time(|| state.execute_explicit(msg.clone()));
            let (apply_ret, emitters) = execution_result?;

            (apply_ret, emitters, execution_time)
        };

        let exit_code = apply_ret.msg_receipt.exit_code.value();

        let response = ApplyResponse {
            apply_ret,
            from: msg.from,
            to: msg.to,
            method_num: msg.method_num,
            gas_limit: msg.gas_limit,
            emitters,
        };

        emit(MsgExec {
            purpose: MsgExecPurpose::Apply,
            height: state.block_height(),
            message: msg,
            duration: execution_time.as_secs_f64(),
            exit_code,
        });

        Ok(response)
    }

    /// Attempts to perform an upgrade if one is scheduled for the current block height,
    /// updating the provided state in-place.
    fn perform_upgrade_if_needed(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<()> {
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
