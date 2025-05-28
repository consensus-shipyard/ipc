// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

pub(crate) mod perm;
pub use perm::*;

pub(crate) mod keystore;
pub use keystore::*;

pub(crate) mod errors;
pub use errors::*;

pub(crate) mod crypto;
pub use crypto::*;

mod evm;
mod fvm;

#[cfg(feature = "with-ethers")]
pub use crate::evm::{
    DEFAULT_KEYSTORE_NAME,
};

pub use crate::fvm::*;

/// WalletType determines the kind of keys and wallets
/// supported in the keystore
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "network_type")]
pub enum WalletType {
    Etherium,
    Filecoin,
}

impl FromStr for WalletType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "etherium"|"evm" => Self::Etherium,
            "filecoin"|"fvm" => Self::Filecoin,
            _ => return Err(anyhow!("invalid wallet type")),
        })
    }
}
