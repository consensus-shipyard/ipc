use ethers::etherscan::verify;
use ethers::providers::Quorum;
use ipc_api::checkpoint;
use std::sync::Arc;
use tendermint::chain;
use thiserror::Error;

// TODO Karel - consider moving this type elwhere
use crate::chain::ChainEnv;
use crate::fvm::state::ipc::GatewayCaller;
use crate::fvm::state::FvmExecState;
use crate::fvm::store::ReadOnlyBlockstore;

use crate::selector::{select_messages_by_gas_limit, select_messages_until_total_bytes};

use crate::{
    check,
    check::{check_nonce_and_sufficient_balance, FvmCheckRet},
};
use crate::{
    fvm::FvmMessage,
    signed::{SignedMessageApplyRes, SignedMessageCheckRes, SyntheticMessage},
    verify::{IllegalMessage, VerifiableMessage},
    CheckInterpreter, ExecInterpreter, ProposalInterpreter, QueryInterpreter,
};

use fendermint_vm_message::signed::SignedMessageError;

use fendermint_tracing::emit;
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_resolver::pool::{ResolveKey, ResolvePool};
use fvm_ipld_blockstore::Blockstore;

use fendermint_vm_event::ParentFinalityMissingQuorum;

use anyhow::{anyhow, Chain, Context};
use fvm_ipld_encoding::RawBytes;
use fvm_shared::{address::Address, chainid::ChainID, error::ExitCode};

use fendermint_vm_message::ipc::{
    BottomUpCheckpoint, CertifiedMessage, IpcMessage, SignedRelayedMessage,
};

use fendermint_vm_topdown::{
    CachedFinalityProvider, IPCParentFinality, ParentFinalityProvider, ParentViewProvider, Toggle,
};

use fendermint_vm_message::ipc::ParentFinality;

use fendermint_vm_topdown::proxy::IPCProviderProxyWithLatency;

use fendermint_vm_topdown::voting::{ValidatorKey, VoteTally};

use fvm_shared::clock::ChainEpoch;

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

impl From<&CheckpointPoolItem> for ResolveKey {
    fn from(value: &CheckpointPoolItem) -> Self {
        match value {
            CheckpointPoolItem::BottomUp(cp) => {
                (cp.message.subnet_id.clone(), cp.message.bottom_up_messages)
            }
        }
    }
}

struct Interpreter<DB> {
    // TODO Karel - Consider using trait
    bottom_up_checkpoint_pool: CheckpointPool,

    /// The parent finality provider for top down checkpoint
    parent_finality_provider: TopDownFinalityProvider,
    parent_finality_votes: VoteTally,

    // TODO Karel - consider using trait to make this testable
    gateway_caller: GatewayCaller<DB>,

    /// Maximum number of messages to allow in a block.
    max_msgs_per_block: usize,
}

impl<DB> Interpreter<DB>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
{
    pub fn new(
        max_msgs_per_block: usize,
        bottom_up_checkpoint_pool: CheckpointPool,
        parent_finality_provider: TopDownFinalityProvider,
        parent_finality_votes: VoteTally,
    ) -> Self {
        Self {
            gateway_caller: GatewayCaller::default(),
            bottom_up_checkpoint_pool,
            max_msgs_per_block,
            parent_finality_provider,
            parent_finality_votes,
        }
    }

    pub async fn check(
        &self,
        state: FvmExecState<ReadOnlyBlockstore<Arc<DB>>>,
        msg: Vec<u8>,
        is_recheck: bool,
    ) -> anyhow::Result<FvmCheckRet> {
        let chain_msg = fvm_ipld_encoding::from_slice::<ChainMessage>(&msg)?;
        let verifiable_msg = VerifiableMessage::try_from(chain_msg)?;
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

    // Checks the bottom up checkpoint pool and returns the messages that are ready for execution
    async fn resolved_bottom_up_messages(&self) -> Vec<ChainMessage> {
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
    async fn prepare_top_down_exec_message(&self) -> anyhow::Result<Option<ChainMessage>> {
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
            return Ok(None);
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
            return Ok(None);
        };

        // Choose the lower height between the parent's proposal and the quorum.
        let finality = if parent.height <= quorum.height {
            parent
        } else {
            quorum
        };

        Ok(Some(ChainMessage::Ipc(IpcMessage::TopDownExec(
            ParentFinality {
                height: finality.height as ChainEpoch,
                block_hash: finality.block_hash,
            },
        ))))
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
            .filter_map(|msg| {
                // Attempt to decode the message into a ChainMessage.
                fvm_ipld_encoding::from_slice::<ChainMessage>(msg)
                    .map_err(|e| {
                        tracing::warn!(
                            error = %e,
                            "failed to decode message in mempool as ChainMessage"
                        )
                    })
                    .ok()
            })
            .collect();

        // Select messages by block gas limit
        let total_gas_limit = state.block_gas_tracker().available();
        let signed_msgs = select_messages_by_gas_limit(signed_msgs, total_gas_limit);

        // Messages generated by the protocol (e.g. top down, bottom up checkpoints)
        // Add bottom up messages ready for execution directly
        let mut protocol_msgs = self.resolved_bottom_up_messages().await;

        // Add top down message if exists
        if let Some(top_down_message) = self.prepare_top_down_exec_message().await? {
            protocol_msgs.push(top_down_message);
        }

        // Add protocol message first before use messages
        // This ensures that protocol messages are always executed first
        let all_msgs = [protocol_msgs, signed_msgs].concat();

        let mut all_msgs = all_msgs
            .into_iter()
            .map(|msg| {
                fvm_ipld_encoding::to_vec(&msg).context("failed to encode ChainMessage as IPLD")
            })
            .collect::<anyhow::Result<Vec<Vec<u8>>>>()?;

        // Truncate messages
        if all_msgs.len() > self.max_msgs_per_block {
            tracing::warn!(
                max_msgs = self.max_msgs_per_block,
                total_msgs = all_msgs.len(),
                "truncating proposal"
            );
            all_msgs.truncate(self.max_msgs_per_block);
        }

        Ok(select_messages_until_total_bytes(
            all_msgs,
            max_transaction_bytes as usize,
        ))
    }
}
