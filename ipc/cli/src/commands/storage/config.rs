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
    /// Storage home directory for keys, iroh data, etc.
    pub node_home: PathBuf,
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

    /// Gateway URL for storage CLI operations (optional)
    /// If not set, CLI commands will check env var or prompt
    pub gateway_url: Option<String>,
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

    /// Get the gateway URL from various sources
    ///
    /// Priority order:
    /// 1. Explicit gateway_url parameter (CLI flag)
    /// 2. IPC_STORAGE_GATEWAY environment variable
    /// 3. gateway_url field in config file
    /// 4. Prompt user if interactive is true
    ///
    /// If a new URL is discovered via prompt, it will be saved to the config.
    pub fn get_gateway_url<P: AsRef<Path>>(
        &mut self,
        explicit_url: Option<&str>,
        config_path: Option<P>,
        interactive: bool,
    ) -> Result<String> {
        // 1. Check explicit URL (CLI flag)
        if let Some(url) = explicit_url {
            return Ok(url.to_string());
        }

        // 2. Check environment variable
        if let Ok(url) = std::env::var("IPC_STORAGE_GATEWAY") {
            if !url.is_empty() {
                return Ok(url);
            }
        }

        // 3. Check config file
        if let Some(url) = &self.gateway_url {
            if !url.is_empty() {
                return Ok(url.clone());
            }
        }

        // 4. Prompt user if interactive
        if interactive {
            println!("Gateway URL not configured.");
            println!("Please enter the storage gateway URL (e.g., http://localhost:8080):");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let url = input.trim().to_string();

            if url.is_empty() {
                anyhow::bail!("Gateway URL cannot be empty");
            }

            // Save to config if path provided
            if let Some(path) = config_path {
                self.gateway_url = Some(url.clone());
                self.save(path)?;
                println!("Gateway URL saved to config.");
            }

            return Ok(url);
        }

        anyhow::bail!(
            "Gateway URL not configured. Set via:\n\
            1. --gateway flag\n\
            2. IPC_STORAGE_GATEWAY environment variable\n\
            3. gateway_url in storage config file"
        )
    }
}
