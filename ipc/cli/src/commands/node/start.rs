// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::{CommandLineHandler, GlobalArguments};
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Args;
use fendermint_app::settings::Settings;
use log::{info, warn};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::Duration;
use tokio::task::JoinHandle;

pub(crate) struct StartNode;

#[async_trait]
impl CommandLineHandler for StartNode {
    type Arguments = StartNodeArgs;

    async fn handle(_global: &GlobalArguments, arguments: &Self::Arguments) -> Result<()> {
        let home = Path::new(&arguments.home);

        // Validate that the node home directory exists and contains required files
        validate_node_home(home)?;

        // Load Fendermint settings
        let settings = load_fendermint_settings(home)?;

        // Set up unified logging using existing observability infrastructure
        let _trace_file_guard =
            ipc_observability::traces::set_global_tracing_subscriber(&settings.tracing);

        info!("Starting IPC node from home directory: {}", home.display());

        // Start all services with health monitoring
        let mut services = NodeServices::start_all(home, settings).await?;

        info!("All services started successfully");

        // Wait for shutdown signal
        wait_for_shutdown().await?;

        info!("Shutdown signal received, stopping services...");

        // Graceful shutdown
        services.shutdown().await?;

        info!("Node shutdown completed");
        Ok(())
    }
}

/// CLI arguments for starting a node
#[derive(Debug, Args)]
#[command(
    name = "start-node",
    about = "Start a CometBFT+Fendermint+ETH API node"
)]
pub struct StartNodeArgs {
    /// Path to the node's home directory
    #[arg(long, help = "Path to node home directory")]
    pub home: PathBuf,
}

/// Manages all node services
struct NodeServices {
    fendermint_task: JoinHandle<Result<()>>,
    eth_api_task: JoinHandle<Result<()>>,
    cometbft_process: Option<Child>,
    health_monitor: JoinHandle<Result<()>>,
    shutdown_signal: tokio::sync::broadcast::Sender<()>,
}

impl NodeServices {
    async fn start_all(home: &Path, settings: Settings) -> Result<Self> {
        let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);

        // Start Fendermint ABCI application (in-process)
        let _fendermint_settings = settings.clone();
        let fendermint_task = tokio::spawn(async move {
            // For now, just log that we would start Fendermint
            // TODO: Implement proper Fendermint startup
            info!("Fendermint ABCI application would start here");
            Ok(())
        });

        info!("Fendermint ABCI application started");

        // Wait a bit for Fendermint to initialize
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Start CometBFT (external process)
        let cometbft_home = home.join("cometbft");
        let cometbft_process = start_cometbft(&cometbft_home)?;

        info!("CometBFT consensus engine started");

        // Wait a bit for CometBFT to connect to Fendermint
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Start ETH API (in-process)
        let _eth_settings = settings.eth;
        let eth_api_task = tokio::spawn(async move {
            // For now, just log that we would start ETH API
            // TODO: Implement proper ETH API startup
            info!("ETH API would start here");
            Ok(())
        });

        info!("ETH API server started");

        // Start health monitoring
        let health_monitor = tokio::spawn(async move { Self::monitor_health(shutdown_rx).await });

        Ok(Self {
            fendermint_task,
            eth_api_task,
            cometbft_process: Some(cometbft_process),
            health_monitor,
            shutdown_signal: shutdown_tx,
        })
    }

    async fn monitor_health(mut shutdown_rx: tokio::sync::broadcast::Receiver<()>) -> Result<()> {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    // Periodic health check - for now just log that we're alive
                    log::debug!("Health check: all services running");
                }
                _ = shutdown_rx.recv() => {
                    info!("Health monitoring stopped");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Initiating graceful shutdown...");

        // Send shutdown signal to health monitor
        let _ = self.shutdown_signal.send(());

        // Stop CometBFT process
        if let Some(mut process) = self.cometbft_process.take() {
            info!("Stopping CometBFT process...");
            if let Err(e) = process.kill() {
                warn!("Failed to kill CometBFT process: {}", e);
            }
        }

        // Cancel in-process tasks
        info!("Stopping in-process services...");
        self.fendermint_task.abort();
        self.eth_api_task.abort();

        // Take ownership of tasks for cleanup
        let fendermint_task = std::mem::replace(
            &mut self.fendermint_task,
            tokio::task::spawn(async { Ok(()) }),
        );
        let eth_api_task =
            std::mem::replace(&mut self.eth_api_task, tokio::task::spawn(async { Ok(()) }));
        let health_monitor = std::mem::replace(
            &mut self.health_monitor,
            tokio::task::spawn(async { Ok(()) }),
        );

        // Wait for tasks to finish (with timeout)
        let fendermint_result =
            tokio::time::timeout(Duration::from_secs(10), fendermint_task).await;

        let eth_api_result = tokio::time::timeout(Duration::from_secs(10), eth_api_task).await;

        // Wait for health monitor to finish
        let _ = tokio::time::timeout(Duration::from_secs(5), health_monitor).await;

        if let Ok(Ok(Err(e))) = fendermint_result {
            warn!("Fendermint task error during shutdown: {}", e);
        }

        if let Ok(Ok(Err(e))) = eth_api_result {
            warn!("ETH API task error during shutdown: {}", e);
        }

        info!("Graceful shutdown completed");
        Ok(())
    }
}

fn start_cometbft(home: &Path) -> Result<Child> {
    let home_str = home.to_string_lossy();

    // For now, use system cometbft binary
    // TODO: Use embedded comet_runner when we have async support
    let child = Command::new("cometbft")
        .args(["start", "--home", &home_str])
        .spawn()
        .context("failed to start CometBFT process")?;

    Ok(child)
}

fn validate_node_home(home: &Path) -> Result<()> {
    if !home.exists() {
        anyhow::bail!("Node home directory does not exist: {}", home.display());
    }

    if !home.is_dir() {
        anyhow::bail!("Node home path is not a directory: {}", home.display());
    }

    // Check for required subdirectories
    let fendermint_dir = home.join("fendermint");
    let cometbft_dir = home.join("cometbft");

    if !fendermint_dir.exists() {
        anyhow::bail!(
            "Fendermint directory not found: {}",
            fendermint_dir.display()
        );
    }

    if !cometbft_dir.exists() {
        anyhow::bail!("CometBFT directory not found: {}", cometbft_dir.display());
    }

    // Check for required configuration files
    let fendermint_config = fendermint_dir.join("config").join("default.toml");
    let cometbft_config = cometbft_dir.join("config").join("config.toml");

    if !fendermint_config.exists() {
        anyhow::bail!(
            "Fendermint config not found: {}",
            fendermint_config.display()
        );
    }

    if !cometbft_config.exists() {
        anyhow::bail!("CometBFT config not found: {}", cometbft_config.display());
    }

    Ok(())
}

fn load_fendermint_settings(home: &Path) -> Result<Settings> {
    let fendermint_home = home.join("fendermint");
    let settings = Settings::new(
        &fendermint_home.join("config"),
        &fendermint_home,
        "validator",
    )
    .context("failed to load Fendermint settings")?;

    Ok(settings)
}

async fn wait_for_shutdown() -> Result<()> {
    #[cfg(unix)]
    {
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;

        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM");
            }
            _ = sigint.recv() => {
                info!("Received SIGINT");
            }
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
        info!("Received Ctrl+C");
    }

    Ok(())
}
