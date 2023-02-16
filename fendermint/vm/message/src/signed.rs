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
    use fvm_shared::{
        address::Address,
        bigint::{BigInt, Integer, Sign, MAX_BIGINT_SIZE},
        crypto::signature::Signature,
        econ::TokenAmount,
        message::Message,
    };

    use super::SignedMessage;

    /// Unfortunately an arbitrary `TokenAmount` is not serializable if it has more than 128 bytes, we get "BigInt too large" error.
    ///
    /// The max below is taken from https://github.com/filecoin-project/ref-fvm/blob/fvm%40v3.0.0-alpha.24/shared/src/bigint/bigint_ser.rs#L80-L81
    fn fix_tokens(tokens: TokenAmount) -> TokenAmount {
        let max_bigint = BigInt::new(Sign::Plus, vec![u32::MAX; MAX_BIGINT_SIZE / 4 - 1]);
        let atto = tokens.atto();
        let atto = atto.mod_floor(&max_bigint);
        TokenAmount::from_atto(atto)
    }

    /// Unfortunately an arbitrary `DelegatedAddress` can be inconsistent with bytes that do not correspond to its length.
    fn fix_address(addr: Address) -> Address {
        let bz = addr.to_bytes();
        Address::from_bytes(&bz).unwrap()
    }

    /// An arbitrary `SignedMessage` that is at least as consistent as required for serialization.
    impl quickcheck::Arbitrary for SignedMessage {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut message = Message::arbitrary(g);
            message.gas_fee_cap = fix_tokens(message.gas_fee_cap);
            message.gas_premium = fix_tokens(message.gas_premium);
            message.value = fix_tokens(message.value);
            message.to = fix_address(message.to);
            message.from = fix_address(message.from);

            Self {
                message,
                signature: Signature::arbitrary(g),
            }
        }
    }
}
