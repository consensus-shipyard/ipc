// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::commands::node::config::NodeInitConfig;
use crate::commands::storage::config::{StorageConfig, StorageRunMode};
use crate::CommandLineHandler;
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Args;
use std::path::{Path, PathBuf};

pub(crate) struct InitStorage;

#[async_trait]
impl CommandLineHandler for InitStorage {
    type Arguments = InitStorageArgs;

    async fn handle(global: &crate::GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let node_cfg = NodeInitConfig::load(&args.node_config).with_context(|| {
            format!(
                "failed to read node config from {}",
                args.node_config.display()
            )
        })?;

        let node_home = node_cfg.home.clone();
        let storage_dir = node_home.join("storage");

        tokio::fs::create_dir_all(&storage_dir)
            .await
            .with_context(|| {
                format!(
                    "failed to create storage directory at {}",
                    storage_dir.display()
                )
            })?;

        let default_out = {
            let safe_id = sanitize_subnet_id(&node_cfg.subnet);
            global
                .config_dir()
                .join(format!("storage_{}.yaml", safe_id))
        };
        let out = args.out.clone().unwrap_or(default_out);

        let root = workspace_root();
        let secret_key_file = args
            .secret_key_file
            .clone()
            .unwrap_or_else(|| node_home.join("fendermint/validator.sk"));
        let storage_cfg = StorageConfig {
            node_home: node_home.clone(),
            node_config: args.node_config.clone(),
            storage_node_bin: root.join("target/release/node"),
            storage_gateway_bin: root.join("target/release/gateway"),
            network: "testnet".to_string(),
            tendermint_rpc_url: "http://127.0.0.1:26657".to_string(),
            eth_rpc_url: "http://127.0.0.1:8545".to_string(),
            secret_key_file,
            bls_key_file: storage_dir.join("bls_key.hex"),
            operator_rpc_url: "http://127.0.0.1:8081".to_string(),
            run_mode: StorageRunMode::Both,
            node_rpc_bind_addr: "127.0.0.1:8081".to_string(),
            iroh_node_path: storage_dir.join("iroh-node"),
            iroh_node_v4_addr: Some("0.0.0.0:11204".to_string()),
            node_batch_size: 10,
            node_poll_interval_secs: 5,
            node_max_concurrent_downloads: 10,
            objects_listen_addr: "127.0.0.1:8080".to_string(),
            iroh_gateway_path: storage_dir.join("iroh-gateway"),
            iroh_gateway_v4_addr: Some("0.0.0.0:11205".to_string()),
        };

        storage_cfg
            .save(&out)
            .with_context(|| format!("failed to write storage config to {}", out.display()))?;

        log::info!("Storage configuration generated at: {}", out.display());
        log::info!(
            "Optional: register operator with `ipc-cli storage run --config {} --register-operator`",
            out.display()
        );
        log::info!(
            "Using operator secret key file: {}",
            storage_cfg.secret_key_file.display()
        );
        log::info!(
            "Run storage services with `ipc-cli storage run --config {}`",
            out.display()
        );
        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(name = "init", about = "Generate storage-node config from node config")]
pub struct InitStorageArgs {
    #[arg(long, help = "Path to node init YAML config (node_*.yaml)")]
    pub node_config: PathBuf,
    #[arg(long, help = "Output path for generated storage YAML config")]
    pub out: Option<PathBuf>,
    #[arg(
        long,
        help = "Path to operator secp256k1 secret key file (base64). Defaults to <node-home>/fendermint/validator.sk"
    )]
    pub secret_key_file: Option<PathBuf>,
}

fn sanitize_subnet_id(subnet_id: &str) -> String {
    subnet_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}
