use ethers::abi::{Address, Hash};
use ethers::etherscan::{gas, verify};
use ethers::providers::Quorum;
use ipc_api::checkpoint;
use std::sync::Arc;
use tendermint::chain;
use thiserror::Error;

use crate::fvm::state::ipc::GatewayCaller;
use crate::fvm::state::FvmExecState;
use crate::fvm::store::ReadOnlyBlockstore;
use crate::fvm::upgrades::UpgradeScheduler;

use crate::selector::{select_messages_by_gas_limit, select_messages_until_total_bytes};

use crate::types::*;

use crate::{check, check::check_nonce_and_sufficient_balance};
use crate::{
    fvm::FvmMessage,
    signed::{SignedMessageApplyRes, SignedMessageCheckRes, SyntheticMessage},
    verify::{IllegalMessage, VerifiableMessage},
    CheckInterpreter, ExecInterpreter, ProposalInterpreter, QueryInterpreter,
};

use fendermint_vm_message::signed::SignedMessageError;

use anyhow::{anyhow, Chain, Context};
use fendermint_tracing::emit;
use fendermint_vm_actor_interface::{chainmetadata, cron, system};
use fendermint_vm_event::ParentFinalityMissingQuorum;
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::ipc::ParentFinality;
use fendermint_vm_message::ipc::{
    BottomUpCheckpoint, CertifiedMessage, IpcMessage, SignedRelayedMessage,
};
use fendermint_vm_resolver::pool::{ResolveKey, ResolvePool};
use fendermint_vm_topdown::proxy::IPCProviderProxyWithLatency;
use fendermint_vm_topdown::voting::{ValidatorKey, VoteTally};
use fendermint_vm_topdown::{
    CachedFinalityProvider, IPCParentFinality, ParentFinalityProvider, ParentViewProvider, Toggle,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::BLOCK_GAS_LIMIT;

pub type TopDownFinalityProvider = Arc<Toggle<CachedFinalityProvider<IPCProviderProxyWithLatency>>>;

use async_stm::{atomically, StmControl};

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("illegal message: {0}")]
    IllegalMessage(#[from] IllegalMessage),
    #[error("invalid message: {0}")]
    InvalidMessage(String),
    #[error("message verification error")]
    SignedMessageError(#[from] SignedMessageError),
}

pub type CheckpointPool = ResolvePool<CheckpointPoolItem>;

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

pub enum ProcessDecision {
    /// The batch of messages meets the criteria and should be included in the block.
    Accept,
    /// The batch of messages does not meet the criteria and should be rejected.
    Reject,
}

// TODO Karel - handle this type in the check function instead
// pub enum CheckDecision {
//     Accept(FvmCheckRet),
//     Reject,
// }

// Arbitrarily large gas limit for cron (matching how Forest does it, which matches Lotus).
// XXX: Our blocks are not necessarily expected to be 30 seconds apart, so the gas limit might be wrong.
const GAS_LIMIT: u64 = BLOCK_GAS_LIMIT * 10000;

struct Interpreter<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    // TODO Karel - Consider using trait
    /// TODO Karel - move these into separate module for top down
    bottom_up_checkpoint_pool: CheckpointPool,

    /// The parent finality provider for top down checkpoint
    /// TODO Karel - move these into separate module for top down
    parent_finality_provider: TopDownFinalityProvider,
    parent_finality_votes: VoteTally,

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
        bottom_up_checkpoint_pool: CheckpointPool,
        parent_finality_provider: TopDownFinalityProvider,
        parent_finality_votes: VoteTally,
        upgrade_scheduler: UpgradeScheduler<DB>,
        push_chain_meta: bool,
        max_msgs_per_block: usize,
        reject_malformed_proposal: bool,
    ) -> Self {
        Self {
            gateway_caller: GatewayCaller::default(),
            bottom_up_checkpoint_pool,
            parent_finality_provider,
            parent_finality_votes,
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

        let check_ret = check::check_nonce_and_sufficient_balance(&state, &fvm_msg)?;

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
        let mut protocol_msgs = self.messages_from_resolved_bottom_up_checkpoints().await;

        // Add top down message if exists
        if let Some(top_down_message) = self.resolve_top_down_exec_message().await {
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
                        .check_bottom_up_checkpoint_resolved(checkpoint.into())
                        .await
                    {
                        return Ok(ProcessDecision::Reject);
                    }
                }
                ChainMessage::Ipc(IpcMessage::TopDownExec(finality)) => {
                    if !self.check_parent_checkpoint_finalized(finality).await {
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

    async fn begin(
        &self,
        mut state: FvmExecState<DB>,
    ) -> anyhow::Result<(FvmExecState<DB>, FvmApplyRet)> {
        // Block height (FVM epoch) as sequence is intentional
        let height = state.block_height() as u64;

        // Check for upgrades in the upgrade_scheduler
        self.maybe_perform_upgrade(&mut state)?;

        // Execute cron message in the cron actor
        let cron_apply_ret = self.execute_cron_message(&mut state, height)?;

        // Push the current block hash to the chainmetadata actor if possible
        if self.push_chain_meta {
            self.maybe_push_chain_metadata(&mut state, height)?;
        }

        Ok((state, cron_apply_ret))
    }

    async fn check_parent_checkpoint_finalized(&self, msg: ParentFinality) -> bool {
        let prop = IPCParentFinality {
            height: msg.height as u64,
            block_hash: msg.block_hash,
        };
        atomically(|| self.parent_finality_provider.check_proposal(&prop)).await
    }

    async fn check_bottom_up_checkpoint_resolved(
        &self,
        msg: CertifiedMessage<BottomUpCheckpoint>,
    ) -> bool {
        let item = CheckpointPoolItem::BottomUp(msg);

        // We can just look in memory because when we start the application, we should retrieve any
        // pending checkpoints (relayed but not executed) from the ledger, so they should be there.
        // We don't have to validate the checkpoint here, because
        // 1) we validated it when it was relayed, and
        // 2) if a validator proposes something invalid, we can make them pay during execution.
        let is_resolved =
            atomically(|| match self.bottom_up_checkpoint_pool.get_status(&item)? {
                None => Ok(false),
                Some(status) => status.is_resolved(),
            })
            .await;
        is_resolved
    }

    // Checks the bottom up checkpoint pool and returns the messages that are ready for execution
    async fn messages_from_resolved_bottom_up_checkpoints(&self) -> Vec<ChainMessage> {
        let resolved = atomically(|| self.bottom_up_checkpoint_pool.collect_resolved()).await;
        resolved
            .into_iter()
            .map(|checkpoint| match checkpoint {
                CheckpointPoolItem::BottomUp(checkpoint) => {
                    ChainMessage::Ipc(IpcMessage::BottomUpExec(checkpoint))
                }
            })
            .collect()
    }

    /// Prepares a top-down execution message based on the current parent's finality proposal and quorum.
    ///
    /// This function first pauses incoming votes to prevent interference during processing. It then atomically retrieves
    /// both the next parent's proposal and the quorum of votes. If either the parent's proposal or the quorum is missing,
    /// the function returns `None`. When both are available, it selects the finality with the lower block height and wraps
    /// it into a `ChainMessage` for top-down execution.
    async fn resolve_top_down_exec_message(&self) -> Option<ChainMessage> {
        // Prepare top down proposals.
        // Before we try to find a quorum, pause incoming votes. This is optional but if there are lots of votes coming in it might hold up proposals.
        atomically(|| self.parent_finality_votes.pause_votes_until_find_quorum()).await;

        // The pre-requisite for proposal is that there is a quorum of gossiped votes at that height.
        // The final proposal can be at most as high as the quorum, but can be less if we have already,
        // hit some limits such as how many blocks we can propose in a single step.
        let (parent, quorum) = atomically(|| {
            let parent = self.parent_finality_provider.next_proposal()?;

            let quorum = self
                .parent_finality_votes
                .find_quorum()?
                .map(|(height, block_hash)| IPCParentFinality { height, block_hash });

            Ok((parent, quorum))
        })
        .await;

        // If there is no parent proposal, exit early.
        let parent = if let Some(parent) = parent {
            parent
        } else {
            return None;
        };

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

    /// Executes the cron message for the given block height.
    fn execute_cron_message(
        &self,
        state: &mut FvmExecState<DB>,
        height: u64,
    ) -> anyhow::Result<FvmApplyRet> {
        let from = system::SYSTEM_ACTOR_ADDR;
        let to = cron::CRON_ACTOR_ADDR;
        let method_num = cron::Method::EpochTick as u64;
        let gas_limit = GAS_LIMIT;

        let msg = FvmMessage {
            from,
            to,
            sequence: height,
            gas_limit,
            method_num,
            params: Default::default(),
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let (apply_ret, emitters) = state.execute_implicit(msg)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to apply block cron message: {}", err);
        }

        Ok(FvmApplyRet {
            apply_ret,
            emitters,
            from,
            to,
            method_num,
            gas_limit,
        })
    }

    /// Attempts to push chain metadata if a block hash is available.
    fn maybe_push_chain_metadata(
        &self,
        state: &mut FvmExecState<DB>,
        height: u64,
    ) -> anyhow::Result<Option<FvmApplyRet>> {
        let gas_limit = GAS_LIMIT;
        let from = system::SYSTEM_ACTOR_ADDR;
        let to = chainmetadata::CHAINMETADATA_ACTOR_ADDR;
        let method_num = fendermint_actor_chainmetadata::Method::PushBlockHash as u64;

        // Proceed only if the current block has a hash.
        if let Some(block_hash) = state.block_hash() {
            // Serialize the push parameters.
            let params = fvm_ipld_encoding::RawBytes::serialize(
                fendermint_actor_chainmetadata::PushBlockParams {
                    // TODO: this conversion from u64 to i64 should be revisited.
                    epoch: height as i64,
                    block: block_hash,
                },
            )?;

            let msg = FvmMessage {
                from,
                to,
                sequence: height,
                gas_limit,
                method_num,
                params,
                value: Default::default(),
                version: Default::default(),
                gas_fee_cap: Default::default(),
                gas_premium: Default::default(),
            };

            let (apply_ret, emitters) = state.execute_implicit(msg)?;
            if let Some(err) = apply_ret.failure_info {
                anyhow::bail!("failed to apply chainmetadata message: {}", err);
            }

            let fvm_apply_ret = FvmApplyRet {
                apply_ret,
                emitters,
                from,
                to,
                method_num,
                gas_limit,
            };
            Ok(Some(fvm_apply_ret))
        } else {
            // No block hash available; nothing to push.
            Ok(None)
        }
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
