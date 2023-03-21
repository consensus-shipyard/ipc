// Copyright 2022-2023 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::crypto::signature::{Signature, SignatureType};
use fvm_shared::message::Message;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignedMessageError {
    #[error("message cannot be serialized")]
    Ipld(#[from] fvm_ipld_encoding::Error),
    #[error("invalid signature: {0}")]
    InvalidSignature(String),
}

/// Represents a wrapped message with signature bytes.
///
/// This is the message that the client needs to send, but only the `message`
/// part is signed over.
///
/// Tuple serialization is used because it might result in a more compact data structure for storage,
/// and because the `Message` is already serialized as a tuple.
#[derive(PartialEq, Clone, Debug, Serialize_tuple, Deserialize_tuple, Hash, Eq)]
pub struct SignedMessage {
    pub message: Message,
    pub signature: Signature,
}

impl SignedMessage {
    /// Generate a new signed message from fields.
    ///
    /// The signature will not be verified.
    pub fn new_unchecked(message: Message, signature: Signature) -> SignedMessage {
        SignedMessage { message, signature }
    }

    /// Generate a new signed message from fields.
    ///
    /// The signature will be verified.
    pub fn new_checked(
        message: Message,
        signature: Signature,
    ) -> Result<SignedMessage, SignedMessageError> {
        Self::verify_signature(&message, &signature)?;
        Ok(SignedMessage { message, signature })
    }

    /// Calculate the CID of an FVM message.
    pub fn cid(message: &Message) -> Result<Cid, fvm_ipld_encoding::Error> {
        crate::cid(message)
    }

    /// Verify that the message CID was signed by the `from` address.
    pub fn verify_signature(
        message: &Message,
        signature: &Signature,
    ) -> Result<(), SignedMessageError> {
        let cid = Self::cid(message)?.to_bytes();
        signature
            .verify(&cid, &message.from)
            .map_err(SignedMessageError::InvalidSignature)
    }

    /// Verifies that the from address of the message generated the signature.
    pub fn verify(&self) -> Result<(), SignedMessageError> {
        Self::verify_signature(&self.message, &self.signature)
    }

    /// Returns reference to the unsigned message.
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Returns signature of the signed message.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// Consumes self and returns it's unsigned message.
    pub fn into_message(self) -> Message {
        self.message
    }

    /// Checks if the signed message is a BLS message.
    pub fn is_bls(&self) -> bool {
        self.signature.signature_type() == SignatureType::BLS
    }

    /// Checks if the signed message is a SECP message.
    pub fn is_secp256k1(&self) -> bool {
        self.signature.signature_type() == SignatureType::Secp256k1
    }
}

/// Signed message with an invalid random signature.
#[cfg(feature = "arb")]
mod arb {
    use fendermint_testing::arb::{ArbAddress, ArbTokenAmount};
    use fvm_shared::{crypto::signature::Signature, message::Message};

    use super::SignedMessage;

    /// An arbitrary `SignedMessage` that is at least as consistent as required for serialization.
    impl quickcheck::Arbitrary for SignedMessage {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut message = Message::arbitrary(g);
            message.gas_fee_cap = ArbTokenAmount::arbitrary(g).0;
            message.gas_premium = ArbTokenAmount::arbitrary(g).0;
            message.value = ArbTokenAmount::arbitrary(g).0;
            message.to = ArbAddress::arbitrary(g).0;
            message.from = ArbAddress::arbitrary(g).0;

            Self {
                message,
                signature: Signature::arbitrary(g),
            }
        }
    }
}
