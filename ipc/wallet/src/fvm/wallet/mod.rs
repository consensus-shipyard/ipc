// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{convert::TryFrom, str::FromStr};

use ahash::{HashMap, HashMapExt};
use fvm_shared::{
    address::Address,
    crypto::signature::{Signature, SignatureType},
};
use serde::{Deserialize, Serialize};

use crate::errors::*;

use super::{FvmCrownJewels, FvmKeyInfo};

pub mod helpers;

#[cfg(test)]
mod tests;

/// A key, this contains a `KeyInfo`, an address, and a public key.
#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct FullKey {
    pub key_info: FvmKeyInfo,
    // Vec<u8> is used because The public keys for BLS and SECP256K1 are not of the same type
    // FIXME use proper type representation, if it's only those two keytypes
    pub public_key: Vec<u8>,
    pub address: Address,
}

impl TryFrom<FvmKeyInfo> for FullKey {
    type Error = crate::errors::WalletErr;

    fn try_from(key_info: FvmKeyInfo) -> Result<Self, Self::Error> {
        let public_key = self::helpers::to_public(*key_info.key_type(), key_info.private_key())?;
        let address = self::helpers::new_address(*key_info.key_type(), &public_key)?;
        Ok(FullKey {
            key_info,
            public_key,
            address,
        })
    }
}

// This is a Wallet, it contains 2 HashMaps:
// - keys which is a HashMap of Keys resolved by their Address
// - keystore which is a HashMap of KeyInfos resolved by their Address
/// A wallet is a collection of private keys with optional persistence and
/// optional encryption.
#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Wallet /*<S=FvmCrownJewels>*/ {
    in_memory_cache: HashMap<Address, FullKey>,
    // double check, the key is string due to some hackory with test vs life key names
    keystore: FvmCrownJewels,
}

impl Wallet /*<S>*/ {
    /// Return a new wallet with a given `KeyStore`
    pub fn new(keystore: FvmCrownJewels) -> Self {
        Wallet {
            in_memory_cache: HashMap::new(),
            keystore,
        }
    }

    /// Return a wallet from a given amount of keys.
    pub fn new_from_keys(
        keystore: FvmCrownJewels,
        key_vec: impl IntoIterator<Item = FullKey>,
    ) -> Self {
        let mut keys: HashMap<Address, FullKey> = HashMap::new();
        for item in key_vec.into_iter() {
            keys.insert(item.address, item);
        }
        Wallet {
            in_memory_cache: keys,
            keystore,
        }
    }

    // If this key does not exist in the keys hashmap, check if this key is in
    // the keystore, if it is, then add it to keys, otherwise return Error
    /// Return the key that is resolved by a given address,
    pub fn find_key(&mut self, addr: &Address) -> Result<FullKey, WalletErr> {
        if let Some(k) = self.in_memory_cache.get(addr) {
            return Ok(k.clone());
        }
        let key_string = format!("wallet-{addr}");
        let key_info = match self.keystore.get(&key_string) {
            Ok(k) => k,
            Err(_) => {
                // replace with testnet prefix
                self.keystore
                    .get(&format!("wallet-t{}", &addr.to_string()[1..]))?
            }
        };
        let new_key = FullKey::try_from(key_info)?;
        self.in_memory_cache.insert(*addr, new_key.clone());
        Ok(new_key)
    }

    /// Return the resultant `Signature` after signing a given message
    pub fn sign(&mut self, addr: &Address, msg: &[u8]) -> Result<Signature, WalletErr> {
        // this will return an error if the key cannot be found in either the keys
        // hashmap or it is not found in the keystore
        let key = self.find_key(addr)?;
        self::helpers::sign(*key.key_info.key_type(), key.key_info.private_key(), msg)
    }

    /// Return the `KeyInfo` for a given address
    pub fn export(&mut self, addr: &Address) -> Result<FvmKeyInfo, WalletErr> {
        let k = self.find_key(addr)?;
        Ok(k.key_info)
    }

    /// Add `KeyInfo` to the wallet, return the address that resolves to this
    /// newly added `KeyInfo`
    pub fn import(&mut self, key_info: FvmKeyInfo) -> Result<Address, WalletErr> {
        let k = FullKey::try_from(key_info)?;
        let addr = format!("wallet-{}", k.address);
        self.keystore.put(addr, k.key_info)?;
        Ok(k.address)
    }

    /// Return a vector that contains all of the addresses in the wallet's
    /// `KeyStore`
    pub fn list_addrs(&self) -> Result<Vec<Address>, WalletErr> {
        list_addrs(&self.keystore)
    }

    pub fn remove(&mut self, addr: &Address) -> Result<(), WalletErr> {
        let _ = self.in_memory_cache.remove(addr);
        self.keystore.remove(addr.to_string())?;
        Ok(())
    }

    pub fn get_default_info(&self) -> Result<FvmKeyInfo, WalletErr> {
        let key_info = self.keystore.get(&String::from("default"))?;
        Ok(key_info)
    }

    /// Return the address of the default `KeyInfo` in the wallet
    pub fn get_default(&self) -> Result<Address, WalletErr> {
        let key_info = self.get_default_info()?;
        let k = FullKey::try_from(key_info)?;
        Ok(k.address)
    }

    /// Set a default `KeyInfo` to the wallet
    pub fn set_default(&mut self, addr: Address) -> Result<(), WalletErr> {
        let addr_string = format!("wallet-{addr}");
        let key_info = self.keystore.get(&addr_string)?;
        let default_key_info = self.get_default_info();
        if default_key_info.is_ok() {
            self.keystore.remove("default".to_string())?; // This line should
                                                          // unregister current
                                                          // default key then
                                                          // continue
        }
        self.keystore.put("default".to_string(), key_info)?;
        Ok(())
    }

    /// Generate a new address that fits the requirement of the given
    /// `SignatureType`
    ///
    /// If no default key is present, makes the generated key the default one!
    pub fn generate_addr(&mut self, typ: SignatureType) -> Result<Address, WalletErr> {
        let key = generate_key(typ)?;
        let addr = format!("wallet-{}", key.address);
        self.keystore.put(addr, key.key_info.clone())?;
        self.in_memory_cache.insert(key.address, key.clone());
        let value = self.get_default_info();
        if value.is_err() {
            self.keystore
                .put("default".to_string(), key.key_info.clone())
                .map_err(|err| WalletErr::Other(err.to_string()))?;
        }

        Ok(key.address)
    }

    /// Return whether or not the Wallet contains a key that is resolved by the
    /// supplied address
    pub fn has_key(&mut self, addr: &Address) -> bool {
        self.find_key(addr).is_ok()
    }
}

/// Return the default address for `KeyStore`
pub fn get_default(keystore: &FvmCrownJewels) -> Result<Option<Address>, WalletErr> {
    if let Ok(key_info) = keystore.get(&String::from("default")) {
        let k = FullKey::try_from(key_info)?;
        Ok(Some(k.address))
    } else {
        Ok(None)
    }
}

/// Return vector of addresses sorted by their string representation in
/// `KeyStore`
pub fn list_addrs(keystore: &FvmCrownJewels) -> Result<Vec<Address>, WalletErr> {
    let mut all = Vec::from_iter(keystore.list());
    all.sort();
    let mut out = Vec::new();
    for i in all {
        if let Some(addr_str) = i.strip_prefix("wallet-") {
            if let Ok(addr) = Address::from_str(addr_str) {
                out.push(addr);
            }
        }
    }
    Ok(out)
}

/// Returns a key corresponding to given address
pub fn find_key(addr: &Address, keystore: &FvmCrownJewels) -> Result<FullKey, WalletErr> {
    let key_string = format!("wallet-{addr}");
    let key_info = keystore.get(&key_string)?;
    let new_key = FullKey::try_from(key_info)?;
    Ok(new_key)
}

pub fn try_find(addr: &Address, keystore: &FvmCrownJewels) -> Result<FvmKeyInfo, WalletErr> {
    let key_string = format!("wallet-{addr}");
    match keystore.get(&key_string) {
        Ok(k) => Ok(k),
        Err(_) => {
            let mut new_addr = addr.to_string();

            // Try to replace prefix with testnet, for backwards compatibility
            // * We might be able to remove this, look into variants
            new_addr.replace_range(0..1, "t");
            let key_string = format!("wallet-{new_addr}");
            let key_info = match keystore.get(&key_string) {
                Ok(k) => k,
                Err(_) => keystore.get(&format!("wallet-f{}", &new_addr[1..]))?,
            };
            Ok(key_info)
        }
    }
}

/// Return `KeyInfo` for given address in `KeyStore`
pub fn export_key_info(addr: &Address, keystore: &FvmCrownJewels) -> Result<FvmKeyInfo, WalletErr> {
    let key = find_key(addr, keystore)?;
    Ok(key.key_info)
}

/// Generate new key of given `SignatureType`
pub fn generate_key(typ: SignatureType) -> Result<FullKey, WalletErr> {
    let private_key = self::helpers::generate(typ)?;
    let key_info = FvmKeyInfo::new(typ, private_key);
    FullKey::try_from(key_info)
}

/// Import `KeyInfo` into `KeyStore`
pub fn import(key_info: FvmKeyInfo, keystore: &mut FvmCrownJewels) -> Result<Address, WalletErr> {
    let k = FullKey::try_from(key_info)?;
    let addr = format!("wallet-{}", k.address);
    keystore.put(addr, k.key_info)?;
    Ok(k.address)
}
