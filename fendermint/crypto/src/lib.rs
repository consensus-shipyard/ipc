// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use rand::Rng;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

pub use libsecp256k1::{PublicKey, RecoveryId, Signature};

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
