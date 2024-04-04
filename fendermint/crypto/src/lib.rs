// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use base64::engine::GeneralPurpose;
use base64::engine::{DecodePaddingMode, GeneralPurposeConfig};
use base64::{alphabet, Engine};
use ethers::{types::Address, utils::keccak256};
use libsecp256k1::{recover, Message};
use rand::Rng;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

pub use libsecp256k1::{PublicKey, RecoveryId, Signature};

/// A [`GeneralPurpose`] engine using the [`alphabet::STANDARD`] base64 alphabet
/// padding bytes when writing but requireing no padding when reading.
const B64_ENGINE: GeneralPurpose = GeneralPurpose::new(
    &alphabet::STANDARD,
    GeneralPurposeConfig::new()
        .with_encode_padding(true)
        .with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

/// Encode bytes in a format that the Genesis deserializer can handle.
pub fn to_b64(bz: &[u8]) -> String {
    B64_ENGINE.encode(bz)
}

/// Decode bytes from Base64
pub fn from_b64(b64: &str) -> anyhow::Result<Vec<u8>> {
    Ok(B64_ENGINE.decode(b64)?)
}

/// Create a new key and make sure the wrapped public key is normalized,
/// which is to ensure the results look the same after a serialization roundtrip.
pub fn normalize_public_key(pk: PublicKey) -> PublicKey {
    let mut aff: libsecp256k1::curve::Affine = pk.into();
    aff.x.normalize();
    aff.y.normalize();
    PublicKey::try_from(aff).unwrap()
}

/// Wrapper around a [libsecp256k1::SecretKey] that implements [Zeroize].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretKey(libsecp256k1::SecretKey);

impl SecretKey {
    pub fn sign(&self, bz: &[u8; 32]) -> (libsecp256k1::Signature, libsecp256k1::RecoveryId) {
        libsecp256k1::sign(&libsecp256k1::Message::parse(bz), &self.0)
    }

    pub fn random<R: Rng>(rng: &mut R) -> Self {
        Self(libsecp256k1::SecretKey::random(rng))
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_secret_key(&self.0)
    }

    pub fn serialize(&self) -> Zeroizing<[u8; libsecp256k1::util::SECRET_KEY_SIZE]> {
        Zeroizing::new(self.0.serialize())
    }
}

impl Zeroize for SecretKey {
    fn zeroize(&mut self) {
        let mut sk = libsecp256k1::SecretKey::default();
        std::mem::swap(&mut self.0, &mut sk);
        let mut sk: libsecp256k1::curve::Scalar = sk.into();
        sk.0.zeroize();
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.zeroize()
    }
}

impl ZeroizeOnDrop for SecretKey {}

impl TryFrom<Vec<u8>> for SecretKey {
    type Error = libsecp256k1::Error;

    fn try_from(mut value: Vec<u8>) -> Result<Self, Self::Error> {
        let sk = libsecp256k1::SecretKey::parse_slice(&value)?;
        value.zeroize();
        Ok(Self(sk))
    }
}

impl From<libsecp256k1::SecretKey> for SecretKey {
    fn from(value: libsecp256k1::SecretKey) -> Self {
        Self(value)
    }
}

impl From<&SecretKey> for PublicKey {
    fn from(value: &SecretKey) -> Self {
        value.public_key()
    }
}

/// Recover a sender, given message and the signature.
/// and returns the address of a public key.
///
/// The public address is defined as the low 20 bytes of the keccak hash of
/// the public key. Note that the public key returned from the `secp256k1`
/// crate is 65 bytes long, that is because it is prefixed by `0x04` to
/// indicate an uncompressed public key; this first byte is ignored when
/// computing the hash.
///
/// Based on https://github.com/tomusdrw/rust-web3/blob/master/src/signing.rs
pub fn recover_address(
    message: &[u8; 32],
    signature: &[u8],
    recovery_id: u8,
) -> Result<Address, libsecp256k1::Error> {
    let message = Message::parse(message);
    let signature = Signature::parse_standard_slice(signature)?;
    let recovery_id = RecoveryId::parse(recovery_id)?;

    let public_key = recover(&message, &signature, &recovery_id)?;
    let public_key_bytes = public_key.serialize();
    debug_assert_eq!(public_key_bytes[0], 0x04);

    let hash = keccak256(&public_key_bytes[1..]);

    Ok(Address::from_slice(&hash[12..]))
}
