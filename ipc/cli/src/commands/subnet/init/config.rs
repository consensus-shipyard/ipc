// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Configuration definitions for the `subnet init` command

pub(crate) use crate::commands::deploy::DeployConfig;
pub(crate) use crate::commands::subnet::{
    create::SubnetCreateConfig, join::JoinSubnetArgs, set_federated_power::SetFederatedPowerArgs,
};
pub(crate) use crate::commands::wallet::import::WalletImportArgs;

use anyhow::ensure;
use ethers::{types::H160, utils::keccak256};
use fendermint_app::options::parse::{parse_network_version, parse_token_amount};
use fs_err as fs;
use fvm_shared::{econ::TokenAmount, version::NetworkVersion};
use hex::FromHex;
use ipc_api::subnet::PermissionMode;
use serde::{Deserialize, Deserializer};
use std::path::Path;

/// Convert an uncompressed secp256k1 public key (0x04-prefixed) into an Ethereum address
fn public_key_to_address(hex_str: &str) -> anyhow::Result<H160> {
    let key = hex_str
        .strip_prefix("0x")
        .or_else(|| hex_str.strip_prefix("0X"))
        .unwrap_or(hex_str);
    let bytes = Vec::from_hex(key).map_err(|e| anyhow::anyhow!("Invalid public key hex: {}", e))?;
    ensure!(
        bytes.len() == 65 && bytes[0] == 0x04,
        "Expected 65-byte uncompressed key starting with 0x04"
    );
    let hash = keccak256(&bytes[1..]);
    Ok(H160::from_slice(&hash[12..]))
}

/// Deserialize a string into `NetworkVersion`
fn de_network_version<'de, D>(deserializer: D) -> Result<NetworkVersion, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_network_version(&s).map_err(serde::de::Error::custom)
}

/// Deserialize a string into `TokenAmount`
fn de_token_amount<'de, D>(deserializer: D) -> Result<TokenAmount, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_token_amount(&s).map_err(serde::de::Error::custom)
}

/// Config for setting federated or static validator power
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PowerConfig {
    pub validator_pubkeys: Vec<String>,
    pub validator_power: Vec<u64>,
}

impl PowerConfig {
    pub fn into_args(
        self,
        subnet: String,
        sender: String,
    ) -> anyhow::Result<SetFederatedPowerArgs> {
        let addresses = self
            .validator_pubkeys
            .iter()
            .map(|pk| public_key_to_address(pk).map(|a| format!("{:#x}", a)))
            .collect::<anyhow::Result<_>>()?;

        let pubkeys = self
            .validator_pubkeys
            .into_iter()
            .map(|s| s.trim_start_matches("0x").to_string())
            .collect();

        let power = self.validator_power.into_iter().map(u128::from).collect();

        Ok(SetFederatedPowerArgs {
            from: sender,
            subnet,
            validator_addresses: addresses,
            validator_pubkeys: pubkeys,
            validator_power: power,
        })
    }
}

/// Config for joining a subnet
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct JoinConfig {
    pub from: String,
    pub collateral: f64,
    pub initial_balance: Option<f64>,
}

impl JoinConfig {
    pub fn into_args(self, subnet: String) -> JoinSubnetArgs {
        JoinSubnetArgs {
            from: Some(self.from),
            subnet,
            collateral: self.collateral,
            initial_balance: self.initial_balance,
        }
    }
}

/// Activation modes for the subnet
#[derive(Debug, Deserialize)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum ActivateConfig {
    Federated {
        #[serde(flatten)]
        power: PowerConfig,
    },
    Static {
        #[serde(flatten)]
        power: PowerConfig,
    },
    Collateral {
        validators: Vec<JoinConfig>,
    },
}

/// Genesis parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GenesisConfig {
    #[serde(
        default = "GenesisConfig::default_network_version",
        deserialize_with = "de_network_version"
    )]
    pub network_version: NetworkVersion,

    #[serde(
        default = "GenesisConfig::default_base_fee",
        deserialize_with = "de_token_amount"
    )]
    pub base_fee: TokenAmount,

    #[serde(default = "GenesisConfig::default_power_scale")]
    pub power_scale: i8,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        GenesisConfig {
            network_version: GenesisConfig::default_network_version(),
            base_fee: GenesisConfig::default_base_fee(),
            power_scale: GenesisConfig::default_power_scale(),
        }
    }
}

impl GenesisConfig {
    const fn default_network_version() -> NetworkVersion {
        NetworkVersion::V21
    }

    fn default_base_fee() -> TokenAmount {
        parse_token_amount("1000").unwrap()
    }

    const fn default_power_scale() -> i8 {
        3
    }
}

/// Top-level YAML schema for `subnet init`
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SubnetInitConfig {
    /// Wallets to import into the IPC cli keystore
    pub import_wallets: Option<Vec<WalletImportArgs>>,

    /// Deploy contracts to the network. If not specified, the contracts are not deployed.
    #[serde(default)]
    pub deploy: Option<DeployConfig>,

    /// Configuration for subnet creation.
    pub create: SubnetCreateConfig,

    /// Configuration for subnet activation.
    #[serde(default)]
    pub activate: Option<ActivateConfig>,

    /// Configuration for subnet genesis
    #[serde(default)]
    pub genesis: Option<GenesisConfig>,
}

impl SubnetInitConfig {
    /// Ensure activation mode matches permission mode
    pub fn validate(&self) -> anyhow::Result<()> {
        if let Some(act) = &self.activate {
            match (&self.create.permission_mode, act) {
                (PermissionMode::Federated, ActivateConfig::Federated { .. })
                | (PermissionMode::Static, ActivateConfig::Static { .. })
                | (PermissionMode::Collateral, ActivateConfig::Collateral { .. }) => Ok(()),
                (pm, _) => anyhow::bail!(
                    "activation.mode `{:?}` does not match create.permission_mode `{:?}`",
                    act,
                    pm
                ),
            }
        } else {
            Ok(())
        }
    }

    /// Load and parse a YAML config file into `SubnetInitConfig`
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(&path)?;
        let cfg: SubnetInitConfig = serde_yaml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.as_ref().display(), e))?;
        cfg.validate()?;
        Ok(cfg)
    }
}
