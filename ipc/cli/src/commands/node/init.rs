// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::comet_runner::run_comet;
use crate::commands::node::config::{
    CometBftPeerInfo, FendermintPeerInfo, GenesisSource, NodeInfo, NodeInitConfig, P2pConfig,
    PeerInfo,
};
use crate::commands::node::config_override::merge_toml_config;
use crate::commands::subnet::join::join_subnet;
use crate::{
    default_subscriber, get_ipc_provider, ipc_config_store::IpcConfigStore, CommandLineHandler,
    GlobalArguments,
};
use anyhow::{Context, Ok};
use async_trait::async_trait;
use clap::Args;
use fendermint_app::cmd::config::write_default_settings as write_default_fendermint_setting;
use fendermint_app::cmd::genesis::into_tendermint;
use fendermint_app::options::genesis::GenesisIntoTendermintArgs;

use fendermint_app::cmd::key::{convert_key_to_cometbft, store_key};
use fendermint_crypto::SecretKey;
use fs_err as fs;
use ipc_api::subnet_id::SubnetID;
use ipc_provider::IpcProvider;
use std::path::{Path, PathBuf};

use crate::commands::subnet::create_genesis::create_genesis;
use crate::commands::wallet::import::{import_wallet, WalletImportArgs};

use fendermint_app::cmd::key::derive_peer_id_from_public_key;

pub(crate) struct InitNode;

#[async_trait]
impl CommandLineHandler for InitNode {
    type Arguments = InitNodeArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        default_subscriber();

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
        let secret_key = import_secret_key(&provider, &config.key)?;
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
        init_comet_bft_with_overrides(
            &comet_bft_home,
            &secret_key,
            config.cometbft_overrides.as_ref(),
        )
        .await?;
        log::info!("CometBFT initialized successfully");

        log::info!("Initializing Fendermint with configuration overrides");
        init_fendermint_with_overrides(
            &fendermint_home,
            &secret_key,
            config.fendermint_overrides.as_ref(),
        )?;
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

        // Process peer files and apply configurations
        if let Some(p2p_config) = &config.p2p {
            process_p2p_configuration(&comet_bft_home, &fendermint_home, p2p_config).await?;
        }

        // Generate peer information
        generate_peer_info(&home, &comet_bft_home, &secret_key, config.p2p.as_ref()).await?;

        log::info!("Node initialization completed successfully");
        Ok(())
    }
}

pub fn import_secret_key(
    provider: &IpcProvider,
    key_config: &WalletImportArgs,
) -> anyhow::Result<SecretKey> {
    let imported_wallet = import_wallet(provider, key_config)?;

    // Convert to secp256k1 secret key (validators only support secp256k1)
    let secret_key = SecretKey::try_from(imported_wallet.private_key.clone())
        .context("Validator keys must be secp256k1. BLS keys are not supported.")?;

    Ok(secret_key)
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
    secret_key: &SecretKey,
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

    convert_key_to_cometbft(secret_key, &home.join("config/priv_validator_key.json"))?;

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
    secret_key: &SecretKey,
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

    store_key(&secret_key, "validator", home).context("failed to store validator key")?;

    Ok(())
}

/// Generate and save peer information JSON file
async fn generate_peer_info(
    node_home: &Path,
    cometbft_home: &Path,
    secret_key: &SecretKey,
    p2p_config: Option<&P2pConfig>,
) -> anyhow::Result<()> {
    log::info!("Generating peer information");

    // Get configuration values with defaults
    let external_ip = p2p_config
        .and_then(|p2p| p2p.external_ip.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let cometbft_port = p2p_config
        .and_then(|p2p| p2p.ports.as_ref())
        .and_then(|ports| ports.cometbft)
        .unwrap_or(26656);

    let resolver_port = p2p_config
        .and_then(|p2p| p2p.ports.as_ref())
        .and_then(|ports| ports.resolver);

    // Get CometBFT node ID
    let node_id = get_cometbft_node_id(cometbft_home).await?;

    // Get Fendermint peer ID
    let peer_id = get_fendermint_peer_id(secret_key)?;

    // Create peer info structure
    let peer_info = PeerInfo {
        node_info: NodeInfo {
            external_ip: external_ip.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        },
        cometbft: CometBftPeerInfo {
            node_id: node_id.clone(),
            listen_port: cometbft_port,
            external_address: format!("{}:{}", external_ip, cometbft_port),
            peer_string: format!("{}@{}:{}", node_id, external_ip, cometbft_port),
        },
        fendermint: FendermintPeerInfo {
            peer_id: peer_id.clone(),
            listen_port: resolver_port,
            multiaddr: resolver_port
                .map(|port| format!("/ip4/{}/tcp/{}/p2p/{}", external_ip, port, peer_id)),
        },
    };

    // Save to JSON file
    let peer_info_path = node_home.join("peer-info.json");
    let json_content = serde_json::to_string_pretty(&peer_info)?;
    fs::write(&peer_info_path, json_content)?;

    log::info!("Peer information saved to: {}", peer_info_path.display());

    // Print peer information to console
    print_peer_info_to_console(&peer_info);

    Ok(())
}

/// Get CometBFT node ID using the cometbft command
async fn get_cometbft_node_id(cometbft_home: &Path) -> anyhow::Result<String> {
    let output = tokio::process::Command::new("cometbft")
        .args(&["show-node-id", "--home", &cometbft_home.to_string_lossy()])
        .output()
        .await
        .context("failed to run cometbft show-node-id command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cometbft show-node-id failed: {}", stderr);
    }

    let node_id = String::from_utf8(output.stdout)
        .context("failed to parse cometbft command output")?
        .trim()
        .to_string();

    Ok(node_id)
}

/// Get Fendermint libp2p peer ID using existing fendermint logic
fn get_fendermint_peer_id(secret_key: &SecretKey) -> anyhow::Result<String> {
    let public_key = secret_key.public_key();
    let peer_id = derive_peer_id_from_public_key(&public_key)?;
    Ok(peer_id)
}

/// Print peer information to console
fn print_peer_info_to_console(peer_info: &PeerInfo) {
    println!("\n🌐 Node Peer Information Generated:");
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│                    📡 CometBFT Peer                         │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!("│ Node ID:      {}         │", peer_info.cometbft.node_id);
    println!(
        "│ Peer String:  {}@{}:{}    │",
        peer_info.cometbft.node_id, peer_info.node_info.external_ip, peer_info.cometbft.listen_port
    );
    println!("├─────────────────────────────────────────────────────────────┤");
    println!("│                   🔗 Fendermint Resolver                    │");
    println!("├─────────────────────────────────────────────────────────────┤");
    if let Some(multiaddr) = &peer_info.fendermint.multiaddr {
        println!("│ Peer ID:      {}       │", peer_info.fendermint.peer_id);
        println!("│ Multiaddr:    {}    │", multiaddr);
    } else {
        println!("│ Status:       Resolver disabled                             │");
    }
    println!("└─────────────────────────────────────────────────────────────┘");
    println!("📁 Peer info saved to: peer-info.json");
    println!();
}

/// Process P2P configuration including reading peer files and applying configurations
async fn process_p2p_configuration(
    cometbft_home: &Path,
    fendermint_home: &Path,
    p2p_config: &P2pConfig,
) -> anyhow::Result<()> {
    log::info!("Processing P2P configuration");

    // Apply port configurations
    apply_port_configurations(cometbft_home, fendermint_home, p2p_config).await?;

    // Process peer files if specified
    if let Some(peers_config) = &p2p_config.peers {
        if let Some(peer_files) = &peers_config.peer_files {
            if !peer_files.is_empty() {
                log::info!("Processing {} peer file(s)", peer_files.len());
                let peer_infos = read_peer_files(peer_files).await?;
                apply_peer_configurations(cometbft_home, fendermint_home, &peer_infos).await?;
                log::info!(
                    "Applied peer configurations from {} file(s)",
                    peer_files.len()
                );
            }
        }
    }

    Ok(())
}

/// Apply port configurations to CometBFT and Fendermint configs
async fn apply_port_configurations(
    cometbft_home: &Path,
    fendermint_home: &Path,
    p2p_config: &P2pConfig,
) -> anyhow::Result<()> {
    let default_ports = Default::default();
    let ports = p2p_config.ports.as_ref().unwrap_or(&default_ports);

    // Apply CometBFT port configuration
    if let Some(cometbft_port) = ports.cometbft {
        log::info!("Setting CometBFT port to {}", cometbft_port);
        let config_path = cometbft_home.join("config/config.toml");

        // Create the port override
        let port_override = toml::Value::Table({
            let mut table = toml::map::Map::new();
            let mut p2p_table = toml::map::Map::new();
            p2p_table.insert(
                "laddr".to_string(),
                toml::Value::String(format!("tcp://0.0.0.0:{}", cometbft_port)),
            );
            table.insert("p2p".to_string(), toml::Value::Table(p2p_table));
            table
        });

        merge_toml_config(&config_path, &port_override)?;
    }

    // Apply Fendermint resolver port configuration
    if let Some(resolver_port) = ports.resolver {
        log::info!("Setting Fendermint resolver port to {}", resolver_port);
        let config_path = fendermint_home.join("config/default.toml");

        let external_ip = p2p_config.external_ip.as_deref().unwrap_or("127.0.0.1");
        let listen_addr = format!("/ip4/{}/tcp/{}", external_ip, resolver_port);

        // Create the resolver override
        let resolver_override = toml::Value::Table({
            let mut table = toml::map::Map::new();
            let mut resolver_table = toml::map::Map::new();
            let mut connection_table = toml::map::Map::new();
            connection_table.insert("listen_addr".to_string(), toml::Value::String(listen_addr));
            resolver_table.insert(
                "connection".to_string(),
                toml::Value::Table(connection_table),
            );
            table.insert("resolver".to_string(), toml::Value::Table(resolver_table));
            table
        });

        merge_toml_config(&config_path, &resolver_override)?;
    }

    Ok(())
}

/// Read and parse multiple peer info files
async fn read_peer_files(peer_files: &[String]) -> anyhow::Result<Vec<PeerInfo>> {
    let mut peer_infos = Vec::new();

    for peer_file in peer_files {
        log::info!("Reading peer file: {}", peer_file);

        let content = if peer_file.starts_with("http://") || peer_file.starts_with("https://") {
            // Handle remote URLs
            read_remote_peer_file(peer_file).await?
        } else {
            // Handle local files
            fs::read_to_string(peer_file)
                .with_context(|| format!("Failed to read peer file: {}", peer_file))?
        };

        let peer_info: PeerInfo = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse peer file: {}", peer_file))?;

        peer_infos.push(peer_info);
    }

    Ok(peer_infos)
}

/// Read a remote peer file via HTTP/HTTPS
async fn read_remote_peer_file(url: &str) -> anyhow::Result<String> {
    // For now, we'll use a simple approach - in production you might want to use reqwest or similar
    anyhow::bail!("Remote peer file URLs not yet implemented: {}", url);
}

/// Apply peer configurations to CometBFT and Fendermint config files
async fn apply_peer_configurations(
    cometbft_home: &Path,
    fendermint_home: &Path,
    peer_infos: &[PeerInfo],
) -> anyhow::Result<()> {
    // Build CometBFT persistent peers string
    let mut cometbft_peers = Vec::new();
    let mut fendermint_addresses = Vec::new();

    for peer_info in peer_infos {
        // Add CometBFT peer
        let peer_string = format!(
            "{}@{}:{}",
            peer_info.cometbft.node_id,
            peer_info.node_info.external_ip,
            peer_info.cometbft.listen_port
        );
        cometbft_peers.push(peer_string);

        // Add Fendermint resolver address if available
        if let Some(multiaddr) = &peer_info.fendermint.multiaddr {
            fendermint_addresses.push(multiaddr.clone());
        }
    }

    // Apply CometBFT peer configuration
    if !cometbft_peers.is_empty() {
        log::info!(
            "Setting CometBFT persistent peers: {}",
            cometbft_peers.join(", ")
        );
        let config_path = cometbft_home.join("config/config.toml");

        let peers_override = toml::Value::Table({
            let mut table = toml::map::Map::new();
            let mut p2p_table = toml::map::Map::new();
            p2p_table.insert(
                "persistent_peers".to_string(),
                toml::Value::String(cometbft_peers.join(",")),
            );
            table.insert("p2p".to_string(), toml::Value::Table(p2p_table));
            table
        });

        merge_toml_config(&config_path, &peers_override)?;
    }

    // Apply Fendermint resolver static addresses
    if !fendermint_addresses.is_empty() {
        log::info!(
            "Setting Fendermint static addresses: {}",
            fendermint_addresses.join(", ")
        );
        let config_path = fendermint_home.join("config/default.toml");

        let addresses_override = toml::Value::Table({
            let mut table = toml::map::Map::new();
            let mut resolver_table = toml::map::Map::new();
            let mut discovery_table = toml::map::Map::new();
            let addresses: Vec<toml::Value> = fendermint_addresses
                .iter()
                .map(|addr| toml::Value::String(addr.clone()))
                .collect();
            discovery_table.insert(
                "static_addresses".to_string(),
                toml::Value::Array(addresses),
            );
            resolver_table.insert("discovery".to_string(), toml::Value::Table(discovery_table));
            table.insert("resolver".to_string(), toml::Value::Table(resolver_table));
            table
        });

        merge_toml_config(&config_path, &addresses_override)?;
    }

    Ok(())
}
