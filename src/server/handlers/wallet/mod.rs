// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
pub mod balances;
pub mod export;
pub mod import;
pub mod new;
pub mod remove;

/// The wallet type, i.e. for which network
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "network_type")]
pub enum WalletType {
    Evm,
    Fvm,
}

impl FromStr for WalletType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "evm" => Self::Evm,
            "fvm" => Self::Fvm,
            _ => return Err(anyhow!("invalid wallet type")),
        })
    }
}
