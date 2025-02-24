use thiserror::Error;

use crate::fvm::FvmMessage;
use fendermint_vm_message::chain::ChainMessage;
use fendermint_vm_message::{
    ipc::{BottomUpCheckpoint, CertifiedMessage, IpcMessage, SignedRelayedMessage},
    signed::SignedMessageError,
};
use fvm_shared::chainid::ChainID;

use fendermint_vm_message::signed::SignedMessage;

#[derive(Debug, Error)]
#[error("illegal message: {0}")]
pub struct IllegalMessage(String);

pub enum VerifiableMessage {
    Signed(SignedMessage),
    BottomUp(SignedRelayedMessage<CertifiedMessage<BottomUpCheckpoint>>),
}

impl VerifiableMessage {
    pub fn message(&self) -> FvmMessage {
        match self {
            VerifiableMessage::Signed(inner) => inner.message().to_owned(),
            VerifiableMessage::BottomUp(inner) => inner.message(),
        }
    }

    pub fn verify(&self, chain_id: &ChainID) -> Result<(), SignedMessageError> {
        match self {
            VerifiableMessage::Signed(inner) => inner.verify(chain_id),
            VerifiableMessage::BottomUp(inner) => inner.verify(chain_id),
        }
    }
}

impl TryFrom<ChainMessage> for VerifiableMessage {
    type Error = IllegalMessage;

    fn try_from(msg: ChainMessage) -> Result<Self, Self::Error> {
        match msg {
            ChainMessage::Signed(inner) => Ok(VerifiableMessage::Signed(inner)),
            ChainMessage::Ipc(inner) => match inner {
                IpcMessage::BottomUpResolve(inner) => Ok(VerifiableMessage::BottomUp(inner)),
                other => Err(IllegalMessage(format!("{:?}", other))),
            },
        }
    }
}
