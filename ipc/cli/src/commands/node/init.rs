// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
// use crate::default_subscriber;
use crate::comet_runner::run_comet;
use crate::{CommandLineHandler, GlobalArguments};
use anyhow::Ok;
use async_trait::async_trait;
use clap::Args;
use fendermint_app::cmd::config::write_default_settings as write_default_fendermint_setting;
use fendermint_app::cmd::key::{convert_key_to_cometbft, generate_key, key_from_eth};
use fendermint_app::options::key::{KeyFromEthArgs, KeyGenArgs, KeyIntoTendermintArgs};
use fs_err as fs;
use std::path::{Path, PathBuf};

pub(crate) struct InitNode;

#[async_trait]
impl CommandLineHandler for InitNode {
    type Arguments = InitNodeArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        let home = Path::new(&arguments.home);
        create_dir(&home)?;

        let fendermint_home = home.join("fendermint");
        create_dir(&fendermint_home)?;

        let key_name = "validator".into();
        if let Some(path) = &arguments.eth_key {
            log::info!("Using ETH key from {}", path.display());
            key_from_eth(&KeyFromEthArgs {
                secret_key: path.to_path_buf(),
                name: key_name,
                out_dir: fendermint_home.clone(),
            })?
        } else {
            log::info!("Generating ETH key");
            generate_key(&KeyGenArgs {
                name: key_name,
                out_dir: fendermint_home.clone(),
            })?
        };

        init_fendermint(&fendermint_home)?;

        let comet_bft_home = home.join("cometbft");
        create_dir(&comet_bft_home)?;
        init_comet_bft(&comet_bft_home).await?;

        // TODO Karel - add the keys generation, config override etc...

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Arguments to initialize a new node")]
pub(crate) struct InitNodeArgs {
    /// Path to the home folder.
    #[arg(short = 'd', long)]
    pub home: PathBuf,

    /// Path to an existing Ethereum key file. If omitted, a new key is generated.
    #[arg(short = 'k', long)]
    pub eth_key: Option<PathBuf>,
}

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

    write_default_fendermint_setting(&config_dir)?;

    Ok(())
}
