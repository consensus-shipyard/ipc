// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::comet_runner::run_comet;
use crate::commands::node::config::{GenesisSource, NodeInitConfig};
use crate::commands::node::config_override::merge_toml_config;
use crate::commands::subnet::join::join_subnet;
use crate::{
    get_ipc_provider, ipc_config_store::IpcConfigStore, CommandLineHandler, GlobalArguments,
};
use anyhow::{Context, Ok};
use async_trait::async_trait;
use clap::Args;
use fendermint_app::cmd::config::write_default_settings as write_default_fendermint_setting;
use fendermint_app::cmd::genesis::into_tendermint;
use fendermint_app::options::genesis::GenesisIntoTendermintArgs;

use fendermint_app::cmd::key::store_key;
use fendermint_crypto::SecretKey;
use fs_err as fs;
use ipc_api::subnet_id::SubnetID;
use ipc_provider::IpcProvider;
use std::path::{Path, PathBuf};

use crate::commands::subnet::create_genesis::create_genesis;
use crate::commands::wallet::import::{import_wallet, WalletImportArgs};

pub(crate) struct InitNode;

#[async_trait]
impl CommandLineHandler for InitNode {
    type Arguments = InitNodeArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        let ipc_config_store = IpcConfigStore::load_or_init(global).await?;
        let config = NodeInitConfig::load(&arguments.config)?;

        let subnet_id: SubnetID = config.subnet.parse().context("invalid subnet ID")?;
        let parent_id: SubnetID = config.parent.parse().context("invalid parent ID")?;

        let home = Path::new(&config.home);
        create_dir(home)?;
        let fendermint_home = home.join("fendermint");
        create_dir(&fendermint_home)?;
        let comet_bft_home = home.join("cometbft");
        create_dir(&comet_bft_home)?;

        let provider = get_ipc_provider(global)?;
        log::info!("Importing and storing validator key");
        import_and_store_validator_key(&provider, &config.key, &fendermint_home)?;
        log::info!("Validator key imported and stored");

        if let Some(join_config) = &config.join {
            let mut provider = get_ipc_provider(global)?;
            log::info!("Joining subnet `{}` as `{}`", subnet_id, join_config.from);
            let args = join_config.clone().into_args(subnet_id.to_string());
            join_subnet(&mut provider, &args).await?;
            log::info!("Joined subnet `{}` as `{}`", subnet_id, join_config.from);
        };

        let created_genesis = match config.genesis {
            GenesisSource::Create(gen_cfg) => {
                let parent = ipc_config_store
                    .get_subnet(&parent_id)
                    .await
                    .context("parent subnet not found in config store")?;

                let created_genesis =
                    create_genesis(&gen_cfg, &subnet_id, &parent, &fendermint_home).await?;
                log::info!("Genesis created at: {:?}", created_genesis);
                created_genesis
            }
            GenesisSource::Path(created_genesis) => {
                log::info!("Using genesis at: {:?}", created_genesis);
                created_genesis
            }
        };

        log::info!("Initializing CometBFT with configuration overrides");
        init_comet_bft_with_overrides(&comet_bft_home, config.cometbft_overrides.as_ref()).await?;
        log::info!("CometBFT initialized successfully");

        log::info!("Initializing Fendermint with configuration overrides");
        init_fendermint_with_overrides(&fendermint_home, config.fendermint_overrides.as_ref())?;
        log::info!("Fendermint initialized successfully");

        log::info!("Converting genesis to Tendermint format");
        into_tendermint(
            &created_genesis.genesis,
            &GenesisIntoTendermintArgs {
                app_state: Some(created_genesis.sealed),
                out: comet_bft_home.join("config/genesis.json"),
                block_max_bytes: 22020096, // Default value from GenesisIntoTendermintArgs
            },
        )?;
        log::info!("Genesis converted to Tendermint format");

        log::info!("Node initialization completed successfully");
        Ok(())
    }
}

pub fn import_and_store_validator_key(
    provider: &IpcProvider,
    key_config: &WalletImportArgs,
    dir: &Path,
) -> anyhow::Result<()> {
    let imported_wallet = import_wallet(provider, key_config)?;

    // Convert to secp256k1 secret key (validators only support secp256k1)
    let secret_key = SecretKey::try_from(imported_wallet.private_key.clone())
        .context("Validator keys must be secp256k1. BLS keys are not supported.")?;

    store_key(&secret_key, "validator", dir).context("failed to store validator key")?;

    Ok(())
}

/// CLI arguments for initializing a new node via config file
#[derive(Debug, Args)]
#[command(
    name = "init-node",
    about = "Initialize a new CometBFT+FenderMint node from YAML spec"
)]
pub struct InitNodeArgs {
    /// Path to the node-init YAML configuration file
    #[arg(long, help = "Path to node init YAML config file")]
    pub config: PathBuf,
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
    run_comet(["init", "--home", &home])?;
    Ok(())
}

async fn init_comet_bft_with_overrides(
    home: &Path,
    overrides: Option<&toml::Value>,
) -> anyhow::Result<()> {
    log::info!("Initializing CometBFT");
    init_comet_bft(home).await?;

    if let Some(overrides) = overrides {
        let config_path = home.join("config/config.toml");
        log::info!("Applying CometBFT configuration overrides");
        merge_toml_config(&config_path, overrides)?;
        log::info!("CometBFT configuration overrides applied");
    } else {
        log::info!("No CometBFT overrides provided");
    }

    Ok(())
}

fn init_fendermint(home: &Path) -> anyhow::Result<()> {
    let data_dir = home.join("data");
    let config_dir = home.join("config");
    create_dir(&data_dir)?;
    create_dir(&config_dir)?;

    write_default_fendermint_setting(&config_dir)?;

    Ok(())
}

fn init_fendermint_with_overrides(
    home: &Path,
    overrides: Option<&toml::Value>,
) -> anyhow::Result<()> {
    log::info!("Initializing Fendermint");
    init_fendermint(home)?;

    if let Some(overrides) = overrides {
        let config_path = home.join("config/default.toml");
        log::info!("Applying Fendermint configuration overrides");
        merge_toml_config(&config_path, overrides)?;
        log::info!("Fendermint configuration overrides applied");
    } else {
        log::info!("No Fendermint overrides provided");
    }

    Ok(())
}
