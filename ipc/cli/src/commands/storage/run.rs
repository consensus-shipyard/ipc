// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::commands::storage::config::{StorageConfig, StorageRunMode};
use crate::CommandLineHandler;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use clap::Args;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::str::FromStr;

pub(crate) struct RunStorage;

#[async_trait]
impl CommandLineHandler for RunStorage {
    type Arguments = RunStorageArgs;

    async fn handle(_global: &crate::GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let cfg = StorageConfig::load(&args.config).with_context(|| {
            format!(
                "failed to load storage config from {}",
                args.config.display()
            )
        })?;

        preflight(&cfg).await?;

        if args.register_operator {
            run_register_operator(&cfg)?;
        }

        let mode = args.mode.unwrap_or(cfg.run_mode);
        match mode {
            StorageRunMode::Node => run_node(&cfg),
            StorageRunMode::Gateway => run_gateway(&cfg),
            StorageRunMode::Both => run_both(&cfg),
        }
    }
}

#[derive(Debug, Args)]
#[command(name = "run", about = "Run storage node and gateway")]
pub struct RunStorageArgs {
    #[arg(long, help = "Path to storage YAML config")]
    pub config: std::path::PathBuf,
    #[arg(long, help = "Register node operator before launching services")]
    pub register_operator: bool,
    #[arg(long, help = "Override run mode (node|gateway|both)")]
    pub mode: Option<StorageRunMode>,
}

fn run_register_operator(cfg: &StorageConfig) -> Result<()> {
    log::info!("Registering storage node operator");
    let status = Command::new(&cfg.storage_node_bin)
        .arg("register-operator")
        .arg("--bls-key-file")
        .arg(&cfg.bls_key_file)
        .arg("--secret-key-file")
        .arg(&cfg.secret_key_file)
        .arg("--operator-rpc-url")
        .arg(&cfg.operator_rpc_url)
        .arg("--chain-rpc-url")
        .arg(&cfg.tendermint_rpc_url)
        .env("FM_NETWORK", &cfg.network)
        .status()
        .with_context(|| {
            format!(
                "failed to execute register-operator using {}",
                cfg.storage_node_bin.display()
            )
        })?;

    if !status.success() {
        bail!("register-operator exited with status {}", status);
    }
    Ok(())
}

fn run_node(cfg: &StorageConfig) -> Result<()> {
    log::info!("Starting storage node");
    let status = node_command(cfg)?
        .status()
        .context("failed to start storage node process")?;
    if !status.success() {
        bail!("storage node exited with status {}", status);
    }
    Ok(())
}

fn run_gateway(cfg: &StorageConfig) -> Result<()> {
    log::info!("Starting storage gateway");
    let status = gateway_command(cfg)?
        .status()
        .context("failed to start storage gateway process")?;
    if !status.success() {
        bail!("storage gateway exited with status {}", status);
    }
    Ok(())
}

fn run_both(cfg: &StorageConfig) -> Result<()> {
    log::info!("Starting storage gateway + storage node");
    let mut gateway = gateway_command(cfg)?
        .spawn()
        .context("failed to spawn storage gateway")?;
    let mut node = node_command(cfg)?
        .spawn()
        .context("failed to spawn storage node")?;

    let node_status = node.wait().context("failed to wait for storage node")?;
    if let Err(e) = terminate_child(&mut gateway) {
        log::warn!("failed to stop gateway after node exit: {}", e);
    }

    if !node_status.success() {
        bail!("storage node exited with status {}", node_status);
    }
    Ok(())
}

fn terminate_child(child: &mut Child) -> Result<()> {
    if child.try_wait()?.is_none() {
        child.kill()?;
    }
    let _ = child.wait();
    Ok(())
}

fn node_command(cfg: &StorageConfig) -> Result<Command> {
    let node_bin = resolve_bin_path(&cfg.storage_node_bin, "node")?;
    let mut cmd = Command::new(&node_bin);
    cmd.arg("run")
        .arg("--secret-key-file")
        .arg(&cfg.bls_key_file)
        .arg("--iroh-path")
        .arg(&cfg.iroh_node_path)
        .arg("--rpc-url")
        .arg(&cfg.tendermint_rpc_url)
        .arg("--eth-rpc-url")
        .arg(&cfg.eth_rpc_url)
        .arg("--batch-size")
        .arg(cfg.node_batch_size.to_string())
        .arg("--poll-interval-secs")
        .arg(cfg.node_poll_interval_secs.to_string())
        .arg("--max-concurrent-downloads")
        .arg(cfg.node_max_concurrent_downloads.to_string())
        .arg("--rpc-bind-addr")
        .arg(&cfg.node_rpc_bind_addr)
        .env("FM_NETWORK", &cfg.network)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if let Some(v4) = &cfg.iroh_node_v4_addr {
        cmd.arg("--iroh-v4-addr").arg(v4);
    }
    Ok(cmd)
}

fn gateway_command(cfg: &StorageConfig) -> Result<Command> {
    let gateway_bin = resolve_bin_path(&cfg.storage_gateway_bin, "gateway")?;
    let mut cmd = Command::new(&gateway_bin);
    cmd.arg("--secret-key-file")
        .arg(&cfg.secret_key_file)
        .arg("--bls-key-file")
        .arg(&cfg.bls_key_file)
        .arg("--rpc-url")
        .arg(&cfg.tendermint_rpc_url)
        .arg("--objects-listen-addr")
        .arg(&cfg.objects_listen_addr)
        .arg("--iroh-path")
        .arg(&cfg.iroh_gateway_path)
        .env("FM_NETWORK", &cfg.network)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if let Some(v4) = &cfg.iroh_gateway_v4_addr {
        cmd.arg("--iroh-v4-addr").arg(v4);
    }
    Ok(cmd)
}

async fn preflight(cfg: &StorageConfig) -> Result<()> {
    if resolve_bin_path(&cfg.storage_node_bin, "node").is_err() {
        bail!(
            "storage node binary not found at {} (build: cargo build --release -p ipc-decentralized-storage --bin node --bin gateway)",
            cfg.storage_node_bin.display()
        );
    }
    if resolve_bin_path(&cfg.storage_gateway_bin, "gateway").is_err() {
        bail!(
            "storage gateway binary not found at {}",
            cfg.storage_gateway_bin.display()
        );
    }
    if !cfg.secret_key_file.exists() {
        bail!(
            "operator secret key file not found: {}",
            cfg.secret_key_file.display()
        );
    }
    if cfg
        .secret_key_file
        .ends_with(std::path::Path::new("fendermint/validator.sk"))
    {
        log::warn!(
            "Storage config uses validator key ({}). If register-operator fails with sender/account errors, regenerate config with --secret-key-file pointing to a dedicated funded operator key.",
            cfg.secret_key_file.display()
        );
    }

    if !cfg.node_home.exists() {
        bail!(
            "node home directory does not exist: {}",
            cfg.node_home.display()
        );
    }

    if !cfg.iroh_node_path.exists() {
        tokio::fs::create_dir_all(&cfg.iroh_node_path)
            .await
            .with_context(|| {
                format!(
                    "failed to create node iroh directory at {}",
                    cfg.iroh_node_path.display()
                )
            })?;
    }
    if !cfg.iroh_gateway_path.exists() {
        tokio::fs::create_dir_all(&cfg.iroh_gateway_path)
            .await
            .with_context(|| {
                format!(
                    "failed to create gateway iroh directory at {}",
                    cfg.iroh_gateway_path.display()
                )
            })?;
    }
    Ok(())
}

fn resolve_bin_path(configured: &Path, bin_name: &str) -> Result<PathBuf> {
    if configured.exists() {
        return Ok(configured.to_path_buf());
    }

    let fallback = workspace_root().join("target/release").join(bin_name);
    if fallback.exists() {
        log::warn!(
            "Configured {} binary not found at {}; using fallback {}",
            bin_name,
            configured.display(),
            fallback.display()
        );
        return Ok(fallback);
    }

    bail!(
        "{} binary not found at {} or fallback {}",
        bin_name,
        configured.display(),
        fallback.display()
    )
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

impl FromStr for StorageRunMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "node" => Ok(Self::Node),
            "gateway" => Ok(Self::Gateway),
            "both" => Ok(Self::Both),
            _ => bail!("invalid run mode '{}', expected node|gateway|both", s),
        }
    }
}
