// Copyright 2022-2023 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::crypto::signature::{Signature, SignatureType};
use fvm_shared::message::Message;

/// Represents a wrapped message with signature bytes.
///
/// This is the message
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
    pub fn new_checked(message: Message, signature: Signature) -> anyhow::Result<SignedMessage> {
        Self::verify_signature(&message, &signature)?;
        Ok(SignedMessage { message, signature })
    }

    /// Verify that the message CID was signed by the `from` address.
    pub fn verify_signature(message: &Message, signature: &Signature) -> anyhow::Result<()> {
        let cid = crate::cid(&message)?.to_bytes();
        signature
            .verify(&cid, &message.from)
            .map_err(anyhow::Error::msg)
    }

    /// Verifies that the from address of the message generated the signature.
    pub fn verify(&self) -> anyhow::Result<()> {
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
