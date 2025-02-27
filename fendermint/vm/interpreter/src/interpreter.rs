use crate::fvm::state::ipc::GatewayCaller;
use crate::fvm::state::FvmExecState;
use crate::fvm::store::ReadOnlyBlockstore;
use crate::fvm::upgrades::UpgradeScheduler;
use cid::Cid;
use ethers::core::k256::elliptic_curve::rand_core::le;
use ethers::etherscan::verify;
use num_traits::Signed;
use std::any;
use std::sync::Arc;
use std::time::Instant;
use tendermint_rpc::Client as TendermintClient;
use thiserror::Error;

use crate::fvm::state::FvmQueryState;
use crate::selector::{select_messages_by_gas_limit, select_messages_until_total_bytes};
use fendermint_vm_topdown::voting::ValidatorKey;
use fvm_ipld_encoding::RawBytes;

use crate::check::check_nonce_and_sufficient_balance;
use crate::implicit_messages::{execute_cron_message, push_block_to_chainmeta_actor_if_possible};
use crate::types::*;

use crate::fvm::activity::ValidatorActivityTracker;
use crate::fvm::PowerUpdates;

use crate::fvm::observe::{MsgExec, MsgExecPurpose};
use fendermint_vm_message::signed::{SignedMessage, SignedMessageError};

use anyhow::Context;

use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::IpcMessage;

use fendermint_vm_actor_interface::system as system_actor;

use fvm_ipld_blockstore::Blockstore;

use crate::bottomup::BottomUpManager;
use crate::topdown::TopDownManager;

use fendermint_vm_message::query::{ActorState, FvmQuery, GasEstimate, StateParams};
use ipc_observability::{emit, measure_time, observe::TracingError, Traceable};

use fvm_shared::{
    bigint::BigInt, econ::TokenAmount, error::ExitCode, message::Message, ActorID, BLOCK_GAS_LIMIT,
};

use num_traits::Zero;

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("illegal message: {0}")]
    IllegalMessage(String),
    #[error("invalid message: {0}")]
    InvalidMessage(String),
    #[error("invalid signature")]
    InvalidSignature(#[from] SignedMessageError),
    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub struct MessagesInterpreter<DB, C>
where
    DB: Blockstore + Clone + Send + Sync + 'static,
    C: TendermintClient + Clone + Send + Sync + 'static,
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

    /// Overestimation rate applied to gas to ensure that the
    /// message goes through in the gas estimation.
    gas_overestimation_rate: f64,
    /// Gas search step increase used to find the optimal gas limit.
    /// It determines how fine-grained we want the gas estimation to be.
    gas_search_step: f64,
}

impl<DB, C> MessagesInterpreter<DB, C>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
    C: TendermintClient + Clone + Send + Sync + 'static,
{
    pub fn new(
        bottom_up_resolver: BottomUpManager<DB, C>,
        top_down_resolver: TopDownManager<DB>,
        upgrade_scheduler: UpgradeScheduler<DB>,
        push_block_data_to_chainmeta_actor: bool,
        max_msgs_per_block: usize,
        reject_malformed_proposal: bool,
        gas_overestimation_rate: f64,
        gas_search_step: f64,
    ) -> Self {
        Self {
            bottom_up_manager: bottom_up_resolver,
            top_down_manager: top_down_resolver,
            upgrade_scheduler,
            push_block_data_to_chainmeta_actor: push_block_data_to_chainmeta_actor,
            max_msgs_per_block,
            reject_malformed_proposal,
            gas_overestimation_rate,
            gas_search_step,
        }
    }

    /// Check that the message is valid for inclusion in a mempool
    pub async fn check_message(
        &self,
        state: Arc<FvmExecState<ReadOnlyBlockstore<DB>>>,
        msg: Vec<u8>,
        is_recheck: bool,
    ) -> anyhow::Result<CheckResponse, InterpreterError> {
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
        let signed_msgs_iter = select_messages_by_gas_limit(signed_msgs, total_gas_limit)
            .into_iter()
            .map(Into::into);

        let top_down_iter = self
            .top_down_manager
            .chain_message_from_finality_or_quorum()
            .await
            .into_iter();

        // Add top down message first (if possible) and then signed messages.
        let mut all_msgs = top_down_iter
            .chain(signed_msgs_iter)
            .map(ipld_encode_message)
            .collect::<anyhow::Result<Vec<Vec<u8>>>>()?;

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

    pub async fn begin_block(
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

    pub async fn end_block(
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

    pub async fn deliver_message(
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
                msg.verify(&state.chain_id())?;

                let response = self.execute_signed_message(&mut state, msg).await?;

                Ok((state, response))
            }
            ChainMessage::Ipc(msg) => match msg {
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

    pub async fn query(
        &self,
        state: FvmQueryState<DB>,
        query: Query,
    ) -> anyhow::Result<QueryResponse> {
        let query = if query.path.as_str() == "/store" {
            // According to the docstrings, the application MUST interpret `/store` as a query on the underlying KV store.
            let cid = fvm_ipld_encoding::from_slice::<Cid>(&query.params)?;
            FvmQuery::Ipld(cid)
        } else {
            // Otherwise ignore the path for now. The docs also say that the query bytes can be used in lieu of the path,
            // so it's okay to have two ways to send IPLD queries: either by using the `/store` path and sending a CID,
            // or by sending the appropriate `FvmQuery`.
            fvm_ipld_encoding::from_slice::<FvmQuery>(&query.params)?
        };

        match query {
            FvmQuery::Ipld(cid) => {
                let data = state.store_get(&cid)?;
                tracing::info!(
                    height = state.block_height(),
                    pending = state.pending(),
                    cid = cid.to_string(),
                    found = data.is_some(),
                    "query IPLD"
                );
                let out = QueryResponse::Ipld(data);
                Ok(out)
            }
            FvmQuery::ActorState(address) => {
                let (state, ret) = state.actor_state(&address).await?;
                tracing::info!(
                    height = state.block_height(),
                    pending = state.pending(),
                    addr = address.to_string(),
                    found = ret.is_some(),
                    "query actor state"
                );
                let out = QueryResponse::ActorState(ret.map(Box::new));
                Ok(out)
            }
            FvmQuery::Call(msg) => {
                let from = msg.from;
                let to = msg.to;
                let method_num = msg.method_num;
                let gas_limit = msg.gas_limit;

                let start = Instant::now();
                // Do not stack effects
                let (state, (apply_ret, emitters)) = state.call(*msg.clone()).await?;
                let latency = start.elapsed().as_secs_f64();
                let exit_code = apply_ret.msg_receipt.exit_code.value();

                emit(MsgExec {
                    purpose: MsgExecPurpose::Call,
                    height: state.block_height(),
                    message: *msg,
                    duration: latency,
                    exit_code,
                });

                let response = ApplyResponse {
                    apply_ret,
                    from,
                    to,
                    method_num,
                    gas_limit,
                    emitters,
                };

                let out = QueryResponse::Call(Box::new(response));
                Ok(out)
            }
            FvmQuery::EstimateGas(mut msg) => {
                tracing::info!(
                    height = state.block_height(),
                    pending = state.pending(),
                    to = msg.to.to_string(),
                    from = msg.from.to_string(),
                    method_num = msg.method_num,
                    "query estimate gas"
                );
                // Populate gas message parameters.

                match self.estimate_gassed_msg(state, &mut msg).await? {
                    (_, Some(est)) => {
                        // return immediately if something is returned,
                        // it means that the message failed to execute so there's
                        // no point on estimating the gas.
                        Ok(QueryResponse::EstimateGas(est))
                    }
                    (state, None) => {
                        // perform a gas search for an accurate value
                        let (_, mut est) = self.gas_search(state, &msg).await?;
                        // we need an additional overestimation for the case where
                        // the exact value is returned as part of the gas search
                        // (for some reason with subsequent calls sometimes this is the case).
                        est.gas_limit =
                            (est.gas_limit as f64 * self.gas_overestimation_rate) as u64;

                        Ok(QueryResponse::EstimateGas(est))
                    }
                }
            }
            FvmQuery::StateParams => {
                let state_params = state.state_params();
                let state_params = StateParams {
                    base_fee: state_params.base_fee.clone(),
                    circ_supply: state_params.circ_supply.clone(),
                    chain_id: state_params.chain_id,
                    network_version: state_params.network_version,
                };
                Ok(QueryResponse::StateParams(state_params))
            }
            FvmQuery::BuiltinActors => {
                let (_, ret) = state.builtin_actors().await?;
                Ok(QueryResponse::BuiltinActors(ret))
            }
        }
    }

    async fn execute_signed_message(
        &self,
        state: &mut FvmExecState<DB>,
        msg: SignedMessage,
    ) -> anyhow::Result<ApplyResponse> {
        let msg = msg.into_message();

        // Execute the message and measure execution time.
        // TODO: This should not be possible? As it is a signed message.
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

    async fn estimate_gassed_msg(
        &self,
        state: FvmQueryState<DB>,
        msg: &mut Message,
    ) -> anyhow::Result<(FvmQueryState<DB>, Option<GasEstimate>)> {
        // Setting BlockGasLimit as initial limit for gas estimation
        msg.gas_limit = BLOCK_GAS_LIMIT;

        // With unlimited gas we are probably better off setting the prices to zero.
        let gas_premium = msg.gas_premium.clone();
        let gas_fee_cap = msg.gas_fee_cap.clone();
        msg.gas_premium = TokenAmount::zero();
        msg.gas_fee_cap = TokenAmount::zero();

        let start = Instant::now();
        // estimate the gas limit and assign it to the message
        // revert any changes because we'll repeat the estimation
        let (state, (ret, _)) = state.call(msg.clone()).await?;
        let latency = start.elapsed().as_secs_f64();

        emit(MsgExec {
            purpose: MsgExecPurpose::Estimate,
            height: state.block_height(),
            message: msg.clone(),
            duration: latency,
            exit_code: ret.msg_receipt.exit_code.value(),
        });

        if !ret.msg_receipt.exit_code.is_success() {
            // if the message fail we can't estimate the gas.
            return Ok((
                state,
                Some(GasEstimate {
                    exit_code: ret.msg_receipt.exit_code,
                    info: ret.failure_info.map(|x| x.to_string()).unwrap_or_default(),
                    return_data: ret.msg_receipt.return_data,
                    gas_limit: 0,
                }),
            ));
        }

        msg.gas_limit = (ret.msg_receipt.gas_used as f64 * self.gas_overestimation_rate) as u64;

        if gas_premium.is_zero() {
            // We need to set the gas_premium to some value other than zero for the
            // gas estimation to work accurately (I really don't know why this is
            // the case but after a lot of testing, setting this value to zero rejects the transaction)
            msg.gas_premium = TokenAmount::from_nano(BigInt::from(1));
        } else {
            msg.gas_premium = gas_premium;
        }

        // Same for the gas_fee_cap, not setting the fee cap leads to the message
        // being sent after the estimation to fail.
        if gas_fee_cap.is_zero() {
            // TODO: In Lotus historical values of the base fee and a more accurate overestimation is performed
            // for the fee cap. If we issues with messages going through let's consider the historical analysis.
            // For now we are disregarding the base_fee so I don't think this is needed here.
            // Filecoin clamps the gas premium at GasFeeCap - BaseFee, if lower than the
            // specified premium. Returns 0 if GasFeeCap is less than BaseFee.
            // see https://spec.filecoin.io/#section-systems.filecoin_vm.message.message-semantic-validation
            msg.gas_fee_cap = msg.gas_premium.clone();
        } else {
            msg.gas_fee_cap = gas_fee_cap;
        }

        Ok((state, None))
    }

    // This function performs a simpler implementation of the gas search than the one used in Lotus.
    // Instead of using historical information of the gas limit for other messages, it searches
    // for a valid gas limit for the current message in isolation.
    async fn gas_search(
        &self,
        mut state: FvmQueryState<DB>,
        msg: &Message,
    ) -> anyhow::Result<(FvmQueryState<DB>, GasEstimate)> {
        let mut curr_limit = msg.gas_limit;

        loop {
            let (st, est) = self
                .estimation_call_with_limit(state, msg.clone(), curr_limit)
                .await?;

            if let Some(est) = est {
                return Ok((st, est));
            } else {
                state = st;
            }

            curr_limit = (curr_limit as f64 * self.gas_search_step) as u64;
            if curr_limit > BLOCK_GAS_LIMIT {
                let est = GasEstimate {
                    exit_code: ExitCode::OK,
                    info: "".to_string(),
                    return_data: RawBytes::default(),
                    gas_limit: BLOCK_GAS_LIMIT,
                };
                return Ok((state, est));
            }
        }

        // TODO: For a more accurate gas estimation we could track the low and the high
        // of the search and make higher steps (e.g. `GAS_SEARCH_STEP = 2`).
        // Once an interval is found of [low, high] for which the message
        // succeeds, we make a finer-grained within that interval.
        // At this point, I don't think is worth being that accurate as long as it works.
    }

    async fn estimation_call_with_limit(
        &self,
        state: FvmQueryState<DB>,
        mut msg: Message,
        limit: u64,
    ) -> anyhow::Result<(FvmQueryState<DB>, Option<GasEstimate>)> {
        msg.gas_limit = limit;
        // set message nonce to zero so the right one is picked up
        msg.sequence = 0;

        let start = Instant::now();
        let (state, (apply_ret, _)) = state.call(msg.clone()).await?;
        let latency = start.elapsed().as_secs_f64();

        let ret = GasEstimate {
            exit_code: apply_ret.msg_receipt.exit_code,
            info: apply_ret
                .failure_info
                .map(|x| x.to_string())
                .unwrap_or_default(),
            return_data: apply_ret.msg_receipt.return_data,
            gas_limit: apply_ret.msg_receipt.gas_used,
        };

        emit(MsgExec {
            purpose: MsgExecPurpose::Estimate,
            height: state.block_height(),
            message: msg,
            duration: latency,
            exit_code: ret.exit_code.value(),
        });

        // if the message succeeded or failed with a different error than `SYS_OUT_OF_GAS`,
        // immediately return as we either succeeded finding the right gas estimation,
        // or something non-related happened.
        if ret.exit_code == ExitCode::OK || ret.exit_code != ExitCode::SYS_OUT_OF_GAS {
            return Ok((state, Some(ret)));
        }

        Ok((state, None))
    }
}

// TODO Karel - these should use their custom errors?

/// Serializes a message into IPLD-encoded byte vector.
/// Each message is converted using IPLD encoding.
fn ipld_encode_message<T: serde::Serialize>(msg: T) -> anyhow::Result<Vec<u8>> {
    fvm_ipld_encoding::to_vec(&msg).context("failed to encode message as IPLD")
}

/// Decodes a raw IPLD-encoded message into a ChainMessage,
/// then converts it into a SignedMessage.
/// First, the raw bytes are deserialized into a ChainMessage.
/// Then, the ChainMessage is transformed into a SignedMessage.
fn ipld_decode_signed_message(msg: &[u8]) -> anyhow::Result<SignedMessage> {
    // Decode the raw bytes into a ChainMessage.
    let chain_msg = fvm_ipld_encoding::from_slice::<ChainMessage>(msg).map_err(|e| {
        InterpreterError::InvalidMessage(
            "failed to IPLD decode message as ChainMessage".to_string(),
        )
    })?;

    match chain_msg {
        ChainMessage::Signed(msg) => Ok(msg),
        other => Err(InterpreterError::IllegalMessage(format!("{:?}", other)).into()),
    }
}
