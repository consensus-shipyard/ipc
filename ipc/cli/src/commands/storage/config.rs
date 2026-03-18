// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::Result;
use fs_err as fs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
        let contents = serde_yaml::to_string(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}
