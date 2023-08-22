use anyhow::anyhow;
// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;

use fendermint_vm_message::{chain::ChainMessage, signed::SignedMessage};

use crate::{
    signed::{SignedMessageApplyRet, SignedMessageCheckRet},
    CheckInterpreter, ExecInterpreter, GenesisInterpreter, ProposalInterpreter, QueryInterpreter,
};

/// A message a user is not supposed to send.
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
    // TODO: The state can include the IPLD Resolver mempool, for example by using STM
    // to implement a shared memory space.
    type State = ();
    type Message = ChainMessage;

    /// Check whether there are any "ready" messages in the IPLD resolution mempool which can be appended to the proposal.
    ///
    /// We could also use this to select the most profitable user transactions, within the gas limit. We can also take into
    /// account the transactions which are part of top-down or bottom-up checkpoints, to stay within gas limits.
    async fn prepare(
        &self,
        _state: Self::State,
        msgs: Vec<Self::Message>,
    ) -> anyhow::Result<Vec<Self::Message>> {
        // For now this is just a placeholder.
        Ok(msgs)
    }

    /// Perform finality checks on top-down transactions and availability checks on bottom-up transactions.
    async fn process(
        &self,
        _state: Self::State,
        _msgs: Vec<Self::Message>,
    ) -> anyhow::Result<bool> {
        // For now this is just a placeholder.
        Ok(true)
    }
}

#[async_trait]
impl<I> ExecInterpreter for ChainMessageInterpreter<I>
where
    I: ExecInterpreter<Message = SignedMessage, DeliverOutput = SignedMessageApplyRet>,
{
    type State = I::State;
    type Message = ChainMessage;
    type BeginOutput = I::BeginOutput;
    type DeliverOutput = ChainMessageApplyRet;
    type EndOutput = I::EndOutput;

    async fn deliver(
        &self,
        state: Self::State,
        msg: Self::Message,
    ) -> anyhow::Result<(Self::State, Self::DeliverOutput)> {
        match msg {
            ChainMessage::Signed(msg) => {
                let (state, ret) = self.inner.deliver(state, *msg).await?;
                Ok((state, ChainMessageApplyRet::Signed(ret)))
            }
            ChainMessage::ForExecution(_) | ChainMessage::ForResolution(_) => {
                // This only happens if a validator is malicious or we have made a programming error.
                // I expect for now that we don't run with untrusted validators, so it's okay to quit.
                Err(anyhow!(
                    "The handling of ForExecution and ForResolution is not yet implemented."
                ))
            }
        }
    }

    async fn begin(&self, state: Self::State) -> anyhow::Result<(Self::State, Self::BeginOutput)> {
        self.inner.begin(state).await
    }

    async fn end(&self, state: Self::State) -> anyhow::Result<(Self::State, Self::EndOutput)> {
        self.inner.end(state).await
    }
}

#[async_trait]
impl<I> CheckInterpreter for ChainMessageInterpreter<I>
where
    I: CheckInterpreter<Message = SignedMessage, Output = SignedMessageCheckRet>,
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
                let (state, ret) = self.inner.check(state, *msg, is_recheck).await?;

                Ok((state, Ok(ret)))
            }
            ChainMessage::ForExecution(_) | ChainMessage::ForResolution(_) => {
                // Users cannot send these messages, only validators can propose them in blocks.
                Ok((state, Err(IllegalMessage)))
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
