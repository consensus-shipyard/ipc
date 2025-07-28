// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::comet_runner::run_comet;
use crate::commands::node::config::{
    CometBftOverrides, FendermintOverrides, GenesisSource, NodeInitConfig, P2pConfig,
};
use crate::commands::node::config_override::merge_toml_config;
use crate::commands::node::peer::{generate_peer_info, process_p2p_configuration, NodePaths};
use crate::commands::subnet::join::join_subnet;
use crate::{
    default_subscriber, get_ipc_provider, ipc_config_store::IpcConfigStore, CommandLineHandler,
    GlobalArguments,
};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use clap::Args;
use fendermint_app::cmd::config::write_default_settings as write_default_fendermint_setting;
use fendermint_app::cmd::genesis::into_tendermint;
use fendermint_app::options::genesis::GenesisIntoTendermintArgs;

use fendermint_app::cmd::key::{convert_key_to_cometbft, store_key};
use fendermint_crypto::SecretKey;
use ipc_api::subnet_id::SubnetID;
use ipc_provider::IpcProvider;
use std::path::Path;

use crate::commands::subnet::create_genesis::{create_genesis, CreatedGenesis};
use crate::commands::wallet::import::{import_wallet, WalletImportArgs};

pub(crate) struct InitNode;

#[async_trait]
impl CommandLineHandler for InitNode {
    type Arguments = InitNodeArgs;

    /// Initialize a new CometBFT+Fendermint node from configuration
    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> Result<()> {
        default_subscriber();

        let config = NodeInitConfig::load(&arguments.config)?;
        validate_config(&config)?;

        // Parse IDs early with good error messages
        let subnet_id = config.subnet.parse::<SubnetID>().with_context(|| {
            format!(
                "invalid subnet ID '{}' - expected format like '/r314/t410f...'",
                config.subnet
            )
        })?;

        let parent_id = config.parent.parse::<SubnetID>().with_context(|| {
            format!(
                "invalid parent subnet ID '{}' - expected format like '/r314'",
                config.parent
            )
        })?;

        // Parse overrides early
        let cometbft_overrides = config
            .cometbft_overrides
            .map(CometBftOverrides::from_toml_value)
            .transpose()
            .context("failed to parse CometBFT configuration overrides")?;

        let fendermint_overrides = config
            .fendermint_overrides
            .map(FendermintOverrides::from_toml_value)
            .transpose()
            .context("failed to parse Fendermint configuration overrides")?;

        let home_paths = ensure_directories(&config.home).await?;

        let provider = get_ipc_provider(global).context("failed to initialize IPC provider")?;

        let secret_key = import_and_store_key(&provider, &config.key)
            .await
            .context("failed to import validator key")?;

        if let Some(join_config) = &config.join {
            handle_subnet_join(global, &subnet_id, join_config).await?;
        }

        let genesis =
            setup_genesis(global, &config.genesis, &subnet_id, &parent_id, &home_paths).await?;

        init_cometbft(&home_paths.comet_bft, &secret_key, &cometbft_overrides).await?;

        init_fendermint(&home_paths.fendermint, &secret_key, &fendermint_overrides).await?;

        convert_genesis_to_tendermint(&genesis, &home_paths.comet_bft).await?;

        if let Some(p2p_config) = &config.p2p {
            process_p2p_configuration(&home_paths, p2p_config).await?;
        }

        generate_peer_info(&home_paths, &secret_key, config.p2p.as_ref()).await?;

        log::info!("Node initialization completed successfully");
        Ok(())
    }
}

/// Validate configuration invariants early
fn validate_config(config: &NodeInitConfig) -> Result<()> {
    log::debug!("Validating configuration");

    // Validate home directory
    if config.home.as_os_str().is_empty() {
        bail!("home directory cannot be empty");
    }

    if !config.home.is_absolute() {
        bail!(
            "home directory must be an absolute path, got: {}",
            config.home.display()
        );
    }

    // Validate subnet and parent IDs early
    config.subnet.parse::<SubnetID>().with_context(|| {
        format!(
            "invalid subnet ID '{}' - expected format like '/r314/t410f...'",
            config.subnet
        )
    })?;

    config.parent.parse::<SubnetID>().with_context(|| {
        format!(
            "invalid parent subnet ID '{}' - expected format like '/r314'",
            config.parent
        )
    })?;

    // Validate P2P configuration
    if let Some(p2p) = &config.p2p {
        validate_p2p_config(p2p)?;
    }

    log::debug!("Configuration validation completed");
    Ok(())
}

/// Validate P2P configuration specifics
fn validate_p2p_config(p2p: &P2pConfig) -> Result<()> {
    // Validate external IP if provided
    if let Some(external_ip) = &p2p.external_ip {
        if external_ip.trim().is_empty() {
            bail!("external IP cannot be empty when specified");
        }
        // Could add IP address validation here if needed
    }

    // Validate port ranges
    if let Some(ports) = &p2p.ports {
        if let Some(cometbft_port) = ports.cometbft {
            if cometbft_port < 1024 {
                log::warn!(
                    "CometBFT port {} is in privileged range (<1024), ensure proper permissions",
                    cometbft_port
                );
            }
        }
        if let Some(resolver_port) = ports.resolver {
            if resolver_port < 1024 {
                log::warn!(
                    "Resolver port {} is in privileged range (<1024), ensure proper permissions",
                    resolver_port
                );
            }
        }
    }

    // Validate peer files limit (moved MAX_PEER_FILES to peer module)
    if let Some(peers) = &p2p.peers {
        if let Some(peer_files) = &peers.peer_files {
            const MAX_PEER_FILES: usize = 100;
            if peer_files.len() > MAX_PEER_FILES {
                bail!(
                    "too many peer files specified: {} (maximum: {})",
                    peer_files.len(),
                    MAX_PEER_FILES
                );
            }
        }
    }

    Ok(())
}

/// Create all necessary directories
async fn ensure_directories(home: &Path) -> Result<NodePaths> {
    log::info!("Creating node directories under {}", home.display());

    let paths = NodePaths::new(home.to_path_buf());

    // Create all directories
    tokio::fs::create_dir_all(&paths.home)
        .await
        .with_context(|| {
            format!(
                "failed to create home directory at {}",
                paths.home.display()
            )
        })?;

    tokio::fs::create_dir_all(&paths.fendermint)
        .await
        .with_context(|| {
            format!(
                "failed to create Fendermint directory at {}",
                paths.fendermint.display()
            )
        })?;

    tokio::fs::create_dir_all(&paths.comet_bft)
        .await
        .with_context(|| {
            format!(
                "failed to create CometBFT directory at {}",
                paths.comet_bft.display()
            )
        })?;

    log::info!("Directory structure created successfully");
    Ok(paths)
}

/// Import and store validator key with proper error context
async fn import_and_store_key(
    provider: &IpcProvider,
    key_config: &WalletImportArgs,
) -> Result<SecretKey> {
    log::info!("Importing validator key");

    let imported_wallet = import_wallet(provider, key_config)
        .context("failed to import wallet - check key format and permissions")?;

    // Convert to secp256k1 secret key (validators only support secp256k1)
    let secret_key = SecretKey::try_from(imported_wallet.private_key.clone()).context(
        "validator keys must be secp256k1 format - BLS keys are not supported for validators",
    )?;

    log::info!("Validator key imported successfully");
    Ok(secret_key)
}

/// Handle subnet joining if configured
async fn handle_subnet_join(
    global: &GlobalArguments,
    subnet_id: &SubnetID,
    join_config: &crate::commands::subnet::init::config::JoinConfig,
) -> Result<()> {
    log::info!("Joining subnet {} as {}", subnet_id, join_config.from);

    let mut provider =
        get_ipc_provider(global).context("failed to get IPC provider for subnet join")?;

    let args = join_config.clone().into_args(subnet_id.to_string());
    join_subnet(&mut provider, &args).await.with_context(|| {
        format!(
            "failed to join subnet {} as {}",
            subnet_id, join_config.from
        )
    })?;

    log::info!("Successfully joined subnet {}", subnet_id);
    Ok(())
}

/// Setup genesis state (create or load existing)
async fn setup_genesis(
    global: &GlobalArguments,
    genesis_source: &GenesisSource,
    subnet_id: &SubnetID,
    parent_id: &SubnetID,
    paths: &NodePaths,
) -> Result<CreatedGenesis> {
    match genesis_source {
        GenesisSource::Create(gen_cfg) => {
            log::info!("Creating genesis from parent chain");

            let ipc_config_store = IpcConfigStore::load_or_init(global)
                .await
                .context("failed to load IPC configuration store")?;

            let parent = ipc_config_store
                .get_subnet(parent_id)
                .await
                .with_context(|| {
                    format!("parent subnet {} not found in config store", parent_id)
                })?;

            let genesis = create_genesis(gen_cfg, subnet_id, &parent, &paths.fendermint)
                .await
                .context("failed to create genesis from parent chain")?;

            log::info!("Genesis created successfully");
            Ok(genesis)
        }
        GenesisSource::Path(existing_genesis) => {
            log::info!("Using existing genesis from configuration");
            Ok(existing_genesis.clone())
        }
    }
}

/// Initialize CometBFT with configuration and key
async fn init_cometbft(
    home: &Path,
    secret_key: &SecretKey,
    overrides: &Option<CometBftOverrides>,
) -> Result<()> {
    log::info!("Initializing CometBFT");

    // Run CometBFT init command
    let home_str = home.to_string_lossy();
    // Initialize CometBFT with the validator key
    run_comet(["init", "--home", &home_str])
        .await
        .with_context(|| {
            format!(
                "failed to initialize CometBFT with validator key in {}",
                home.display()
            )
        })?;

    // Apply configuration overrides if provided
    if let Some(overrides) = overrides {
        log::info!("Applying CometBFT configuration overrides");
        let config_path = home.join("config/config.toml");
        let overrides_value = overrides.to_toml_value()?;
        merge_toml_config(&config_path, &overrides_value).with_context(|| {
            format!(
                "failed to apply CometBFT overrides to {}",
                config_path.display()
            )
        })?;
        log::info!("CometBFT configuration overrides applied");
    } else {
        log::debug!("No CometBFT overrides provided");
    }

    // Convert and store validator key
    let key_path = home.join("config/priv_validator_key.json");
    convert_key_to_cometbft(secret_key, &key_path).with_context(|| {
        format!(
            "failed to convert validator key for CometBFT at {}",
            key_path.display()
        )
    })?;

    log::info!("CometBFT initialized successfully");
    Ok(())
}

/// Initialize Fendermint with configuration and key
async fn init_fendermint(
    home: &Path,
    secret_key: &SecretKey,
    overrides: &Option<FendermintOverrides>,
) -> Result<()> {
    log::info!("Initializing Fendermint");

    // Create necessary directories
    let data_dir = home.join("data");
    let config_dir = home.join("config");

    tokio::fs::create_dir_all(&data_dir)
        .await
        .with_context(|| {
            format!(
                "failed to create Fendermint data directory at {}",
                data_dir.display()
            )
        })?;

    tokio::fs::create_dir_all(&config_dir)
        .await
        .with_context(|| {
            format!(
                "failed to create Fendermint config directory at {}",
                config_dir.display()
            )
        })?;

    // Write default settings
    write_default_fendermint_setting(&config_dir).with_context(|| {
        format!(
            "failed to write default Fendermint settings to {}",
            config_dir.display()
        )
    })?;

    // Apply configuration overrides if provided
    if let Some(overrides) = overrides {
        log::info!("Applying Fendermint configuration overrides");
        let config_path = config_dir.join("default.toml");
        let overrides_value = overrides.to_toml_value()?;
        merge_toml_config(&config_path, &overrides_value).with_context(|| {
            format!(
                "failed to apply Fendermint overrides to {}",
                config_path.display()
            )
        })?;
        log::info!("Fendermint configuration overrides applied");
    } else {
        log::debug!("No Fendermint overrides provided");
    }

    // Store validator key
    store_key(secret_key, "validator", home).with_context(|| {
        format!(
            "failed to store validator key in Fendermint home at {}",
            home.display()
        )
    })?;

    log::info!("Fendermint initialized successfully");
    Ok(())
}

/// Convert genesis to Tendermint format
async fn convert_genesis_to_tendermint(
    genesis: &CreatedGenesis,
    comet_bft_home: &Path,
) -> Result<()> {
    log::info!("Converting genesis to Tendermint format");

    let genesis_output_path = comet_bft_home.join("config/genesis.json");

    into_tendermint(
        &genesis.genesis,
        &GenesisIntoTendermintArgs {
            app_state: Some(genesis.sealed.clone()),
            out: genesis_output_path.clone(),
            block_max_bytes: 22020096, // Default value from GenesisIntoTendermintArgs
        },
    )
    .with_context(|| {
        format!(
            "failed to convert genesis to Tendermint format at {}",
            genesis_output_path.display()
        )
    })?;

    log::info!("Genesis converted to Tendermint format successfully");
    Ok(())
}

/// CLI arguments for initializing a new node via config file
#[derive(Debug, Args)]
#[command(
    name = "init",
    about = "Initialize a new CometBFT+Fendermint node from YAML configuration file",
    long_about = "Initialize a complete node setup including CometBFT consensus layer, \
                 Fendermint application layer, genesis state, validator keys, and P2P networking \
                 configuration. The node will be ready to join a subnet after initialization."
)]
pub struct InitNodeArgs {
    /// Path to the node initialization YAML configuration file
    ///
    /// The configuration file should specify the subnet to join, validator key,
    /// networking parameters, and genesis source. See documentation for full schema.
    #[arg(
        long = "config",
        short = 'c',
        help = "Path to node init YAML config file",
        long_help = "Path to the YAML configuration file containing node initialization parameters. \
                    The file must specify: home directory, subnet ID, parent subnet ID, validator key, \
                    and genesis source. Optional: P2P networking config and configuration overrides."
    )]
    pub config: std::path::PathBuf,
}
