// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::anyhow;
use async_trait::async_trait;

use fendermint_vm_message::signed::{SignedMessage, SignedMessageError};

use crate::{
    fvm::{FvmApplyRet, FvmCheckRet, FvmMessage},
    CheckInterpreter, Interpreter,
};

/// Message validation failed due to an invalid signature.
pub struct InvalidSignature(pub String);

pub type SignedMessageApplyRet = Result<FvmApplyRet, InvalidSignature>;
pub type SignedMessageCheckRet = Result<FvmCheckRet, InvalidSignature>;

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

#[async_trait]
impl<I> Interpreter for SignedMessageInterpreter<I>
where
    I: Interpreter<Message = FvmMessage, DeliverOutput = FvmApplyRet>,
{
    type State = I::State;
    type Message = SignedMessage;
    type BeginOutput = I::BeginOutput;
    type DeliverOutput = SignedMessageApplyRet;
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
                Ok((state, Err(InvalidSignature(s))))
            }
            Ok(()) => {
                let (state, ret) = self.inner.deliver(state, msg.message).await?;
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

#[async_trait]
impl<I> CheckInterpreter for SignedMessageInterpreter<I>
where
    I: CheckInterpreter<Message = FvmMessage, Output = FvmCheckRet>,
{
    type State = I::State;
    type Message = SignedMessage;
    type Output = SignedMessageCheckRet;

    async fn check(
        &self,
        state: Self::State,
        msg: Self::Message,
        is_recheck: bool,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        let verify_result = if is_recheck { Ok(()) } else { msg.verify() };

        match verify_result {
            Err(SignedMessageError::Ipld(e)) => Err(anyhow!(e)),
            Err(SignedMessageError::InvalidSignature(s)) => {
                // There is nobody we can punish for this, we can just tell Tendermint to discard this message,
                // and potentially block the source IP address.
                Ok((state, Err(InvalidSignature(s))))
            }
            Ok(()) => {
                let (state, ret) = self.inner.check(state, msg.message, is_recheck).await?;
                Ok((state, Ok(ret)))
            }
        }
    }
}
