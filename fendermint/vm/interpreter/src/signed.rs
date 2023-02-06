// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::anyhow;
use async_trait::async_trait;

use fendermint_vm_message::signed::{SignedMessage, SignedMessageError};

use crate::{
    fvm::{FvmApplyRet, FvmMessage},
    Interpreter,
};

/// Interpreter working on signed messages, validating their signature before sending
/// the unsigned parts on for execution.
#[derive(Clone)]
pub struct SignedMessageInterpreter<I> {
    inner: I,
}

impl<I> SignedMessageInterpreter<I> {
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

pub enum SignedMesssageApplyRet {
    InvalidSignature(String),
    Applied(FvmApplyRet),
}

#[async_trait]
impl<I> Interpreter for SignedMessageInterpreter<I>
where
    I: Interpreter<Message = FvmMessage, DeliverOutput = FvmApplyRet>,
{
    type State = I::State;
    type Message = SignedMessage;
    type BeginOutput = I::BeginOutput;
    type DeliverOutput = SignedMesssageApplyRet;
    type EndOutput = I::EndOutput;

    async fn deliver(
        &self,
        state: Self::State,
        msg: Self::Message,
    ) -> anyhow::Result<(Self::State, Self::DeliverOutput)> {
        match msg.verify() {
            Err(SignedMessageError::Ipld(e)) => Err(anyhow!(e)),
            Err(SignedMessageError::InvalidSignature(s)) => {
                // TODO: We can penalize the validator for including an invalid signature.
                Ok((state, SignedMesssageApplyRet::InvalidSignature(s)))
            }
            Ok(()) => {
                let (state, ret) = self.inner.deliver(state, msg.message).await?;

                Ok((state, SignedMesssageApplyRet::Applied(ret)))
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
