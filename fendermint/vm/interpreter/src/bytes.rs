// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;

use fendermint_vm_message::chain::ChainMessage;

use crate::{chain::ChainMessageApplyRet, Interpreter};

pub type BytesMessageApplyRet = Result<ChainMessageApplyRet, fvm_ipld_encoding::Error>;

/// Interpreter working on raw bytes.
#[derive(Clone)]
pub struct BytesMessageInterpreter<I> {
    inner: I,
}

impl<I> BytesMessageInterpreter<I> {
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<I> Interpreter for BytesMessageInterpreter<I>
where
    I: Interpreter<Message = ChainMessage, DeliverOutput = ChainMessageApplyRet>,
{
    type State = I::State;
    type Message = Vec<u8>;
    type BeginOutput = I::BeginOutput;
    type DeliverOutput = BytesMessageApplyRet;
    type EndOutput = I::EndOutput;

    async fn deliver(
        &self,
        state: Self::State,
        msg: Self::Message,
    ) -> anyhow::Result<(Self::State, Self::DeliverOutput)> {
        match fvm_ipld_encoding::from_slice::<ChainMessage>(&msg) {
            Err(e) =>
            // TODO: Punish the validator for including rubbish.
            // There is always the possibility that our codebase is incompatible,
            // but then we'll have a consensu failure.
            {
                Ok((state, Err(e)))
            }
            Ok(msg) => {
                let (state, ret) = self.inner.deliver(state, msg).await?;
                Ok((state, Ok(ret)))
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
