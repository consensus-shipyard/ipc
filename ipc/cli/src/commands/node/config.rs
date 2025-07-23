// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use fs_err as fs;
use serde::{Deserialize, Deserializer};
use std::path::{Path, PathBuf};

use crate::commands::subnet::create_genesis::{CreatedGenesis, GenesisConfig};
use crate::commands::subnet::init::config::JoinConfig;
use crate::commands::wallet::import::WalletImportArgs;

/// Deserialize a YAML literal block string as TOML value
///
/// This function handles the conversion from YAML literal blocks (using `|`) to TOML values.
/// YAML literal blocks are parsed as strings, but we need to convert them to TOML for
/// configuration overrides.
///
/// # Example
/// ```yaml
/// cometbft-overrides: |
///   [consensus]
///   timeout_commit = "5s"
/// ```
///
/// This YAML will be parsed as a string and then converted to a TOML value.
fn deserialize_toml_override<'de, D>(deserializer: D) -> Result<Option<toml::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    // Try to deserialize as Option<String> first to handle missing fields
    let opt_s: Option<String> = Option::deserialize(deserializer)?;

    match opt_s {
        Some(s) => {
            // Then parse the string as TOML
            toml::from_str(&s).map(Some).map_err(Error::custom)
        }
        None => Ok(None),
    }
}

/// Defines how the genesis state should be obtained
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
    /// Optional TOML overrides for CometBFT configuration
    #[serde(deserialize_with = "deserialize_toml_override", default)]
    pub cometbft_overrides: Option<toml::Value>,
    /// Optional TOML overrides for FenderMint configuration
    #[serde(deserialize_with = "deserialize_toml_override", default)]
    pub fendermint_overrides: Option<toml::Value>,
    /// Whether to automatically join the subnet as a validator (collateral-based subnets only)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_toml_override_valid() {
        let yaml_content = r#"
home: "/tmp/test"
subnet: "/r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly"
parent: "/r31337"
key:
  wallet-type: evm
  private-key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
cometbft-overrides: |
  [consensus]
  timeout_commit = "5s"
  [rpc]
  laddr = "tcp://0.0.0.0:26657"
genesis: !create
  base-fee: "1000"
  power-scale: 0
"#;

        let config: NodeInitConfig = serde_yaml::from_str(yaml_content).unwrap();

        assert!(config.cometbft_overrides.is_some());
        let overrides = config.cometbft_overrides.unwrap();

        // Check that it's a TOML table
        assert!(overrides.is_table());

        // Check specific values
        let table = overrides.as_table().unwrap();
        assert_eq!(
            table
                .get("consensus")
                .unwrap()
                .get("timeout_commit")
                .unwrap()
                .as_str()
                .unwrap(),
            "5s"
        );
        assert_eq!(
            table
                .get("rpc")
                .unwrap()
                .get("laddr")
                .unwrap()
                .as_str()
                .unwrap(),
            "tcp://0.0.0.0:26657"
        );
    }

    #[test]
    fn test_deserialize_toml_override_fendermint() {
        let yaml_content = r#"
home: "/tmp/test"
subnet: "/r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly"
parent: "/r31337"
key:
  wallet-type: evm
  private-key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
fendermint-overrides: |
  [app]
  max_validators = 100
  [broadcast]
  gas_overestimation_rate = 2.0
genesis: !create
  base-fee: "1000"
  power-scale: 0
"#;

        let config: NodeInitConfig = serde_yaml::from_str(yaml_content).unwrap();

        assert!(config.fendermint_overrides.is_some());
        let overrides = config.fendermint_overrides.unwrap();

        // Check specific values
        let table = overrides.as_table().unwrap();
        assert_eq!(
            table
                .get("app")
                .unwrap()
                .get("max_validators")
                .unwrap()
                .as_integer()
                .unwrap(),
            100
        );
        assert_eq!(
            table
                .get("broadcast")
                .unwrap()
                .get("gas_overestimation_rate")
                .unwrap()
                .as_float()
                .unwrap(),
            2.0
        );
    }

    #[test]
    fn test_deserialize_toml_override_invalid_toml() {
        let yaml_content = r#"
home: "/tmp/test"
subnet: "/r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly"
parent: "/r31337"
key:
  wallet-type: evm
  private-key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
cometbft-overrides: |
  [consensus
  timeout_commit = "5s"
genesis: !create
  base-fee: "1000"
  power-scale: 0
"#;

        let result: Result<NodeInitConfig, _> = serde_yaml::from_str(yaml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_toml_override_empty() {
        let yaml_content = r#"
home: "/tmp/test"
subnet: "/r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly"
parent: "/r31337"
key:
  wallet-type: evm
  private-key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
cometbft-overrides: ""
genesis: !create
  base-fee: "1000"
  power-scale: 0
"#;

        let config: NodeInitConfig = serde_yaml::from_str(yaml_content).unwrap();
        assert!(config.cometbft_overrides.is_some());

        // Empty string should parse as empty TOML
        let overrides = config.cometbft_overrides.unwrap();
        assert!(overrides.is_table());
        assert!(overrides.as_table().unwrap().is_empty());
    }

    #[test]
    fn test_deserialize_toml_override_missing() {
        let yaml_content = r#"
home: "/tmp/test"
subnet: "/r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly"
parent: "/r31337"
key:
  wallet-type: evm
  private-key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
genesis: !create
  base-fee: "1000"
  power-scale: 0
"#;

        let config: NodeInitConfig = serde_yaml::from_str(yaml_content).unwrap();

        assert!(config.cometbft_overrides.is_none());
        assert!(config.fendermint_overrides.is_none());
    }

    #[test]
    fn test_deserialize_toml_override_both() {
        let yaml_content = r#"
home: "/tmp/test"
subnet: "/r31337/t410fkzrz3mlkyufisiuae3scumllgalzuu3wxlxa2ly"
parent: "/r31337"
key:
  wallet-type: evm
  private-key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
cometbft-overrides: |
  [consensus]
  timeout_commit = "5s"
fendermint-overrides: |
  [app]
  max_validators = 100
genesis: !create
  base-fee: "1000"
  power-scale: 0
"#;

        let config: NodeInitConfig = serde_yaml::from_str(yaml_content).unwrap();

        assert!(config.cometbft_overrides.is_some());
        assert!(config.fendermint_overrides.is_some());

        let cometbft = config.cometbft_overrides.unwrap();
        let fendermint = config.fendermint_overrides.unwrap();

        assert_eq!(
            cometbft
                .get("consensus")
                .unwrap()
                .get("timeout_commit")
                .unwrap()
                .as_str()
                .unwrap(),
            "5s"
        );
        assert_eq!(
            fendermint
                .get("app")
                .unwrap()
                .get("max_validators")
                .unwrap()
                .as_integer()
                .unwrap(),
            100
        );
    }
}
