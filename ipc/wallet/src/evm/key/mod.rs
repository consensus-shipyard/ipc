use std::str::FromStr;

use fvm_shared::address::Address;
use ipc_types::EthAddress;
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use zeroize::Zeroize;

#[cfg(feature = "ethers")]
use crate::evm::adapter::EthKeyAddress;
use crate::{AddressDerivator, DefaultKey};

pub mod adapter;

#[cfg(test)]
#[cfg(feature = "ethers")]
mod tests;

#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct WrappedEthAddress(String);

impl FromStr for WrappedEthAddress {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _bytes = hex::decode(s)?;
        Ok(Self(s.to_string()))
    }
}

impl WrappedEthAddress {
    pub fn to_eth_address(&self) -> EthAddress {
        <EthAddress as FromStr>::from_str(self.0.as_str()).unwrap()
    }
    pub fn to_fvm_address(&self) -> Address {
        Address::from_str(self.0.as_str()).unwrap()
    }
    #[cfg(feature = "ethers")]
    pub fn to_ethers(&self) -> ethers::types::Address {
        ethers::types::Address::from_str(&self.0).unwrap()
    }
    #[cfg(feature = "ethers")]
    pub fn from_ethers(value: &ethers::types::Address) -> Self {
        let s = hex::encode(value.as_bytes());
        Self(s)
    }

    #[cfg(feature = "ethers")]
    pub fn from_adapter(value: &EthKeyAddress) -> Self {
        let value = value.to_string();
        Self(dbg!(value))
    }
}
impl DefaultKey for WrappedEthAddress {
    fn default_key() -> Self {
        Self("default-key".to_owned())
    }
}
impl AddressDerivator<WrappedEthAddress> for EvmKeyInfo {
    fn as_address(&self) -> WrappedEthAddress {
        // TODO deal with BLS signatures
        let pubkey = secret_key_to_pub_secp256k1(self.private_key())
            .expect("Input is pre-checked at construction");
        let addr = public_key_to_address(&pubkey)
            .expect("Public key to address never fails with valid key");
        WrappedEthAddress(hex::encode(&addr[..]))
    }
}

impl From<(&WrappedEthAddress, &EvmKeyInfo)> for EvmPersistentKeyInfo {
    fn from(value: (&WrappedEthAddress, &EvmKeyInfo)) -> Self {
        let sk = hex::encode(&value.1.private_key);
        let address = value.0.clone();
        EvmPersistentKeyInfo {
            private_key: sk,
            address: address.to_string(),
        }
    }
}
impl From<(&WrappedEthAddress, &EvmPersistentKeyInfo)> for EvmKeyInfo {
    fn from(value: (&WrappedEthAddress, &EvmPersistentKeyInfo)) -> Self {
        let sk = hex::decode(&value.1.private_key)
            .expect("Key in persisent info is always valid hex encoded secret key");
        EvmKeyInfo { private_key: sk }
    }
}

impl std::fmt::Display for WrappedEthAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// The struct that contains evm private key info
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EvmKeyInfo {
    pub(crate) private_key: Vec<u8>,
}

fn secret_key_to_pub_secp256k1(sk: &[u8]) -> anyhow::Result<Vec<u8>> {
    let sk = libsecp256k1::SecretKey::parse_slice(sk)?;
    let pk = libsecp256k1::PublicKey::from_secret_key(&sk);
    Ok(pk.serialize().to_vec())
}
#[allow(dead_code)]
fn secret_key_to_public_bls(sk: &[u8]) -> anyhow::Result<Vec<u8>> {
    use bls_signatures::Serialize;
    let sk = bls_signatures::PrivateKey::from_bytes(sk)?;
    let pk = sk.public_key();
    Ok(pk.as_bytes())
}

pub fn public_key_to_address(pubkey: &[u8]) -> anyhow::Result<[u8; 20]> {
    use fvm_shared::address::SECP_PUB_LEN;
    use tiny_keccak::{Hasher, Keccak};

    if pubkey.len() != SECP_PUB_LEN {
        anyhow::bail!(
            "Unexpected length: {} should be {}",
            pubkey.len(),
            SECP_PUB_LEN
        );
    }

    fn keccak256(bytes: &[u8]) -> [u8; 32] {
        let mut output = [0u8; 32];

        let mut hasher = Keccak::v256();
        hasher.update(bytes.as_ref());
        hasher.finalize(&mut output);
        output
    }

    let mut hash20 = [0u8; 20];
    // Based on [ethers_core::utils::secret_key_to_address]
    let hash32 = keccak256(&pubkey[1..]);
    hash20.copy_from_slice(&hash32[12..]);
    Ok(hash20)
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

impl core::str::FromStr for EvmKeyInfo {
    type Err = hex::FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sk = hex::decode(s)?;
        Ok(Self { private_key: sk })
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
