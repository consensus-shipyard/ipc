// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::commands::storage::config::{StorageConfig, StorageRunMode};
use crate::CommandLineHandler;
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Args;
use fendermint_crypto::{to_b64, SecretKey};
use fendermint_vm_actor_interface::eam::EthAddress;
use fvm_shared::address::{set_current_network, Address, Network};
use rand::thread_rng;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct InitStorage;

#[async_trait]
impl CommandLineHandler for InitStorage {
    type Arguments = InitStorageArgs;

    async fn handle(global: &crate::GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let node_home = args
            .home
            .clone()
            .unwrap_or_else(|| default_storage_home());

        let storage_dir = node_home.join("storage");

        tokio::fs::create_dir_all(&storage_dir)
            .await
            .with_context(|| {
                format!(
                    "failed to create storage directory at {}",
                    storage_dir.display()
                )
            })?;

        let out = args
            .out
            .clone()
            .unwrap_or_else(|| global.config_dir().join("storage.yaml"));

        let root = workspace_root();
        let secret_key_file = args
            .secret_key_file
            .clone()
            .unwrap_or_else(|| storage_dir.join("operator.sk"));
        ensure_operator_secret_key(&secret_key_file)?;

        // Storage tooling defaults to testnet addressing unless explicitly overridden at runtime.
        set_current_network(Network::Testnet);
        let operator_sk = read_operator_secret_key(&secret_key_file).with_context(|| {
            format!(
                "failed to read operator key from {}",
                secret_key_file.display()
            )
        })?;
        let operator_pk = operator_sk.public_key();
        let operator_f1 = Address::new_secp256k1(&operator_pk.serialize())
            .context("failed to derive operator f1 address")?;
        let operator_f410_eth = EthAddress::new_secp256k1(&operator_pk.serialize())
            .context("failed to derive operator delegated address")?;
        let operator_f410 = Address::new_delegated(10, &operator_f410_eth.0)
            .context("failed to construct operator f410 address")?;
        let storage_cfg = StorageConfig {
            node_home: node_home.clone(),
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
            gateway_url: None,
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
        log::info!("Operator funding addresses:");
        log::info!("  - delegated (fund this): {}", operator_f410);
        log::info!("  - native (diagnostic only): {}", operator_f1);
        log::info!(
            "Recommendation: cross-fund the delegated operator address above before `storage run --register-operator`."
        );
        log::info!(
            "Run storage services with `ipc-cli storage run --config {}`",
            out.display()
        );
        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(name = "init", about = "Generate default storage config")]
pub struct InitStorageArgs {
    #[arg(
        long,
        help = "Storage home directory for keys and data (defaults to ~/.ipc-storage)"
    )]
    pub home: Option<PathBuf>,
    #[arg(long, help = "Output path for generated storage YAML config")]
    pub out: Option<PathBuf>,
    #[arg(
        long,
        help = "Path to operator secp256k1 secret key file (base64). Defaults to <home>/storage/operator.sk"
    )]
    pub secret_key_file: Option<PathBuf>,
}

fn default_storage_home() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ipc-storage")
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn ensure_operator_secret_key(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create operator key directory at {}",
                parent.display()
            )
        })?;
    }

    let mut rng = thread_rng();
    let sk = SecretKey::random(&mut rng);
    let sk_bytes = sk.serialize();
    let sk_b64 = to_b64(sk_bytes.as_ref());
    fs::write(path, sk_b64)
        .with_context(|| format!("failed to write operator key to {}", path.display()))?;
    log::info!("Generated dedicated operator key at {}", path.display());
    Ok(())
}

fn read_operator_secret_key(path: &Path) -> Result<SecretKey> {
    let sk = fendermint_rpc::message::SignedMessageFactory::read_secret_key(path)
        .with_context(|| format!("failed to parse secret key at {}", path.display()))?;
    Ok(sk)
}
