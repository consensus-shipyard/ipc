// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::commands::node::config::{
    CometBftOverrides, CometBftPeerInfo, ConnectionOverrideConfig, DiscoveryOverrideConfig,
    FendermintOverrides, FendermintPeerInfo, NodeInfo, P2pCometConfig, P2pConfig, PeerInfo,
    ResolverOverrideConfig,
};
use crate::commands::node::config_override::merge_toml_config;
use anyhow::{bail, Context, Result};
use fendermint_app::cmd::key::derive_peer_id_from_public_key;
use fendermint_crypto::SecretKey;
use std::path::{Path, PathBuf};

/// Maximum allowed number of peer files to prevent abuse
const MAX_PEER_FILES: usize = 100;

/// Directory structure for node components
#[derive(Debug)]
pub struct NodePaths {
    pub home: PathBuf,
    pub fendermint: PathBuf,
    pub comet_bft: PathBuf,
}

impl NodePaths {
    pub fn new(home: PathBuf) -> Self {
        Self {
            fendermint: home.join("fendermint"),
            comet_bft: home.join("cometbft"),
            home,
        }
    }
}

/// Process P2P configuration including ports and peer files
pub async fn process_p2p_configuration(paths: &NodePaths, p2p_config: &P2pConfig) -> Result<()> {
    log::info!("Processing P2P configuration");

    apply_port_configurations(paths, p2p_config).await?;

    if let Some(peers_config) = &p2p_config.peers {
        if let Some(peer_files) = &peers_config.peer_files {
            if !peer_files.is_empty() {
                log::info!("Processing {} peer file(s)", peer_files.len());
                let peer_infos = read_peer_files(peer_files).await?;
                apply_peer_configurations(paths, &peer_infos).await?;
                log::info!(
                    "Applied peer configurations from {} file(s)",
                    peer_files.len()
                );
            } else {
                log::debug!("No peer files specified");
            }
        } else {
            log::debug!("No peer configuration provided");
        }
    }

    log::info!("P2P configuration processed successfully");
    Ok(())
}

/// Apply port configurations to CometBFT and Fendermint configs
async fn apply_port_configurations(paths: &NodePaths, p2p_config: &P2pConfig) -> Result<()> {
    let default_ports = Default::default();
    let ports = p2p_config.ports.as_ref().unwrap_or(&default_ports);

    // Apply CometBFT port configuration
    if let Some(cometbft_port) = ports.cometbft {
        log::info!("Configuring CometBFT P2P port: {}", cometbft_port);

        let comet_config = CometBftOverrides {
            p2p: Some(P2pCometConfig {
                laddr: Some(format!("tcp://0.0.0.0:{}", cometbft_port)),
                persistent_peers: None,
                extra: toml::Table::new(),
            }),
            consensus: None,
            rpc: None,
            extra: toml::Table::new(),
        };

        let config_path = paths.comet_bft.join("config/config.toml");
        let overrides_value = comet_config.to_toml_value()?;
        merge_toml_config(&config_path, &overrides_value).with_context(|| {
            format!(
                "failed to apply CometBFT port configuration to {}",
                config_path.display()
            )
        })?;
    }

    // Apply Fendermint resolver port configuration
    if let Some(resolver_port) = ports.resolver {
        log::info!("Configuring Fendermint resolver port: {}", resolver_port);

        let external_ip = p2p_config.external_ip.as_deref().unwrap_or("127.0.0.1");
        let listen_addr = format!("/ip4/{}/tcp/{}", external_ip, resolver_port);

        let fendermint_config = FendermintOverrides {
            resolver: Some(ResolverOverrideConfig {
                connection: Some(ConnectionOverrideConfig {
                    listen_addr: Some(listen_addr),
                    extra: toml::Table::new(),
                }),
                discovery: None,
                extra: toml::Table::new(),
            }),
            app: None,
            broadcast: None,
            extra: toml::Table::new(),
        };

        let config_path = paths.fendermint.join("config/default.toml");
        let overrides_value = fendermint_config.to_toml_value()?;
        merge_toml_config(&config_path, &overrides_value).with_context(|| {
            format!(
                "failed to apply Fendermint resolver configuration to {}",
                config_path.display()
            )
        })?;
    }

    Ok(())
}

/// Read and parse multiple peer info files
async fn read_peer_files(peer_files: &[String]) -> Result<Vec<PeerInfo>> {
    if peer_files.is_empty() {
        return Ok(Vec::new());
    }

    if peer_files.len() > MAX_PEER_FILES {
        bail!(
            "too many peer files specified: {} (maximum: {})",
            peer_files.len(),
            MAX_PEER_FILES
        );
    }

    let mut peer_infos = Vec::with_capacity(peer_files.len());

    for (index, peer_file) in peer_files.iter().enumerate() {
        log::debug!(
            "Reading peer file {} of {}: {}",
            index + 1,
            peer_files.len(),
            peer_file
        );

        let content = if peer_file.starts_with("http://") || peer_file.starts_with("https://") {
            read_remote_peer_file(peer_file)
                .await
                .with_context(|| format!("failed to fetch remote peer file: {}", peer_file))?
        } else {
            tokio::fs::read_to_string(peer_file)
                .await
                .with_context(|| format!("failed to read local peer file: {}", peer_file))?
        };

        let peer_info: PeerInfo = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse peer info JSON from file: {}", peer_file))?;

        peer_infos.push(peer_info);
    }

    log::info!("Successfully read {} peer file(s)", peer_infos.len());
    Ok(peer_infos)
}

/// Read a remote peer file via HTTP/HTTPS (currently stubbed)
async fn read_remote_peer_file(url: &str) -> Result<String> {
    // Provide clear error message about unsupported feature
    bail!(
        "Remote peer file URLs are not yet supported: {}\n\
        Please download the peer file locally and use a file path instead.\n\
        Example: wget {} -O peer-info.json",
        url,
        url
    );
}

/// Apply peer configurations to CometBFT and Fendermint config files
async fn apply_peer_configurations(paths: &NodePaths, peer_infos: &[PeerInfo]) -> Result<()> {
    if peer_infos.is_empty() {
        log::debug!("No peer configurations to apply");
        return Ok(());
    }

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
            "Configuring {} CometBFT persistent peer(s)",
            cometbft_peers.len()
        );

        let comet_config = CometBftOverrides {
            p2p: Some(P2pCometConfig {
                laddr: None,
                persistent_peers: Some(cometbft_peers.join(",")),
                extra: toml::Table::new(),
            }),
            consensus: None,
            rpc: None,
            extra: toml::Table::new(),
        };

        let config_path = paths.comet_bft.join("config/config.toml");
        let overrides_value = comet_config.to_toml_value()?;
        merge_toml_config(&config_path, &overrides_value).with_context(|| {
            format!(
                "failed to apply CometBFT peer configuration to {}",
                config_path.display()
            )
        })?;
    }

    // Apply Fendermint resolver static addresses
    if !fendermint_addresses.is_empty() {
        log::info!(
            "Configuring {} Fendermint static address(es)",
            fendermint_addresses.len()
        );

        let fendermint_config = FendermintOverrides {
            resolver: Some(ResolverOverrideConfig {
                connection: None,
                discovery: Some(DiscoveryOverrideConfig {
                    static_addresses: Some(fendermint_addresses),
                    extra: toml::Table::new(),
                }),
                extra: toml::Table::new(),
            }),
            app: None,
            broadcast: None,
            extra: toml::Table::new(),
        };

        let config_path = paths.fendermint.join("config/default.toml");
        let overrides_value = fendermint_config.to_toml_value()?;
        merge_toml_config(&config_path, &overrides_value).with_context(|| {
            format!(
                "failed to apply Fendermint peer configuration to {}",
                config_path.display()
            )
        })?;
    }

    Ok(())
}

/// Generate and save peer information JSON file
pub async fn generate_peer_info(
    paths: &NodePaths,
    secret_key: &SecretKey,
    p2p_config: Option<&P2pConfig>,
) -> Result<()> {
    log::info!("Generating peer information");

    // Get configuration values with defaults
    let external_ip = p2p_config
        .and_then(|p2p| p2p.external_ip.as_ref())
        .cloned()
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let cometbft_port = p2p_config
        .and_then(|p2p| p2p.ports.as_ref())
        .and_then(|ports| ports.cometbft)
        .unwrap_or(26656);

    let resolver_port = p2p_config
        .and_then(|p2p| p2p.ports.as_ref())
        .and_then(|ports| ports.resolver);

    // Get CometBFT node ID
    let node_id = get_cometbft_node_id(&paths.comet_bft).await?;

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
    let peer_info_path = paths.home.join("peer-info.json");
    let json_content = serde_json::to_string_pretty(&peer_info)
        .context("failed to serialize peer info to JSON")?;

    tokio::fs::write(&peer_info_path, json_content)
        .await
        .with_context(|| format!("writing peer-info to {}", peer_info_path.display()))?;

    log::info!("Peer information saved to: {}", peer_info_path.display());

    // Print peer information to console
    print_peer_info_to_console(&peer_info);

    Ok(())
}

/// Get CometBFT node ID using the cometbft command
async fn get_cometbft_node_id(cometbft_home: &Path) -> Result<String> {
    let output = tokio::process::Command::new("cometbft")
        .args(["show-node-id", "--home", &cometbft_home.to_string_lossy()])
        .output()
        .await
        .context("Couldn't run cometbft show-node-id—is CometBFT installed and in your PATH?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("CometBFT show-node-id command failed: {}", stderr);
    }

    let node_id = String::from_utf8(output.stdout)
        .context("CometBFT command returned invalid UTF-8 output")?
        .trim()
        .to_string();

    if node_id.is_empty() {
        bail!("CometBFT node ID is empty - this indicates a problem with CometBFT initialization");
    }

    Ok(node_id)
}

/// Get Fendermint libp2p peer ID using existing fendermint logic
fn get_fendermint_peer_id(secret_key: &SecretKey) -> Result<String> {
    let public_key = secret_key.public_key();
    let peer_id = derive_peer_id_from_public_key(&public_key)
        .context("failed to derive libp2p peer ID from validator public key")?;
    Ok(peer_id)
}

/// Print peer information to console with clear formatting
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
