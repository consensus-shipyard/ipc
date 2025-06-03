// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use zeroize::Zeroize;

use crate::AddressDerivator;

pub mod adapter;

/// The struct that contains evm private key info
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EvmKeyInfo {
    pub(crate) private_key: Vec<u8>,
}

impl AddressDerivator<String> for EvmKeyInfo {
    fn as_address(&self) -> String {
        todo!("")
    }
}

impl EvmKeyInfo {
    pub fn new(private_key: Vec<u8>) -> Self {
        Self { private_key }
    }
}

impl core::fmt::Display for EvmKeyInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = hex::encode(self.private_key());
        f.write_str(&key)?;
        Ok(())
    }
}

impl EvmKeyInfo {
    pub fn private_key(&self) -> &[u8] {
        &self.private_key
    }
}

impl Drop for EvmKeyInfo {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct EvmPersistentKeyInfo {
    pub(crate) address: String,
    pub(crate) private_key: String,
}

impl EvmPersistentKeyInfo {
    pub fn new(addr: impl Into<String>, key_info: &EvmKeyInfo) -> Self {
        let sk = hex::encode(key_info.private_key());
        let address = addr.into();
        Self {
            private_key: sk,
            address,
        }
    }
    pub fn private_key(&self) -> &str {
        &self.private_key
    }
}

impl From<(&String, &EvmKeyInfo)> for EvmPersistentKeyInfo {
    fn from(value: (&String, &EvmKeyInfo)) -> Self {
        let sk = hex::encode(&value.1.private_key);
        let address = value.0.clone();
        EvmPersistentKeyInfo {
            private_key: sk,
            address,
        }
    }
}
impl From<(&String, &EvmPersistentKeyInfo)> for EvmKeyInfo {
    fn from(value: (&String, &EvmPersistentKeyInfo)) -> Self {
        let sk = hex::decode(&value.1.private_key).expect("TODO");
        EvmKeyInfo { private_key: sk }
    }
}

impl core::str::FromStr for EvmKeyInfo {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sk = hex::decode(s)?;
        Ok(Self { private_key: sk })
    }
}
