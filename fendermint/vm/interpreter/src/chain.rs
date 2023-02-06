// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;

use fendermint_vm_message::{chain::ChainMessage, signed::SignedMessage};

use crate::{signed::SignedMesssageApplyRet, Interpreter};

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

pub enum ChainMessageApplyRet {
    Signed(SignedMesssageApplyRet),
}

#[async_trait]
impl<I> Interpreter for ChainMessageInterpreter<I>
where
    I: Interpreter<Message = SignedMessage, DeliverOutput = SignedMesssageApplyRet>,
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
                let (state, ret) = self.inner.deliver(state, msg).await?;
                Ok((state, ChainMessageApplyRet::Signed(ret)))
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
