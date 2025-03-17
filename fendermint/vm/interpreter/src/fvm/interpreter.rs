// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{Context, Result};
use cid::Cid;
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::IpcMessage;
use fendermint_vm_message::query::{FvmQuery, StateParams};
use fendermint_vm_message::signed::SignedMessage;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{self};
use fvm_shared::{address::Address, error::ExitCode};
use std::sync::Arc;
use std::time::Instant;
use tendermint_rpc::Client as TendermintClient;

use crate::errors::*;
use crate::fvm::bottomup::{BottomUpManager, PowerUpdates};
use crate::fvm::gas_estimation::{estimate_gassed_msg, gas_search};
use crate::fvm::topdown::TopDownManager;
use crate::fvm::{
    activity::ValidatorActivityTracker,
    observe::{MsgExec, MsgExecPurpose},
    state::{FvmExecState, FvmQueryState},
    store::ReadOnlyBlockstore,
    upgrades::UpgradeScheduler,
    FvmMessage,
};
use crate::selectors::{select_messages_by_gas_limit, select_messages_until_total_bytes};
use crate::types::*;
use crate::MessagesInterpreter;
use fvm_shared::state::ActorState;
use fvm_shared::ActorID;
use ipc_observability::emit;
use std::convert::TryInto;

use crate::fvm::executions::{
    execute_cron_message, execute_signed_message, push_block_to_chainmeta_actor_if_possible,
};

struct Actor {
    id: ActorID,
    state: ActorState,
}

#[derive(Clone)]
pub struct FvmMessagesInterpreter<DB, C>
where
    DB: Blockstore + Clone + Send + Sync + 'static,
    C: TendermintClient + Clone + Send + Sync + 'static,
{
    bottom_up_manager: BottomUpManager<DB, C>,
    top_down_manager: TopDownManager<DB>,
    upgrade_scheduler: UpgradeScheduler<DB>,

    push_block_data_to_chainmeta_actor: bool,
    max_msgs_per_block: usize,

    gas_overestimation_rate: f64,
    gas_search_step: f64,
}

impl<DB, C> FvmMessagesInterpreter<DB, C>
where
    DB: Blockstore + Clone + Send + Sync + 'static,
    C: TendermintClient + Clone + Send + Sync + 'static,
{
    pub fn new(
        bottom_up_manager: BottomUpManager<DB, C>,
        top_down_manager: TopDownManager<DB>,
        upgrade_scheduler: UpgradeScheduler<DB>,
        push_block_data_to_chainmeta_actor: bool,
        max_msgs_per_block: usize,
        gas_overestimation_rate: f64,
        gas_search_step: f64,
    ) -> Self {
        Self {
            bottom_up_manager,
            top_down_manager,
            upgrade_scheduler,
            push_block_data_to_chainmeta_actor,
            max_msgs_per_block,
            gas_overestimation_rate,
            gas_search_step,
        }
    }

    /// Performs an upgrade if one is scheduled at the current block height.
    fn perform_upgrade_if_needed(&self, state: &mut FvmExecState<DB>) -> Result<()> {
        let chain_id = state.chain_id();
        let block_height: u64 = state.block_height().try_into().unwrap();

        if let Some(upgrade) = self.upgrade_scheduler.get(chain_id, block_height) {
            tracing::info!(?chain_id, height = block_height, "executing an upgrade");
            let res = upgrade.execute(state).context("upgrade failed")?;
            if let Some(new_app_version) = res {
                state.update_app_version(|app_version| *app_version = new_app_version);
                tracing::info!(app_version = state.app_version(), "upgraded app version");
            }
        }

        Ok(())
    }

    fn check_nonce_and_sufficient_balance(
        &self,
        state: &FvmExecState<ReadOnlyBlockstore<DB>>,
        msg: &FvmMessage,
    ) -> Result<CheckResponse> {
        let Actor {
            id: _,
            state: actor,
        } = match self.lookup_actor(state, &msg.from)? {
            Some(actor) => actor,
            None => {
                return Ok(CheckResponse::new(
                    msg,
                    ExitCode::SYS_SENDER_INVALID,
                    None,
                    None,
                ))
            }
        };

        let balance_needed = msg.gas_fee_cap.clone() * msg.gas_limit;
        if actor.balance < balance_needed {
            return Ok(CheckResponse::new(
                msg,
                ExitCode::SYS_INSUFFICIENT_FUNDS,
                Some(format!(
                    "actor balance {} less than needed {}",
                    actor.balance, balance_needed
                )),
                None,
            ));
        }

        if actor.sequence != msg.sequence {
            return Ok(CheckResponse::new(
                msg,
                ExitCode::SYS_SENDER_STATE_INVALID,
                Some(format!(
                    "expected sequence {}, got {}",
                    actor.sequence, msg.sequence
                )),
                None,
            ));
        }

        let priority = state.txn_priority_calculator().priority(msg);
        Ok(CheckResponse::new_ok(msg, priority))
    }

    // Increment sequence
    // TODO - remove this once a new pending state solution is implemented
    fn update_nonce(
        &self,
        state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
        msg: &FvmMessage,
    ) -> Result<()> {
        let Actor {
            id: actor_id,
            state: mut actor,
        } = self
            .lookup_actor(state, &msg.from)?
            .expect("actor must exist");

        let state_tree = state.state_tree_mut();

        actor.sequence += 1;
        state_tree.set_actor(actor_id, actor);

        Ok(())
    }

    fn lookup_actor(
        &self,
        state: &FvmExecState<ReadOnlyBlockstore<DB>>,
        address: &Address,
    ) -> Result<Option<Actor>> {
        let state_tree = state.state_tree();
        let id = match state_tree.lookup_id(address)? {
            Some(id) => id,
            None => return Ok(None),
        };

        let state = match state_tree.get_actor(id)? {
            Some(id) => id,
            None => return Ok(None),
        };

        let actor = Actor { id, state };

        Ok(Some(actor))
    }
}

#[async_trait::async_trait]
impl<DB, C> MessagesInterpreter<DB> for FvmMessagesInterpreter<DB, C>
where
    DB: Blockstore + Clone + Send + Sync + 'static,
    C: TendermintClient + Clone + Send + Sync + 'static,
{
    async fn check_message(
        &self,
        state: &mut FvmExecState<ReadOnlyBlockstore<DB>>,
        msg: Vec<u8>,
        is_recheck: bool,
    ) -> Result<CheckResponse, CheckMessageError> {
        let signed_msg = ipld_decode_signed_message(&msg)?;
        let fvm_msg = signed_msg.message();

        fvm_msg
            .check()
            .map_err(|e| CheckMessageError::InvalidMessage(e.to_string()))?;

        if is_recheck {
            let priority = state.txn_priority_calculator().priority(fvm_msg);
            return Ok(CheckResponse::new_ok(fvm_msg, priority));
        }

        let check_ret = self.check_nonce_and_sufficient_balance(state, fvm_msg)?;

        if check_ret.is_ok() {
            signed_msg.verify(&state.chain_id())?;

            // TODO - remove this once a new pending state solution is implemented
            self.update_nonce(state, fvm_msg)?;
        }

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

    async fn prepare_messages_for_block(
        &self,
        state: FvmExecState<ReadOnlyBlockstore<Arc<DB>>>,
        msgs: Vec<Vec<u8>>,
        max_transaction_bytes: u64,
    ) -> Result<PrepareMessagesResponse, PrepareMessagesError> {
        let signed_msgs = msgs
            .iter()
            .filter_map(|msg| match ipld_decode_signed_message(msg) {
                Ok(vm) => Some(vm),
                Err(e) => {
                    tracing::warn!(error = %e, "failed to decode signable mempool message");
                    None
                }
            })
            .collect::<Vec<_>>();

        let total_gas_limit = state.block_gas_tracker().available();
        let signed_msgs_iter = select_messages_by_gas_limit(signed_msgs, total_gas_limit)
            .into_iter()
            .map(Into::into);

        let top_down_iter = self
            .top_down_manager
            .chain_message_from_finality_or_quorum()
            .await
            .into_iter();

        let mut all_msgs = top_down_iter
            .chain(signed_msgs_iter)
            .map(|msg| fvm_ipld_encoding::to_vec(&msg).context("failed to encode message as IPLD"))
            .collect::<Result<Vec<Vec<u8>>>>()?;

        if all_msgs.len() > self.max_msgs_per_block {
            tracing::info!(
                max_msgs = self.max_msgs_per_block,
                total_msgs = all_msgs.len(),
                "truncating proposal due to message count limit"
            );
            all_msgs.truncate(self.max_msgs_per_block);
        }

        let input_msg_count = all_msgs.len();
        let (all_messages, total_bytes) =
            select_messages_until_total_bytes(all_msgs, max_transaction_bytes as usize);

        if let Some(delta) = input_msg_count.checked_sub(all_messages.len()) {
            if delta > 0 {
                tracing::info!(
                    removed_msgs = delta,
                    max_bytes = max_transaction_bytes,
                    "some messages were removed from the proposal because they exceed the limit"
                );
            }
        }

        Ok(PrepareMessagesResponse {
            messages: all_messages,
            total_bytes,
        })
    }

    async fn attest_block_messages(
        &self,
        state: FvmExecState<ReadOnlyBlockstore<Arc<DB>>>,
        msgs: Vec<Vec<u8>>,
    ) -> Result<AttestMessagesResponse, AttestMessagesError> {
        if msgs.len() > self.max_msgs_per_block {
            tracing::warn!(
                block_msgs = msgs.len(),
                "rejecting block: too many messages"
            );
            return Ok(AttestMessagesResponse::Reject);
        }

        let mut block_gas_usage = 0;
        for msg in msgs {
            match fvm_ipld_encoding::from_slice::<ChainMessage>(&msg) {
                Ok(chain_msg) => match chain_msg {
                    ChainMessage::Ipc(IpcMessage::TopDownExec(finality)) => {
                        if !self.top_down_manager.is_finality_valid(finality).await {
                            return Ok(AttestMessagesResponse::Reject);
                        }
                    }
                    ChainMessage::Signed(signed) => {
                        block_gas_usage += signed.message.gas_limit;
                    }
                },
                Err(e) => {
                    tracing::warn!(error = %e, "failed to decode message in proposal as ChainMessage");
                    return Ok(AttestMessagesResponse::Reject);
                }
            }
        }

        if block_gas_usage > state.block_gas_tracker().available() {
            return Ok(AttestMessagesResponse::Reject);
        }

        Ok(AttestMessagesResponse::Accept)
    }

    async fn begin_block(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> Result<BeginBlockResponse, BeginBlockError> {
        let height = state.block_height() as u64;

        tracing::debug!("trying to perform upgrade");
        self.perform_upgrade_if_needed(state)
            .context("failed to perform upgrade")?;

        tracing::debug!("triggering cron event");
        let cron_applied_message =
            execute_cron_message(state, height).context("failed to trigger cron event")?;

        if self.push_block_data_to_chainmeta_actor {
            tracing::debug!("pushing block data to chainmetadata actor");
            push_block_to_chainmeta_actor_if_possible(state, height)
                .context("failed to push block data to chainmetadata")?;
        }

        Ok(BeginBlockResponse {
            applied_cron_message: cron_applied_message,
        })
    }

    async fn end_block(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> Result<EndBlockResponse, EndBlockError> {
        if let Some(pubkey) = state.block_producer() {
            state.activity_tracker().record_block_committed(pubkey)?;
        }

        let checkpoint_outcome = self.bottom_up_manager.create_checkpoint_if_needed(state)?;

        let (power_updates, block_end_events) = if let Some(outcome) = checkpoint_outcome {
            self.bottom_up_manager
                .cast_validator_signatures_for_incomplete_checkpoints(outcome.checkpoint, state)
                .await?;
            (outcome.power_updates, outcome.block_end_events)
        } else {
            (PowerUpdates::default(), BlockEndEvents::default())
        };

        let next_gas_market = state.finalize_gas_market()?;

        if !power_updates.0.is_empty() {
            self.top_down_manager
                .update_voting_power_table(&power_updates)
                .await;
        }

        let response = EndBlockResponse {
            power_updates,
            gas_market: next_gas_market,
            events: block_end_events,
        };
        Ok(response)
    }

    async fn apply_message(
        &self,
        state: &mut FvmExecState<DB>,
        msg: Vec<u8>,
    ) -> Result<ApplyMessageResponse, ApplyMessageError> {
        let chain_msg = match fvm_ipld_encoding::from_slice::<ChainMessage>(&msg) {
            Ok(msg) => msg,
            Err(e) => {
                tracing::warn!(
                    error = e.to_string(),
                    "failed to decode delivered message as ChainMessage; may indicate a node issue"
                );
                return Err(ApplyMessageError::InvalidMessage(e.to_string()));
            }
        };

        match chain_msg {
            ChainMessage::Signed(msg) => {
                if let Err(e) = msg.verify(&state.chain_id()) {
                    return Err(ApplyMessageError::InvalidSignature(e));
                }

                let applied_message = execute_signed_message(state, msg.clone()).await?;
                let domain_hash = msg.domain_hash(&state.chain_id())?;
                Ok(ApplyMessageResponse {
                    applied_message,
                    domain_hash,
                })
            }
            ChainMessage::Ipc(ipc_msg) => match ipc_msg {
                IpcMessage::TopDownExec(p) => {
                    let applied_message =
                        self.top_down_manager.execute_topdown_msg(state, p).await?;
                    Ok(ApplyMessageResponse {
                        applied_message,
                        domain_hash: None,
                    })
                }
            },
        }
    }

    async fn query(
        &self,
        state: FvmQueryState<DB>,
        query: Query,
    ) -> Result<QueryResponse, QueryError> {
        let query = if query.path.as_str() == "/store" {
            let cid = fvm_ipld_encoding::from_slice::<Cid>(&query.params)
                .context("failed to decode CID")
                .map_err(|e| QueryError::InvalidQuery(e.to_string()))?;
            FvmQuery::Ipld(cid)
        } else {
            fvm_ipld_encoding::from_slice::<FvmQuery>(&query.params)
                .context("failed to decode FvmQuery")?
        };

        match query {
            FvmQuery::Ipld(cid) => {
                let data = state.store_get(&cid)?;
                tracing::info!(
                    height = state.block_height(),
                    cid = cid.to_string(),
                    found = data.is_some(),
                    "query IPLD"
                );
                Ok(QueryResponse::Ipld(data))
            }
            FvmQuery::ActorState(address) => {
                let (state, ret) = state.actor_state(&address).await?;
                tracing::info!(
                    height = state.block_height(),
                    addr = address.to_string(),
                    found = ret.is_some(),
                    "query actor state"
                );
                Ok(QueryResponse::ActorState(ret.map(Box::new)))
            }
            FvmQuery::Call(msg) => {
                let from = msg.from;
                let to = msg.to;
                let method_num = msg.method_num;
                let gas_limit = msg.gas_limit;
                let start = Instant::now();
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
                let response = AppliedMessage {
                    apply_ret,
                    from,
                    to,
                    method_num,
                    gas_limit,
                    emitters,
                };
                Ok(QueryResponse::Call(Box::new(response)))
            }
            FvmQuery::EstimateGas(mut msg) => {
                tracing::info!(
                    height = state.block_height(),
                    to = msg.to.to_string(),
                    from = msg.from.to_string(),
                    method_num = msg.method_num,
                    "query estimate gas"
                );
                match estimate_gassed_msg(state, &mut msg, self.gas_overestimation_rate).await? {
                    (_, Some(est)) => Ok(QueryResponse::EstimateGas(est)),
                    (state, None) => {
                        let (_, mut est) = gas_search(state, &msg, self.gas_search_step).await?;
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
}

/// Decodes raw bytes into a SignedMessage by first decoding into a ChainMessage.
/// If the ChainMessage is not signed, returns an error.
fn ipld_decode_signed_message(msg: &[u8]) -> Result<SignedMessage> {
    let chain_msg = fvm_ipld_encoding::from_slice::<ChainMessage>(msg).map_err(|_| {
        CheckMessageError::InvalidMessage(
            "failed to IPLD decode message as ChainMessage".to_string(),
        )
    })?;

    match chain_msg {
        ChainMessage::Signed(msg) => Ok(msg),
        other => Err(CheckMessageError::IllegalMessage(format!("{:?}", other)).into()),
    }
}
