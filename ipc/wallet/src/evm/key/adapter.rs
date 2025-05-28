#![cfg(feature = "with-ethers")]
use super::*;

use std::str::FromStr;

impl TryFrom<&EvmKeyInfo> for ethers::types::Address {
    type Error = anyhow::Error;

    fn try_from(value: &EvmKeyInfo) -> std::result::Result<Self, Self::Error> {
        use ethers::signers::Signer;
        let key = ethers::signers::Wallet::from_bytes(&value.private_key)?;
        Ok(key.address())
    }
}

impl FromStr for EthKeyAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = ethers::types::Address::from_str(s)?;
        Ok(EthKeyAddress { inner })
    }
}

pub fn random_eth_key_info() -> EvmKeyInfo {
    let key = ethers::core::k256::SecretKey::random(&mut rand::thread_rng());
    EvmKeyInfo::new(key.to_bytes().to_vec())
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Default)]
pub struct EthKeyAddress {
    inner: ethers::types::Address,
}

impl From<ethers::types::Address> for EthKeyAddress {
    fn from(inner: ethers::types::Address) -> Self {
        EthKeyAddress { inner }
    }
}

impl TryFrom<EthKeyAddress> for fvm_shared::address::Address {
    type Error = hex::FromHexError;

    fn try_from(value: EthKeyAddress) -> std::result::Result<Self, Self::Error> {
        Ok(fvm_shared::address::Address::from(
            &ipc_types::EthAddress::from_str(&value.to_string())?,
        ))
    }
}

impl From<EthKeyAddress> for ethers::types::Address {
    fn from(val: EthKeyAddress) -> Self {
        val.inner
    }
}

impl Display for EthKeyAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self == &Self::default() {
            write!(f, "default-key")
        } else {
            write!(f, "{:?}", self.inner)
        }
    }
}

impl TryFrom<EvmKeyInfo> for EthKeyAddress {
    type Error = anyhow::Error;

    fn try_from(value: EvmKeyInfo) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            inner: ethers::types::Address::try_from(value)?,
        })
    }
}
