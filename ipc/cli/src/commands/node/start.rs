// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::services::node_manager::NodeManager;
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Args;
use fendermint_app_settings::Settings;
use ipc_observability::config::{
    ConsoleLayerSettings, FileLayerSettings, RotationKind, TracingSettings,
};
use ipc_observability::traces::set_global_tracing_subscriber;
use std::path::{Path, PathBuf};
use tracing::info;

pub(crate) struct StartNode;

#[async_trait]
impl crate::CommandLineHandler for StartNode {
    type Arguments = StartNodeArgs;

    async fn handle(_global: &crate::GlobalArguments, arguments: &Self::Arguments) -> Result<()> {
        let home = &arguments.home;
        validate_node_home(home)?;

        // Create logs directory
        let log_dir = home.join("logs");
        std::fs::create_dir_all(&log_dir)?;

        // Set up proper logging using traces.rs
        let tracing_config = TracingSettings {
            console: Some(ConsoleLayerSettings {
                level: Some("info".to_string()),
            }),
            file: Some(FileLayerSettings {
                enabled: true,
                directory: Some(log_dir.clone()),
                level: Some("info".to_string()),
                max_log_files: Some(5),
                rotation: Some(RotationKind::Daily),
                domain_filter: None,
                events_filter: None,
            }),
        };

        let _guards = set_global_tracing_subscriber(&tracing_config);

        info!("Starting IPC node from home directory: {}", home.display());
        info!("Logs will be written to: {}", log_dir.display());

        // Load Fendermint settings with correct home directory
        let settings = load_fendermint_settings(home)?;
        info!(
            "Fendermint settings loaded with home: {}",
            settings.home_dir().display()
        );

        // Create and start node manager with all services
        let node_manager = NodeManager::new(home.to_path_buf(), settings);
        node_manager.start_all_services().await?;

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
        &fendermint_home, // Use the fendermint home directory
        "validator",
    )
    .context("failed to load Fendermint settings")?;

    Ok(settings)
}
