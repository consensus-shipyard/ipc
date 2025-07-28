// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::services::node_manager::NodeManager;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use clap::Args;
use fendermint_app_settings::Settings;
use ipc_observability::config::{
    ConsoleLayerSettings, FileLayerSettings, RotationKind, TracingSettings,
};
use ipc_observability::traces::{set_global_tracing_subscriber, WorkerGuard};
use std::path::{Path, PathBuf};
use tracing::info;

pub(crate) struct StartNode;

#[async_trait]
impl crate::CommandLineHandler for StartNode {
    type Arguments = StartNodeArgs;

    /// Start a complete IPC node with CometBFT consensus, Fendermint application layer, and ETH API
    async fn handle(_global: &crate::GlobalArguments, arguments: &Self::Arguments) -> Result<()> {
        // Step 1: Validate node home directory and required files
        validate_node_home(&arguments.home).await?;

        // Step 2: Prepare logging infrastructure
        let _guards = prepare_logging(&arguments.home).await?;

        info!(
            "Starting IPC node from home directory: {}",
            arguments.home.display()
        );

        // Step 3: Load Fendermint settings
        let settings = load_fendermint_settings(&arguments.home).await?;
        info!(
            "Fendermint settings loaded with home: {}",
            settings.home_dir().display()
        );

        // Step 4: Start all services
        start_services(&arguments.home, settings).await?;

        info!("Node startup completed successfully");
        Ok(())
    }
}

/// Initialize tracing infrastructure for the node
///
/// Sets up both console and file logging with info level.
/// Returns WorkerGuards that must be kept alive for the duration of the process.
async fn init_tracing(log_dir: &Path) -> Result<Vec<WorkerGuard>> {
    let tracing_config = TracingSettings {
        console: Some(ConsoleLayerSettings {
            level: Some("info".to_string()),
        }),
        file: Some(FileLayerSettings {
            enabled: true,
            directory: Some(log_dir.to_path_buf()),
            level: Some("info".to_string()),
            max_log_files: Some(5),
            rotation: Some(RotationKind::Daily),
            domain_filter: None,
            events_filter: None,
        }),
    };

    let guards = set_global_tracing_subscriber(&tracing_config);
    info!(
        "Tracing initialized | Logs directory: {}",
        log_dir.display()
    );

    Ok(guards)
}

/// Validate that the node home directory exists and contains all required components
///
/// Checks for:
/// - Home directory exists and is accessible
/// - Required subdirectories (fendermint, cometbft)
/// - Required configuration files
/// - Logs directory is writable
/// - Genesis files exist
async fn validate_node_home(home: &Path) -> Result<()> {
    info!("Validating node home directory structure");

    // Check home directory exists and is accessible
    if !home.exists() {
        bail!("Node home directory does not exist: {}", home.display());
    }

    if !home.is_dir() {
        bail!("Node home path is not a directory: {}", home.display());
    }

    // Check for required subdirectories
    let fendermint_dir = home.join("fendermint");
    let cometbft_dir = home.join("cometbft");

    if !fendermint_dir.exists() {
        bail!(
            "Fendermint directory not found - run 'ipc-cli node init' first: {}",
            fendermint_dir.display()
        );
    }

    if !cometbft_dir.exists() {
        bail!(
            "CometBFT directory not found - run 'ipc-cli node init' first: {}",
            cometbft_dir.display()
        );
    }

    // Validate required configuration files exist
    validate_required_config_files(&fendermint_dir, &cometbft_dir).await?;

    // Validate required data files exist
    validate_required_data_files(&fendermint_dir, &cometbft_dir).await?;

    // Test that we can write to logs directory
    validate_logs_directory_writable(home).await?;

    info!("Node home directory validation completed successfully");
    Ok(())
}

/// Validate that all required configuration files are present
async fn validate_required_config_files(fendermint_dir: &Path, cometbft_dir: &Path) -> Result<()> {
    let configs_to_check = [
        (
            fendermint_dir.join("config").join("default.toml"),
            "Fendermint configuration",
        ),
        (
            cometbft_dir.join("config").join("config.toml"),
            "CometBFT configuration",
        ),
        (
            cometbft_dir.join("config").join("genesis.json"),
            "CometBFT genesis",
        ),
        (
            cometbft_dir.join("config").join("priv_validator_key.json"),
            "CometBFT validator key",
        ),
    ];

    for (config_path, description) in configs_to_check {
        if !config_path.exists() {
            bail!(
                "{} not found: {} - run 'ipc-cli node init' first",
                description,
                config_path.display()
            );
        }

        // Verify file is readable
        tokio::fs::read_to_string(&config_path)
            .await
            .with_context(|| {
                format!(
                    "failed to read {} at {}",
                    description,
                    config_path.display()
                )
            })?;
    }

    Ok(())
}

/// Validate that required data files exist
async fn validate_required_data_files(fendermint_dir: &Path, cometbft_dir: &Path) -> Result<()> {
    let fendermint_data = fendermint_dir.join("data");
    let cometbft_data = cometbft_dir.join("data");

    // Check data directories exist (they should be created during init)
    if !fendermint_data.exists() {
        bail!(
            "Fendermint data directory not found: {} - run 'ipc-cli node init' first",
            fendermint_data.display()
        );
    }

    if !cometbft_data.exists() {
        bail!(
            "CometBFT data directory not found: {} - run 'ipc-cli node init' first",
            cometbft_data.display()
        );
    }

    // Verify validator key exists in Fendermint
    let validator_key_path = fendermint_dir.join("validator.sk");
    if !validator_key_path.exists() {
        bail!(
            "Validator private key not found: {} - run 'ipc-cli node init' first",
            validator_key_path.display()
        );
    }

    Ok(())
}

/// Validate that the logs directory is writable
async fn validate_logs_directory_writable(home: &Path) -> Result<()> {
    let logs_dir = home.join("logs");

    // Create logs directory if it doesn't exist
    tokio::fs::create_dir_all(&logs_dir)
        .await
        .with_context(|| format!("failed to create logs directory at {}", logs_dir.display()))?;

    // Test writability by creating and removing a test file
    let test_file = logs_dir.join(".write_test");
    tokio::fs::write(&test_file, "test")
        .await
        .with_context(|| format!("logs directory is not writable: {}", logs_dir.display()))?;

    tokio::fs::remove_file(&test_file).await.with_context(|| {
        format!(
            "failed to clean up test file in logs directory: {}",
            test_file.display()
        )
    })?;

    Ok(())
}

/// Prepare logging infrastructure for the node
///
/// Creates the logs directory and initializes tracing with both console and file output.
/// Returns WorkerGuards that must be kept alive for logging to work.
async fn prepare_logging(home: &Path) -> Result<Vec<WorkerGuard>> {
    let log_dir = home.join("logs");

    // Ensure logs directory exists and is writable (already validated but let's be extra safe)
    tokio::fs::create_dir_all(&log_dir)
        .await
        .with_context(|| format!("failed to create logs directory at {}", log_dir.display()))?;

    // Initialize tracing
    let guards = init_tracing(&log_dir)
        .await
        .context("failed to initialize tracing infrastructure")?;

    info!("Logging infrastructure prepared successfully");
    Ok(guards)
}

/// Load Fendermint settings from the node home directory
///
/// Reads the Fendermint configuration and validates that all required settings are present.
/// Returns the loaded Settings object ready for use by the node services.
async fn load_fendermint_settings(home: &Path) -> Result<Settings> {
    info!("Loading Fendermint settings");

    let fendermint_home = home.join("fendermint");
    let config_dir = fendermint_home.join("config");

    // Verify config directory exists
    if !config_dir.exists() {
        bail!(
            "Fendermint config directory not found: {} - run 'ipc-cli node init' first",
            config_dir.display()
        );
    }

    let settings =
        Settings::new(&config_dir, &fendermint_home, "validator").with_context(|| {
            format!(
                "failed to load Fendermint settings from {}",
                config_dir.display()
            )
        })?;

    info!("Fendermint settings loaded successfully");
    Ok(settings)
}

/// Start all node services and handle graceful shutdown
///
/// Creates the NodeManager and starts CometBFT, Fendermint, and ETH API services.
/// The NodeManager handles all shutdown signals internally.
async fn start_services(home: &Path, settings: Settings) -> Result<()> {
    info!("Starting node services");

    // Create and start node manager with all services
    let node_manager = NodeManager::new(home.to_path_buf(), settings);

    // Start all services - NodeManager handles shutdown signals internally
    node_manager.start_all_services().await
}

/// CLI arguments for starting a node
#[derive(Debug, Args)]
#[command(
    name = "start-node",
    about = "Start a complete IPC node with CometBFT consensus, Fendermint application layer, and ETH API",
    long_about = "Start a complete IPC node that includes:\n\
                 • CometBFT consensus layer for Byzantine fault-tolerant consensus\n\
                 • Fendermint application layer for FVM execution and IPC protocol\n\
                 • ETH API layer for Ethereum JSON-RPC compatibility\n\n\
                 The node must be initialized first using 'ipc-cli node init'."
)]
pub struct StartNodeArgs {
    /// Path to the node's home directory (must be initialized via 'ipc-cli node init')
    ///
    /// This directory should contain the complete node setup including:
    /// - fendermint/ subdirectory with configuration and data
    /// - cometbft/ subdirectory with configuration and data  
    /// - Required configuration files and validator keys
    #[arg(
        long,
        help = "Path to initialized node home directory",
        long_help = "Path to the node's home directory. This directory must have been \
                    initialized using 'ipc-cli node init' and should contain:\n\
                    • fendermint/config/default.toml - Fendermint configuration\n\
                    • cometbft/config/config.toml - CometBFT configuration\n\
                    • cometbft/config/genesis.json - Blockchain genesis\n\
                    • Validator private keys and other required files"
    )]
    pub home: PathBuf,
}
