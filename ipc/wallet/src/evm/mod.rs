// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Ethereum wallet key store.

mod key;

pub const ENCRYPTED_KEYSTORE_NAME: &str = "evm_keystore";
pub const KEYSTORE_NAME: &str = "evm_keystore.json";

pub use key::*;

pub type EvmCrownJewels = crate::CrownJewels<WrappedEthAddress, EvmKeyInfo, EvmPersistentKeyInfo>;

#[cfg(test)]
mod tests;

// The key store trait for different evm key store
// XXX What's this used/useful for?
// pub trait KeyStoreTrait {
//     /// The type of the key that is stored
//     type Key: Clone + Eq + Hash + ToString + TryFrom<EvmKeyInfo>;

//     type InMem: Clone + PartialEq + Eq + for<'a,'b> From<(&'a Key, &'b Persist)> where &InMem: Into<Self::Key>;
//     type Persist: serde::Deserialize + serde::Serialize + for<'a,'b> From<(&'a Key, &'b InMem)>;

//     /// Get the key info by address string
//     fn get(&self, addr: &Self::Key) -> Result<Option<Self::InMem>>;
//     /// List all addresses in the key store
//     fn list(&self) -> Result<Vec<Self::Key>>;
//     /// Put a new info to the addr
//     fn put(&mut self, info: Self::InMem) -> Result<Self::Key>;
//     /// Remove address from the key store
//     fn remove(&mut self, addr: &Self::Key) -> Result<()>;
//     /// Set default wallet
//     fn set_default(&mut self, addr: &Self::Key) -> Result<()>;
//     /// Get default wallet
//     fn get_default(&mut self) -> Result<Option<Self::Key>>;
// }
