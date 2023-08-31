// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::anyhow;
use async_trait::async_trait;

use fendermint_vm_core::chainid::HasChainID;
use fendermint_vm_message::{
    query::FvmQuery,
    signed::{chain_id_bytes, SignedMessage, SignedMessageError},
};
use fvm_ipld_encoding::Error as IpldError;
use fvm_shared::{chainid::ChainID, crypto::signature::Signature};
use serde::Serialize;

use crate::{
    fvm::{FvmApplyRet, FvmCheckRet, FvmMessage, FvmQueryRet},
    CheckInterpreter, ExecInterpreter, GenesisInterpreter, QueryInterpreter,
};

/// Message validation failed due to an invalid signature.
pub struct InvalidSignature(pub String);

pub struct SignedMessageApplyRet {
    pub fvm: FvmApplyRet,
    pub sig_hash: [u8; 32],
}

pub type SignedMessageApplyRes = Result<SignedMessageApplyRet, InvalidSignature>;
pub type SignedMessageCheckRes = Result<FvmCheckRet, InvalidSignature>;

/// Different kinds of signed messages.
///
/// This technical construct was introduced so we can have a simple linear interpreter stack
/// where everything flows through all layers, which means to pass something to the FVM we
/// have to go through the signature check.
pub enum VerifiableMessage {
    /// A normal message sent by a user.
    Signed(SignedMessage),
    /// Something we constructed to pass on to the FVM.
    Synthetic(SyntheticMessage),
}

impl VerifiableMessage {
    pub fn verify(&self, chain_id: &ChainID) -> Result<(), SignedMessageError> {
        match self {
            Self::Signed(m) => m.verify(chain_id),
            Self::Synthetic(m) => m.verify(chain_id),
        }
    }

    pub fn into_message(self) -> FvmMessage {
        match self {
            Self::Signed(m) => m.into_message(),
            Self::Synthetic(m) => m.message,
        }
    }
}

pub struct SyntheticMessage {
    /// The artifical message.
    message: FvmMessage,
    /// The CID of the original message (assuming here that that's what was signed).
    orig_cid: cid::Cid,
    /// The signature over the original CID.
    signature: Signature,
}

impl SyntheticMessage {
    pub fn new<T: Serialize>(
        message: FvmMessage,
        orig: &T,
        signature: Signature,
    ) -> Result<Self, IpldError> {
        let orig_cid = fendermint_vm_message::cid(orig)?;
        Ok(Self {
            message,
            orig_cid,
            signature,
        })
    }

    pub fn verify(&self, chain_id: &ChainID) -> Result<(), SignedMessageError> {
        let mut data = self.orig_cid.to_bytes();
        data.extend(chain_id_bytes(chain_id).iter());

        self.signature
            .verify(&data, &self.message.from)
            .map_err(SignedMessageError::InvalidSignature)
    }
}

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
impl<I> ExecInterpreter for SignedMessageInterpreter<I>
where
    I: ExecInterpreter<Message = FvmMessage, DeliverOutput = FvmApplyRet>,
    I::State: HasChainID,
{
    type State = I::State;
    type Message = VerifiableMessage;
    type BeginOutput = I::BeginOutput;
    type DeliverOutput = SignedMessageApplyRes;
    type EndOutput = I::EndOutput;

    async fn deliver(
        &self,
        state: Self::State,
        msg: Self::Message,
    ) -> anyhow::Result<(Self::State, Self::DeliverOutput)> {
        // Doing these first, so the compiler doesn't need `Send` bound, which it would if the
        // async call to `inner.deliver` would be inside a match holding a reference to `state`.
        let chain_id = state.chain_id();

        match msg.verify(&chain_id) {
            Err(SignedMessageError::Ipld(e)) => Err(anyhow!(e)),
            Err(SignedMessageError::Ethereum(e)) => {
                Ok((state, Err(InvalidSignature(e.to_string()))))
            }
            Err(SignedMessageError::InvalidSignature(s)) => {
                // TODO: We can penalize the validator for including an invalid signature.
                Ok((state, Err(InvalidSignature(s))))
            }
            Ok(()) => {
                let msg = msg.into_message();
                let sig_hash = SignedMessage::sig_hash(&msg, &chain_id)?;
                let (state, ret) = self.inner.deliver(state, msg).await?;
                let ret = SignedMessageApplyRet { fvm: ret, sig_hash };
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
    I::State: HasChainID + Send + 'static,
{
    type State = I::State;
    type Message = VerifiableMessage;
    type Output = SignedMessageCheckRes;

    async fn check(
        &self,
        state: Self::State,
        msg: Self::Message,
        is_recheck: bool,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        let verify_result = if is_recheck {
            Ok(())
        } else {
            msg.verify(&state.chain_id())
        };

        match verify_result {
            Err(SignedMessageError::Ipld(e)) => Err(anyhow!(e)),
            Err(SignedMessageError::Ethereum(e)) => {
                Ok((state, Err(InvalidSignature(e.to_string()))))
            }
            Err(SignedMessageError::InvalidSignature(s)) => {
                // There is nobody we can punish for this, we can just tell Tendermint to discard this message,
                // and potentially block the source IP address.
                Ok((state, Err(InvalidSignature(s))))
            }
            Ok(()) => {
                let (state, ret) = self
                    .inner
                    .check(state, msg.into_message(), is_recheck)
                    .await?;
                Ok((state, Ok(ret)))
            }
        }
    }
}

#[async_trait]
impl<I> QueryInterpreter for SignedMessageInterpreter<I>
where
    I: QueryInterpreter<Query = FvmQuery, Output = FvmQueryRet>,
    I::State: HasChainID,
{
    type State = I::State;
    type Query = I::Query;
    type Output = FvmQueryRet<SignedMessageApplyRet>;

    async fn query(
        &self,
        state: Self::State,
        qry: Self::Query,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        let sig_hash = match qry {
            FvmQuery::Call(ref msg) => SignedMessage::sig_hash(msg, &state.chain_id()).ok(),
            _ => None,
        };

        let (state, ret) = self.inner.query(state, qry).await?;

        let ret = match ret {
            FvmQueryRet::Ipld(x) => FvmQueryRet::Ipld(x),
            FvmQueryRet::ActorState(x) => FvmQueryRet::ActorState(x),
            FvmQueryRet::EstimateGas(x) => FvmQueryRet::EstimateGas(x),
            FvmQueryRet::StateParams(x) => FvmQueryRet::StateParams(x),
            FvmQueryRet::Call(x) => FvmQueryRet::Call(SignedMessageApplyRet {
                fvm: x,
                sig_hash: sig_hash.unwrap_or_default(),
            }),
        };

        Ok((state, ret))
    }
}

#[async_trait]
impl<I> GenesisInterpreter for SignedMessageInterpreter<I>
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
