// Copyright 2022-2023 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::chainid::ChainID;
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
        chain_id: &ChainID,
    ) -> Result<SignedMessage, SignedMessageError> {
        Self::verify_signature(&message, &signature, chain_id)?;
        Ok(SignedMessage { message, signature })
    }

    /// Create a signed message.
    #[cfg(feature = "secp256k1")]
    pub fn new_secp256k1(
        message: Message,
        sk: &libsecp256k1::SecretKey,
        chain_id: &ChainID,
    ) -> Result<Self, fvm_ipld_encoding::Error> {
        let data = Self::bytes_to_sign(&message, chain_id)?;
        let signature = Signature {
            sig_type: SignatureType::Secp256k1,
            bytes: sign_secp256k1(sk, &data).to_vec(),
        };
        Ok(Self { message, signature })
    }

    /// Calculate the CID of an FVM message.
    pub fn cid(message: &Message) -> Result<Cid, fvm_ipld_encoding::Error> {
        crate::cid(message)
    }

    /// Calculate the bytes that need to be signed.
    ///
    /// The [`ChainID`] is used as a replay attack protection, a variation of
    /// https://github.com/filecoin-project/FIPs/blob/master/FIPS/fip-0039.md
    pub fn bytes_to_sign(
        message: &Message,
        chain_id: &ChainID,
    ) -> Result<Vec<u8>, fvm_ipld_encoding::Error> {
        let mut data = Self::cid(message)?.to_bytes();
        data.extend(chain_id_bytes(chain_id).iter());
        Ok(data)
    }

    /// Verify that the message CID was signed by the `from` address.
    pub fn verify_signature(
        message: &Message,
        signature: &Signature,
        chain_id: &ChainID,
    ) -> Result<(), SignedMessageError> {
        let data = Self::bytes_to_sign(message, chain_id)?;

        signature
            .verify(&data, &message.from)
            .map_err(SignedMessageError::InvalidSignature)
    }

    /// Verifies that the from address of the message generated the signature.
    pub fn verify(&self, chain_id: &ChainID) -> Result<(), SignedMessageError> {
        Self::verify_signature(&self.message, &self.signature, chain_id)
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

#[cfg(feature = "secp256k1")]
fn sign_secp256k1(
    sk: &libsecp256k1::SecretKey,
    data: &[u8],
) -> [u8; fvm_shared::crypto::signature::SECP_SIG_LEN] {
    let hash: [u8; 32] = blake2b_simd::Params::new()
        .hash_length(32)
        .to_state()
        .update(data)
        .finalize()
        .as_bytes()
        .try_into()
        .unwrap();

    let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&hash), sk);

    let mut signature = [0u8; fvm_shared::crypto::signature::SECP_SIG_LEN];
    signature[..64].copy_from_slice(&sig.serialize());
    signature[64] = recovery_id.serialize();
    signature
}

/// Turn a [`ChainID`] into bytes. Uses big-endian encoding.
fn chain_id_bytes(chain_id: &ChainID) -> [u8; 8] {
    u64::from(*chain_id).to_be_bytes()
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

#[cfg(test)]
mod tests {
    use fvm_shared::{address::Address, chainid::ChainID};
    use quickcheck_macros::quickcheck;
    use rand::{rngs::StdRng, SeedableRng};

    use super::SignedMessage;

    #[quickcheck]
    fn chain_id_in_signature(msg: SignedMessage, chain_id: u64, seed: u64) -> Result<(), String> {
        let mut rng = StdRng::seed_from_u64(seed);
        let sk = libsecp256k1::SecretKey::random(&mut rng);
        let pk = libsecp256k1::PublicKey::from_secret_key(&sk);

        let chain_id0 = ChainID::from(chain_id);
        let chain_id1 = ChainID::from(chain_id.overflowing_add(1).0);

        let mut msg = msg.into_message();
        msg.from = Address::new_secp256k1(&pk.serialize())
            .map_err(|e| format!("failed to conver to address: {e}"))?;

        let signed = SignedMessage::new_secp256k1(msg, &sk, &chain_id0)
            .map_err(|e| format!("signing failed: {e}"))?;

        signed
            .verify(&chain_id0)
            .map_err(|e| format!("verifying failed: {e}"))?;

        if signed.verify(&chain_id1).is_ok() {
            return Err("verifying with a different chain ID should fail".into());
        }
        Ok(())
    }
}
