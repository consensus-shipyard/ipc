// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

//! Ethereum wallet key store.

mod memory;
mod persistent;

use anyhow::Result;
use std::hash::Hash;
use zeroize::Zeroize;

pub use crate::evm::persistent::{PersistentKeyInfo, PersistentKeyStore};

pub const DEFAULT_KEYSTORE_NAME: &str = "evm_keystore.json";

/// The key store trait for different evm key store
pub trait KeyStore {
    /// The type of the key that is stored
    type Key: Clone + Eq + Hash + TryFrom<KeyInfo>;

    /// Get the key info by address string
    fn get(&self, addr: &Self::Key) -> Result<Option<KeyInfo>>;
    /// List all addresses in the key store
    fn list(&self) -> Result<Vec<Self::Key>>;
    /// Put a new info to the addr
    fn put(&mut self, info: KeyInfo) -> Result<Self::Key>;
    /// Remove address from the key store
    fn remove(&mut self, addr: &Self::Key) -> Result<()>;
}

/// The struct that contains evm private key info
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct KeyInfo {
    private_key: Vec<u8>,
}

impl KeyInfo {
    pub fn new(private_key: Vec<u8>) -> Self {
        Self { private_key }
    }
}

impl KeyInfo {
    pub fn private_key(&self) -> &[u8] {
        &self.private_key
    }
}

impl Drop for KeyInfo {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

#[cfg(feature = "with-ethers")]
impl TryFrom<KeyInfo> for ethers::types::Address {
    type Error = anyhow::Error;

    fn try_from(value: KeyInfo) -> std::result::Result<Self, Self::Error> {
        use ethers::signers::Signer;
        let key = ethers::signers::Wallet::from_bytes(&value.private_key)?;
        Ok(key.address())
    }
}

#[cfg(feature = "with-ethers")]
pub fn random_key_info() -> KeyInfo {
    let key = ethers::core::k256::SecretKey::random(&mut rand::thread_rng());
    KeyInfo::new(key.to_bytes().to_vec())
}
