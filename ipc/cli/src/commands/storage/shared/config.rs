// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::Result;
use fs_err as fs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

fn ipc_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ipc")
}

pub fn legacy_storage_config_path() -> PathBuf {
    ipc_config_dir().join("storage.yaml")
}

pub fn default_storage_provider_config_path() -> PathBuf {
    ipc_config_dir()
        .join("storage")
        .join("node")
        .join("config.yaml")
}

pub fn default_storage_client_config_path() -> PathBuf {
    ipc_config_dir()
        .join("storage")
        .join("client")
        .join("config.yaml")
}

fn old_storage_provider_config_path() -> PathBuf {
    ipc_config_dir().join("storage-provider.yaml")
}

fn old_storage_client_config_path() -> PathBuf {
    ipc_config_dir().join("storage-client.yaml")
}

/// Resolve provider config path with fallback to legacy storage.yaml.
pub fn resolve_provider_config_path(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(path) = explicit {
        return path;
    }
    let provider = default_storage_provider_config_path();
    if provider.exists() {
        return provider;
    }
    let old_provider = old_storage_provider_config_path();
    if old_provider.exists() {
        return old_provider;
    }
    let legacy = legacy_storage_config_path();
    if legacy.exists() {
        return legacy;
    }
    provider
}

/// Resolve client config path with fallback to legacy storage.yaml.
pub fn resolve_client_config_path(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(path) = explicit {
        return path;
    }
    let client = default_storage_client_config_path();
    if client.exists() {
        return client;
    }
    let old_client = old_storage_client_config_path();
    if old_client.exists() {
        return old_client;
    }
    let legacy = legacy_storage_config_path();
    if legacy.exists() {
        return legacy;
    }
    client
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct StorageClientConfig {
    /// Tendermint RPC endpoint used for read-only chain queries.
    pub tendermint_rpc_url: String,
    /// Gateway URL for object download/read operations.
    pub gateway_url: Option<String>,
    /// Optional default account address for user-oriented queries.
    pub address: Option<String>,
}

impl StorageClientConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)?;
        let cfg: StorageClientConfig = serde_yaml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.display(), e))?;
        Ok(cfg)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = serde_yaml::to_string(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn default_with_local_rpc() -> Self {
        Self {
            tendermint_rpc_url: "http://127.0.0.1:26657".to_string(),
            gateway_url: None,
            address: None,
        }
    }
}

pub fn resolve_client_gateway_url(
    explicit_gateway: Option<&str>,
    explicit_config: Option<PathBuf>,
    interactive: bool,
) -> Result<String> {
    if let Some(url) = explicit_gateway {
        return Ok(url.to_string());
    }

    if let Ok(url) = std::env::var("IPC_STORAGE_GATEWAY") {
        if !url.is_empty() {
            return Ok(url);
        }
    }

    let config_path = resolve_client_config_path(explicit_config);
    if config_path.exists() {
        let mut cfg = if let Ok(client_cfg) = StorageClientConfig::load(&config_path) {
            client_cfg
        } else {
            StorageClientConfig::default_with_local_rpc()
        };
        if let Some(url) = &cfg.gateway_url {
            if !url.is_empty() {
                return Ok(url.clone());
            }
        }
        if interactive {
            println!("Gateway URL not configured.");
            println!("Please enter the storage gateway URL (e.g., http://localhost:8080):");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let url = input.trim().to_string();
            if url.is_empty() {
                anyhow::bail!("Gateway URL cannot be empty");
            }
            cfg.gateway_url = Some(url.clone());
            cfg.save(&config_path)?;
            println!("Gateway URL saved to {}", config_path.display());
            return Ok(url);
        }
    }

    anyhow::bail!(
        "Gateway URL not configured. Set via:\n\
        1. --gateway flag\n\
        2. IPC_STORAGE_GATEWAY environment variable\n\
        3. gateway_url in storage client config (`ipc-cli storage client init` / `... set`)"
    )
}

/// Which storage components to run.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum StorageRunMode {
    Node,
    Gateway,
    #[default]
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StorageConfig {
    /// IPC node home, usually "~/.node-ipc".
    pub node_home: PathBuf,
    /// Source node-init config used to derive defaults.
    pub node_config: PathBuf,
    /// Path to ipc-storage node binary (ipc-decentralized-storage `node`).
    pub storage_node_bin: PathBuf,
    /// Path to ipc-storage gateway binary.
    pub storage_gateway_bin: PathBuf,

    /// FM network passed to storage binaries (testnet/mainnet).
    pub network: String,

    /// Tendermint RPC endpoint of the subnet node.
    pub tendermint_rpc_url: String,
    /// EVM JSON-RPC endpoint of the subnet node.
    pub eth_rpc_url: String,

    /// Secp256k1 key for signing chain transactions.
    pub secret_key_file: PathBuf,
    /// BLS key used by storage node/operator.
    pub bls_key_file: PathBuf,

    /// Operator API URL published on-chain during registration.
    pub operator_rpc_url: String,

    /// Run mode for `ipc-cli storage run`.
    pub run_mode: StorageRunMode,

    /// Storage-node settings
    pub node_rpc_bind_addr: String,
    pub iroh_node_path: PathBuf,
    pub iroh_node_v4_addr: Option<String>,
    pub node_batch_size: u32,
    pub node_poll_interval_secs: u64,
    pub node_max_concurrent_downloads: usize,

    /// Gateway settings
    pub objects_listen_addr: String,
    pub iroh_gateway_path: PathBuf,
    pub iroh_gateway_v4_addr: Option<String>,
}

impl StorageConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)?;
        let cfg: StorageConfig = serde_yaml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.display(), e))?;
        Ok(cfg)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = serde_yaml::to_string(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}
