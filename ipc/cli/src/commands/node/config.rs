// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use fs_err as fs;
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::commands::subnet::create_genesis::{CreatedGenesis, GenesisConfig};
use crate::commands::subnet::init::config::JoinConfig;
use crate::commands::wallet::import::WalletImportArgs;

// Defines how the genesis state should be obtained
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GenesisSource {
    /// Create genesis by fetching from parent subnet using the provided config
    Create(GenesisConfig),
    /// Use an existing sealed genesis file
    Path(CreatedGenesis),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NodeInitConfig {
    /// Home directory of the node. Will be created if it does not exist
    pub home: PathBuf,
    /// Subnet to join, as a subnet ID
    pub subnet: String,
    /// Parent subnet, as a subnet ID
    pub parent: String,
    /// Validator key to import. Must be a secp256k1 key
    pub key: WalletImportArgs,
    /// Optional overrides config for CometBFT
    pub cometbft: Option<toml::Value>,
    /// Optional overrides config for Fendermint
    pub fendermint: Option<toml::Value>,
    /// Whether to automatically join the subnet as a validator. It is only possible for collateral based subnets
    pub join: Option<JoinConfig>,
    /// Source of the genesis state
    pub genesis: GenesisSource,
}

impl NodeInitConfig {
    /// Load and parse a YAML config file into `NodeInitConfig`
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(&path)?;
        let cfg: NodeInitConfig = serde_yaml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.as_ref().display(), e))?;
        Ok(cfg)
    }
}
