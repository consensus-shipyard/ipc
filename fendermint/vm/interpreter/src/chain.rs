// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use crate::{
    fvm::FvmMessage,
    signed::{SignedMessageApplyRet, SignedMessageCheckRet, SyntheticMessage, VerifiableMessage},
    CheckInterpreter, ExecInterpreter, GenesisInterpreter, ProposalInterpreter, QueryInterpreter,
};
use anyhow::Context;
use async_stm::atomically;
use async_trait::async_trait;
use fendermint_vm_actor_interface::ipc;
use fendermint_vm_message::{
    chain::ChainMessage,
    ipc::{BottomUpCheckpoint, CertifiedMessage, IpcMessage, SignedRelayedMessage},
};
use fendermint_vm_resolver::pool::{ResolveKey, ResolvePool};
use fvm_ipld_encoding::RawBytes;
use fvm_shared::econ::TokenAmount;
use num_traits::Zero;

/// A resolution pool for bottom-up and top-down checkpoints.
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

/// A user sent a transaction which they are not allowed to do.
pub struct IllegalMessage;

// For now this is the only option, later we can expand.
pub enum ChainMessageApplyRet {
    Signed(SignedMessageApplyRet),
}

/// We only allow signed messages into the mempool.
pub type ChainMessageCheckRet = Result<SignedMessageCheckRet, IllegalMessage>;

/// Interpreter working on chain messages; in the future it will schedule
/// CID lookups to turn references into self-contained user or cross messages.
#[derive(Clone)]
pub struct ChainMessageInterpreter<I> {
    inner: I,
}

impl<I> ChainMessageInterpreter<I> {
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<I> ProposalInterpreter for ChainMessageInterpreter<I>
where
    I: Sync + Send,
{
    type State = CheckpointPool;
    type Message = ChainMessage;

    /// Check whether there are any "ready" messages in the IPLD resolution mempool which can be appended to the proposal.
    ///
    /// We could also use this to select the most profitable user transactions, within the gas limit. We can also take into
    /// account the transactions which are part of top-down or bottom-up checkpoints, to stay within gas limits.
    async fn prepare(
        &self,
        pool: Self::State,
        mut msgs: Vec<Self::Message>,
    ) -> anyhow::Result<Vec<Self::Message>> {
        // Collect resolved CIDs ready to be proposed from the pool.
        let ckpts = atomically(|| pool.collect_resolved()).await;

        // Create transactions ready to be included on the chain.
        let ckpts = ckpts.into_iter().map(|ckpt| match ckpt {
            CheckpointPoolItem::BottomUp(ckpt) => ChainMessage::Ipc(IpcMessage::BottomUpExec(ckpt)),
        });

        // Append at the end - if we run out of block space, these are going to be reproposed in the next block.
        msgs.extend(ckpts);
        Ok(msgs)
    }

    /// Perform finality checks on top-down transactions and availability checks on bottom-up transactions.
    async fn process(&self, pool: Self::State, msgs: Vec<Self::Message>) -> anyhow::Result<bool> {
        for msg in msgs {
            if let ChainMessage::Ipc(IpcMessage::BottomUpExec(msg)) = msg {
                let item = CheckpointPoolItem::BottomUp(msg);

                // We can just look in memory because when we start the application, we should retrieve any
                // pending checkpoints (relayed but not executed) from the ledger, so they should be there.
                // We don't have to validate the checkpoint here, because
                // 1) we validated it when it was relayed, and
                // 2) if a validator proposes something invalid, we can make them pay during execution.
                let is_resolved = atomically(|| match pool.get_status(&item)? {
                    None => Ok(false),
                    Some(status) => status.is_resolved(),
                })
                .await;

                if !is_resolved {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}

#[async_trait]
impl<I> ExecInterpreter for ChainMessageInterpreter<I>
where
    I: ExecInterpreter<Message = VerifiableMessage, DeliverOutput = SignedMessageApplyRet>,
{
    // The state consists of the resolver pool, which this interpreter needs, and the rest of the
    // state which the inner interpreter uses. This is a technical solution because the pool doesn't
    // fit with the state we use for execution messages further down the stack, which depend on block
    // height and are used in queries as well.
    type State = (CheckpointPool, I::State);
    type Message = ChainMessage;
    type BeginOutput = I::BeginOutput;
    type DeliverOutput = ChainMessageApplyRet;
    type EndOutput = I::EndOutput;

    async fn deliver(
        &self,
        (pool, state): Self::State,
        msg: Self::Message,
    ) -> anyhow::Result<(Self::State, Self::DeliverOutput)> {
        match msg {
            ChainMessage::Signed(msg) => {
                let (state, ret) = self
                    .inner
                    .deliver(state, VerifiableMessage::Signed(msg))
                    .await?;
                Ok(((pool, state), ChainMessageApplyRet::Signed(ret)))
            }
            ChainMessage::Ipc(msg) => match msg {
                IpcMessage::BottomUpResolve(msg) => {
                    let smsg = relayed_bottom_up_ckpt_to_fvm(&msg)
                        .context("failed to syntesize FVM message")?;

                    // Let the FVM validate the checkpoint quorum certificate and take note of the relayer for rewards.
                    let (state, ret) = self
                        .inner
                        .deliver(state, VerifiableMessage::Synthetic(smsg))
                        .await?;

                    // If successful, add the CID to the background resolution pool.
                    let is_success = match ret {
                        Ok(ref ret) => ret.apply_ret.msg_receipt.exit_code.is_success(),
                        Err(_) => false,
                    };

                    if is_success {
                        atomically(|| {
                            pool.add(CheckpointPoolItem::BottomUp(msg.message.message.clone()))
                        })
                        .await;
                    }

                    // We can use the same result type for now, it's isomorphic.
                    Ok(((pool, state), ChainMessageApplyRet::Signed(ret)))
                }
                IpcMessage::BottomUpExec(_) => {
                    todo!("#197: implement BottomUp checkpoint execution")
                }
                IpcMessage::TopDown => {
                    todo!("implement TopDown handling; this is just a placeholder")
                }
            },
        }
    }

    async fn begin(
        &self,
        (pool, state): Self::State,
    ) -> anyhow::Result<(Self::State, Self::BeginOutput)> {
        let (state, out) = self.inner.begin(state).await?;
        Ok(((pool, state), out))
    }

    async fn end(
        &self,
        (pool, state): Self::State,
    ) -> anyhow::Result<(Self::State, Self::EndOutput)> {
        let (state, out) = self.inner.end(state).await?;
        Ok(((pool, state), out))
    }
}

#[async_trait]
impl<I> CheckInterpreter for ChainMessageInterpreter<I>
where
    I: CheckInterpreter<Message = VerifiableMessage, Output = SignedMessageCheckRet>,
{
    type State = I::State;
    type Message = ChainMessage;
    type Output = ChainMessageCheckRet;

    async fn check(
        &self,
        state: Self::State,
        msg: Self::Message,
        is_recheck: bool,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        match msg {
            ChainMessage::Signed(msg) => {
                let (state, ret) = self
                    .inner
                    .check(state, VerifiableMessage::Signed(msg), is_recheck)
                    .await?;

                Ok((state, Ok(ret)))
            }
            ChainMessage::Ipc(msg) => {
                match msg {
                    IpcMessage::BottomUpResolve(msg) => {
                        let msg = relayed_bottom_up_ckpt_to_fvm(&msg)
                            .context("failed to syntesize FVM message")?;

                        let (state, ret) = self
                            .inner
                            .check(state, VerifiableMessage::Synthetic(msg), is_recheck)
                            .await?;

                        Ok((state, Ok(ret)))
                    }
                    IpcMessage::TopDown | IpcMessage::BottomUpExec(_) => {
                        // Users cannot send these messages, only validators can propose them in blocks.
                        Ok((state, Err(IllegalMessage)))
                    }
                }
            }
        }
    }
}

#[async_trait]
impl<I> QueryInterpreter for ChainMessageInterpreter<I>
where
    I: QueryInterpreter,
{
    type State = I::State;
    type Query = I::Query;
    type Output = I::Output;

    async fn query(
        &self,
        state: Self::State,
        qry: Self::Query,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        self.inner.query(state, qry).await
    }
}

#[async_trait]
impl<I> GenesisInterpreter for ChainMessageInterpreter<I>
where
    I: GenesisInterpreter,
{
    type State = I::State;
    type Genesis = I::Genesis;
    type Output = I::Output;

    async fn init(
        &self,
        state: Self::State,
        genesis: Self::Genesis,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        self.inner.init(state, genesis).await
    }
}

/// Convert a signed relayed bottom-up checkpoint to a syntetic message we can send to the FVM.
///
/// By mapping to an FVM message we invoke the right contract to validate the checkpoint,
/// and automatically charge the relayer gas for the execution of the check, but not the
/// execution of the cross-messages, which aren't part of the payload.
fn relayed_bottom_up_ckpt_to_fvm(
    relayed: &SignedRelayedMessage<CertifiedMessage<BottomUpCheckpoint>>,
) -> anyhow::Result<SyntheticMessage> {
    // TODO #192: Convert the checkpoint to what the actor expects.
    let params = RawBytes::default();

    let msg = FvmMessage {
        version: 0,
        from: relayed.message.relayer,
        to: ipc::GATEWAY_ACTOR_ADDR,
        sequence: relayed.message.sequence,
        value: TokenAmount::zero(),
        method_num: ipc::gateway::METHOD_INVOKE_CONTRACT,
        params,
        gas_limit: relayed.message.gas_limit,
        gas_fee_cap: relayed.message.gas_fee_cap.clone(),
        gas_premium: relayed.message.gas_premium.clone(),
    };

    let msg = SyntheticMessage::new(msg, &relayed.message, relayed.signature.clone())
        .context("failed to create syntetic message")?;

    Ok(msg)
}
