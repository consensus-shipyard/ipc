// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use base64::{prelude::BASE64_STANDARD, Engine};
use fvm_shared::crypto::signature::SignatureType;

use crate::{new_address, to_public, AddressDerivator};

// TODO need to update keyinfo to not use SignatureType, use string instead to
// save keys like jwt secret
/// `KeyInfo` structure, this contains the type of key (stored as a string) and
/// the private key. Note how the private key is stored as a byte vector
#[derive(Clone, PartialEq, Debug, Eq, serde::Serialize, serde::Deserialize)]
pub struct FvmKeyInfo {
    pub(crate) key_type: SignatureType,
    // Vec<u8> is used because The private keys for BLS and SECP256K1 are not of the same type
    pub(crate) private_key: Vec<u8>,
}

impl AddressDerivator<String> for FvmKeyInfo {
    /// Address as used in the wallet
    // XXX the `wallet-` is absurd, remove
    fn as_address(&self) -> String {
        let key_type = *self.key_type();
        let pub_key = to_public(key_type, &self.private_key).unwrap();
        let address = new_address(key_type, pub_key.as_slice()).unwrap();
        format!("wallet-{address}")
    }
}

impl FvmKeyInfo {
    /// Return a new `KeyInfo` given the key type and private key
    pub fn new(key_type: SignatureType, private_key: Vec<u8>) -> Self {
        FvmKeyInfo {
            key_type,
            private_key,
        }
    }

    /// Return a reference to the key's signature type
    pub fn key_type(&self) -> &SignatureType {
        &self.key_type
    }

    /// Return a reference to the private key
    pub fn private_key(&self) -> &[u8] {
        &self.private_key[..]
    }
}

#[derive(Clone, PartialEq, Debug, Eq, serde::Serialize, serde::Deserialize)]
pub struct PersistentKeyInfo {
    pub(crate) key_type: SignatureType,
    pub(crate) private_key: String,
}

// TODO make this a `TryFrom`, we cannot sanity for on-disk stuff
impl From<(&String, &PersistentKeyInfo)> for FvmKeyInfo {
    fn from(value: (&String, &PersistentKeyInfo)) -> Self {
        Self {
            key_type: value.1.key_type,
            private_key: BASE64_STANDARD
                .decode(value.1.private_key.clone())
                .expect("Sane key, insane assumption, but hey"),
        }
    }
}

impl From<(&String, &FvmKeyInfo)> for PersistentKeyInfo {
    fn from(value: (&String, &FvmKeyInfo)) -> Self {
        Self {
            key_type: value.1.key_type,
            private_key: BASE64_STANDARD.encode(value.1.private_key()),
        }
    }
}
