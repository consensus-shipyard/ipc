// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
// use crate::default_subscriber;
use crate::comet_runner::run_comet;
use crate::default_subscriber;
use crate::{CommandLineHandler, GlobalArguments};
use anyhow::Ok;
use async_trait::async_trait;
use clap::Args;
use fendermint_app_settings::Settings;
use fs_err as fs;
use std::path::Path;

pub(crate) struct InitNode;

#[async_trait]
impl CommandLineHandler for InitNode {
    type Arguments = InitNodeArgs;

    async fn handle(global: &GlobalArguments, _arguments: &Self::Arguments) -> anyhow::Result<()> {
        // TODO Karel - make a home folder a global argument instead
        let default_home = ipc_provider::default_repo_path();
        let home = Path::new(&default_home).join("node");

        create_dir(&home)?;

        let comet_bft_home = home.join("cometbft");
        create_dir(&comet_bft_home)?;
        init_comet_bft(&comet_bft_home).await?;

        let fendermint_home = home.join("fendermint");
        create_dir(&fendermint_home)?;
        init_fendermint(&fendermint_home)?;

        // TODO Karel - add the keys generation, config override etc...

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Arguments to initialize a new node")]
pub(crate) struct InitNodeArgs {}

fn create_dir(home: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(home).map_err(|e| {
        log::error!("Failed to create home directory {}: {}", home.display(), e);
        e
    })?;
    log::info!("Home directory created/exists: {}", home.display());
    Ok(())
}

async fn init_comet_bft(home: &Path) -> anyhow::Result<()> {
    let home = home.to_string_lossy();
    run_comet(&["init", "--home", &home])?;
    Ok(())
}

// TODO Karel - move this to fendermint as a command
fn init_fendermint(home: &Path) -> anyhow::Result<()> {
    let data_dir = home.join("data");
    let config_dir = home.join("config");
    create_dir(&data_dir)?;
    create_dir(&config_dir)?;

    // Create the default settings.
    let default_settings = Settings::default();
    log::info!("Default settings created.");

    // Serialize the settings into a pretty TOML string.
    let toml_string = toml::to_string_pretty(&default_settings).map_err(|e| {
        log::error!("Failed to serialize config to TOML: {}", e);
        e
    })?;
    log::info!("Configuration serialized to TOML.");

    // Determine the output file path.
    let path = config_dir.join("default.toml");
    // Write the TOML string to the file.
    fs::write(&path, toml_string).map_err(|e| {
        log::error!("Failed to write config to file {}: {}", path.display(), e);
        e
    })?;
    log::info!("Default configuration written to: {}", path.display());

    Ok(())
}
