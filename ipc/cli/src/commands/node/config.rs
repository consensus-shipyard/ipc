// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use fs_err as fs;
use serde::{Deserialize, Deserializer, Serialize};
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

    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => {
            if s.trim().is_empty() {
                // Empty string should parse as empty TOML table
                Ok(Some(toml::Value::Table(toml::Table::new())))
            } else {
                let value: toml::Value = s.parse().map_err(D::Error::custom)?;
                Ok(Some(value))
            }
        }
        None => Ok(None),
    }
}

/// Serialize a TOML value as a YAML literal block string
///
/// This function converts TOML values back to YAML literal blocks for serialization.
fn serialize_toml_override<S>(value: &Option<toml::Value>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // use serde::ser::Error; // Not needed for this implementation

    match value {
        Some(value) => {
            let s = value.to_string();
            if s.trim().is_empty() {
                serializer.serialize_none()
            } else {
                serializer.serialize_str(&s)
            }
        }
        None => serializer.serialize_none(),
    }
}

/// Schema-driven CometBFT overrides instead of manual toml::Value manipulation
#[derive(Debug, Serialize, Deserialize)]
pub struct CometBftOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consensus: Option<ConsensusConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc: Option<RpcConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p2p: Option<P2pCometConfig>,
    // Allow additional unknown fields for flexibility
    #[serde(flatten)]
    pub extra: toml::Table,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_propose: Option<String>,
    #[serde(flatten)]
    pub extra: toml::Table,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub laddr: Option<String>,
    #[serde(flatten)]
    pub extra: toml::Table,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct P2pCometConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub laddr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistent_peers: Option<String>,
    #[serde(flatten)]
    pub extra: toml::Table,
}

impl CometBftOverrides {
    pub fn from_toml_value(value: toml::Value) -> anyhow::Result<Self> {
        let override_config: Self = value.try_into().map_err(|e| {
            anyhow::anyhow!("invalid CometBFT override configuration structure: {}", e)
        })?;
        Ok(override_config)
    }

    pub fn to_toml_value(&self) -> anyhow::Result<toml::Value> {
        toml::Value::try_from(self)
            .map_err(|e| anyhow::anyhow!("failed to serialize CometBFT overrides: {}", e))
    }
}

/// Schema-driven Fendermint overrides
#[derive(Debug, Serialize, Deserialize)]
pub struct FendermintOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broadcast: Option<BroadcastConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver: Option<ResolverOverrideConfig>,
    #[serde(flatten)]
    pub extra: toml::Table,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_validators: Option<u64>,
    #[serde(flatten)]
    pub extra: toml::Table,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_overestimation_rate: Option<f64>,
    #[serde(flatten)]
    pub extra: toml::Table,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolverOverrideConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection: Option<ConnectionOverrideConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovery: Option<DiscoveryOverrideConfig>,
    #[serde(flatten)]
    pub extra: toml::Table,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionOverrideConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_addr: Option<String>,
    #[serde(flatten)]
    pub extra: toml::Table,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryOverrideConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_addresses: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: toml::Table,
}

impl FendermintOverrides {
    pub fn from_toml_value(value: toml::Value) -> anyhow::Result<Self> {
        let override_config: Self = value.try_into().map_err(|e| {
            anyhow::anyhow!("invalid Fendermint override configuration structure: {}", e)
        })?;
        Ok(override_config)
    }

    pub fn to_toml_value(&self) -> anyhow::Result<toml::Value> {
        toml::Value::try_from(self)
            .map_err(|e| anyhow::anyhow!("failed to serialize Fendermint overrides: {}", e))
    }
}

/// P2P networking configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct P2pConfig {
    /// External IP address for peer connections (defaults to "127.0.0.1")
    pub external_ip: Option<String>,
    /// Network port configuration
    pub ports: Option<P2pPortsConfig>,
    /// Peer configuration from various sources
    pub peers: Option<P2pPeersConfig>,
}

/// Port configuration for different P2P services
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct P2pPortsConfig {
    /// CometBFT P2P port (defaults to 26656)
    pub cometbft: Option<u16>,
    /// IPLD Resolver P2P port (defaults to disabled)
    pub resolver: Option<u16>,
}

/// Peer configuration sources
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct P2pPeersConfig {
    /// List of peer info JSON files (local paths or URLs)
    pub peer_files: Option<Vec<String>>,
}

impl Default for P2pConfig {
    fn default() -> Self {
        Self {
            external_ip: Some("127.0.0.1".to_string()),
            ports: Some(P2pPortsConfig::default()),
            peers: None,
        }
    }
}

impl Default for P2pPortsConfig {
    fn default() -> Self {
        Self {
            cometbft: Some(26656),
            resolver: None, // Disabled by default
        }
    }
}

/// Peer information that gets serialized to peer-info.json
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct PeerInfo {
    /// General node information
    pub node_info: NodeInfo,
    /// CometBFT peer information
    pub cometbft: CometBftPeerInfo,
    /// Fendermint resolver peer information  
    pub fendermint: FendermintPeerInfo,
}

/// General node information
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NodeInfo {
    /// External IP address configured for this node
    pub external_ip: String,
    /// Timestamp when this peer info was generated
    pub generated_at: String,
}

/// CometBFT peer connection information
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct CometBftPeerInfo {
    /// CometBFT node ID (hex-encoded)
    pub node_id: String,
    /// Port where CometBFT P2P listens
    pub listen_port: u16,
    /// Full external address for peer connections
    pub external_address: String,
    /// Ready-to-use peer string for CometBFT config
    pub peer_string: String,
}

/// Fendermint IPLD resolver peer information
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct FendermintPeerInfo {
    /// Libp2p peer ID (base58-encoded)
    pub peer_id: String,
    /// Port where resolver listens (if enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_port: Option<u16>,
    /// Full multiaddr for peer connections (if resolver enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiaddr: Option<String>,
}

/// Defines how the genesis state should be obtained
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GenesisSource {
    /// Create genesis by fetching from parent subnet using the provided config
    Create(GenesisConfig),
    /// Use an existing sealed genesis file
    Path(CreatedGenesis),
}

#[derive(Debug, Serialize, Deserialize)]
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
    /// P2P networking configuration
    pub p2p: Option<P2pConfig>,
    /// Optional TOML overrides for CometBFT configuration
    #[serde(
        deserialize_with = "deserialize_toml_override",
        serialize_with = "serialize_toml_override",
        default
    )]
    pub cometbft_overrides: Option<toml::Value>,
    /// Optional TOML overrides for FenderMint configuration
    #[serde(
        deserialize_with = "deserialize_toml_override",
        serialize_with = "serialize_toml_override",
        default
    )]
    pub fendermint_overrides: Option<toml::Value>,
    /// Whether to automatically join the subnet as a validator (collateral-based subnets only)
    pub join: Option<JoinConfig>,
    /// Source of the genesis state
    pub genesis: GenesisSource,
}

impl NodeInitConfig {
    /// Load and parse a YAML config file into `NodeInitConfig`
    #[cfg_attr(not(test), allow(dead_code))]
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
