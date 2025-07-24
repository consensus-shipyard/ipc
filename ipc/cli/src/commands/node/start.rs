// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::services::node_manager::NodeManager;
use crate::{CommandLineHandler, GlobalArguments};
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Args;
use fendermint_app::settings::Settings;
use std::path::{Path, PathBuf};
use tracing::info;

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
        &fendermint_home,
        "validator",
    )
    .context("failed to load Fendermint settings")?;

    Ok(settings)
}
