// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Cat command for displaying file contents from storage

use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::io::{self, Write};
use std::path::PathBuf;

use async_trait::async_trait;

use crate::commands::storage::{client::GatewayClient, config::StorageConfig, path};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
pub struct CatArgs {
    /// Storage path (ipc://bucket_address/path/to/file)
    #[arg(value_name = "PATH")]
    pub path: String,

    /// Gateway URL (overrides config and env var)
    #[arg(long)]
    pub gateway: Option<String>,

    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,
}

pub struct CatStorage;

#[async_trait]
impl CommandLineHandler for CatStorage {
    type Arguments = CatArgs;

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let storage_path = path::StoragePath::parse(&args.path)?;

        if storage_path.is_bucket_root() {
            return Err(anyhow!("Path must include a file key, not just bucket address"));
        }

        // Load config
        let config_path = args.config.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap()
                .join(".ipc")
                .join("storage.yaml")
        });

        let mut config = if config_path.exists() {
            StorageConfig::load(&config_path)?
        } else {
            return Err(anyhow!(
                "Storage config not found at {}. Run 'ipc-cli storage init' first.",
                config_path.display()
            ));
        };

        // Get gateway URL
        let gateway_url = config.get_gateway_url(
            args.gateway.as_deref(),
            Some(&config_path),
            false, // not interactive (avoid prompting when piping)
        )?;

        // Download from gateway
        let gateway_client = GatewayClient::new(gateway_url)?;
        let data = gateway_client
            .download_object(&storage_path.bucket_address, &storage_path.key, None)
            .await
            .context("Failed to download object from gateway")?;

        // Write to stdout
        io::stdout().write_all(&data)?;
        io::stdout().flush()?;

        Ok(())
    }
}
