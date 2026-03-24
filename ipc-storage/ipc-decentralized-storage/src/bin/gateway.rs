// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! CLI for running the blob gateway with objects API

use anyhow::{Context, Result};
use bls_signatures::{PrivateKey as BlsPrivateKey, Serialize as BlsSerialize};
use clap::Parser;
use fendermint_rpc::message::SignedMessageFactory;
use fendermint_rpc::FendermintClient;
use fendermint_rpc::QueryClient;
use fendermint_vm_actor_interface::eam::EthAddress as FvmEthAddress;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_shared::address::{set_current_network, Address, Network};
use fvm_shared::chainid::ChainID;
use ipc_decentralized_storage::gateway::objects_service;
use ipc_decentralized_storage::gateway::BlobGateway;
use ipc_decentralized_storage::objects::ObjectsConfig;
use iroh_manager::IrohNode;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::PathBuf;
use std::time::Duration;
use tendermint_rpc::Url;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser, Debug)]
#[command(name = "gateway")]
#[command(
    about = "Run the blob gateway with objects API to query pending blobs and handle object uploads"
)]
struct Args {
    /// Set the FVM Address Network: "mainnet" (f) or "testnet" (t)
    #[arg(short, long, default_value = "testnet", env = "FM_NETWORK")]
    network: String,

    /// Path to file containing the secp256k1 secret key in Base64 format (for signing transactions)
    #[arg(long, env = "SECRET_KEY_FILE", required = true)]
    secret_key_file: PathBuf,

    /// Path to file containing BLS private key in hex format (96 characters)
    /// If not provided, a new key will be generated and saved to this path
    #[arg(long, env = "BLS_KEY_FILE")]
    bls_key_file: Option<PathBuf>,

    /// Tendermint RPC URL
    #[arg(short, long, default_value = "http://localhost:26657")]
    rpc_url: Url,

    /// Number of pending blobs to fetch per query
    #[arg(short, long, default_value = "10")]
    batch_size: u32,

    /// Polling interval in seconds
    #[arg(short = 'i', long, default_value = "5")]
    poll_interval_secs: u64,

    // Objects service arguments
    /// Enable objects HTTP API service
    #[arg(long, default_value = "true")]
    enable_objects: bool,

    /// Objects service listen address
    #[arg(long, default_value = "127.0.0.1:8080", env = "OBJECTS_LISTEN_ADDR")]
    objects_listen_addr: SocketAddr,

    /// Maximum object size in bytes (default 100MB)
    #[arg(long, default_value = "104857600", env = "MAX_OBJECT_SIZE")]
    max_object_size: u64,

    /// Path to Iroh data directory
    #[arg(long, env = "IROH_PATH")]
    iroh_path: PathBuf,

    /// Iroh IPv4 bind address
    #[arg(long, env = "IROH_V4_ADDR")]
    iroh_v4_addr: Option<SocketAddrV4>,

    /// Iroh IPv6 bind address
    #[arg(long, env = "IROH_V6_ADDR")]
    iroh_v6_addr: Option<SocketAddrV6>,
}

/// Get the next sequence number (nonce) of an account if it exists.
async fn get_sequence_opt(client: &impl QueryClient, addr: &Address) -> Result<Option<u64>> {
    let state = client
        .actor_state(addr, FvmQueryHeight::default())
        .await
        .context("failed to get actor state")?;

    match state.value {
        Some((_id, state)) => Ok(Some(state.sequence)),
        None => Ok(None),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    // Set the network for address display (f for mainnet, t for testnet)
    let network = match args.network.to_lowercase().as_str() {
        "main" | "mainnet" | "f" => Network::Mainnet,
        "test" | "testnet" | "t" => Network::Testnet,
        _ => {
            anyhow::bail!(
                "Invalid network: {}. Use 'mainnet' or 'testnet'",
                args.network
            );
        }
    };
    set_current_network(network);
    tracing::info!("Using network: {:?}", network);

    // Read secp256k1 secret key for signing transactions
    tracing::info!(
        "Reading secret key from: {}",
        args.secret_key_file.display()
    );
    let sk = SignedMessageFactory::read_secret_key(&args.secret_key_file)
        .context("failed to read secret key")?;

    let pk = sk.public_key();
    let from_f1 = Address::new_secp256k1(&pk.serialize()).context("failed to create f1 address")?;
    let from_eth = FvmEthAddress::new_secp256k1(&pk.serialize())
        .context("failed to derive delegated address from secret key")?;
    let from_f410 =
        Address::new_delegated(10, &from_eth.0).context("failed to create f410 address")?;
    tracing::info!("Gateway sender f1 address: {}", from_f1);
    tracing::info!("Gateway sender f410 address: {}", from_f410);

    // Parse or generate BLS private key if provided
    let _bls_private_key = if let Some(key_file) = &args.bls_key_file {
        if key_file.exists() {
            tracing::info!("Reading BLS private key from: {}", key_file.display());
            let key_hex = std::fs::read_to_string(key_file)
                .context("failed to read BLS private key file")?
                .trim()
                .to_string();

            let key_bytes = hex::decode(&key_hex)
                .context("failed to decode BLS private key hex string from file")?;

            let key = BlsPrivateKey::from_bytes(&key_bytes)
                .map_err(|e| anyhow::anyhow!("failed to parse BLS private key: {:?}", e))?;

            tracing::info!("Loaded BLS private key successfully");
            tracing::info!(
                "BLS Public key: {}",
                hex::encode(key.public_key().as_bytes())
            );
            Some(key)
        } else {
            tracing::info!("BLS key file not found, generating a new BLS private key");
            let key = BlsPrivateKey::generate(&mut rand::thread_rng());
            let key_hex = hex::encode(key.as_bytes());

            // Save the key to the file
            std::fs::write(key_file, &key_hex)
                .context("failed to write BLS private key to file")?;

            tracing::info!(
                "Generated and saved new BLS private key to: {}",
                key_file.display()
            );
            tracing::info!(
                "BLS Public key: {}",
                hex::encode(key.public_key().as_bytes())
            );
            Some(key)
        }
    } else {
        tracing::info!("No BLS private key file provided");
        None
    };

    tracing::info!("Starting blob gateway");
    tracing::info!("RPC URL: {}", args.rpc_url);
    tracing::info!("Batch size: {}", args.batch_size);
    tracing::info!("Poll interval: {}s", args.poll_interval_secs);

    // Start Iroh node for objects service
    tracing::info!("Starting Iroh node at: {}", args.iroh_path.display());
    let iroh_node = IrohNode::persistent(args.iroh_v4_addr, args.iroh_v6_addr, &args.iroh_path)
        .await
        .context("failed to start Iroh node")?;

    let node_addr = iroh_node.endpoint().node_addr().await?;
    tracing::info!("Iroh node started: {}", node_addr.node_id);

    // Start objects service if enabled (upload only)
    if args.enable_objects {
        let objects_config = ObjectsConfig {
            listen_addr: args.objects_listen_addr,
            tendermint_url: args.rpc_url.clone(),
            max_object_size: args.max_object_size,
            metrics_enabled: false,
            metrics_listen: None,
        };

        // Use the gateway's own Iroh blobs client for uploads
        let iroh_blobs = iroh_node.blobs_client().clone();

        let _objects_handle =
            objects_service::start_objects_service(objects_config, iroh_node.clone(), iroh_blobs);
        tracing::info!(
            "Objects upload service started on {}",
            args.objects_listen_addr
        );
    }

    // Create the Fendermint RPC client
    let client = FendermintClient::new_http(args.rpc_url, None)
        .context("failed to create Fendermint client")?;

    // Query the chain ID
    let chain_id = client
        .state_params(FvmQueryHeight::default())
        .await
        .context("failed to get state params")?
        .value
        .chain_id;

    tracing::info!("Chain ID: {}", chain_id);

    let (from_addr, sequence) = if let Some(sequence) = get_sequence_opt(&client, &from_f410)
        .await
        .context("failed to get delegated account sequence")?
    {
        tracing::info!("Using delegated sender (f410) for gateway transactions");
        (from_f410, sequence)
    } else {
        anyhow::bail!(
            "delegated sender {} not found on-chain; cross-fund this delegated address and retry (native f1 {} is intentionally not used)",
            from_f410, from_f1
        );
    };
    tracing::info!("Gateway sender address: {}", from_addr);
    tracing::info!("Account sequence: {}", sequence);

    // Create signed message factory
    let mf = SignedMessageFactory::new(sk, from_addr, sequence, ChainID::from(chain_id));

    // Bind the client with the message factory for transaction signing
    let bound_client = client.bind(mf);

    // Create the gateway with the bound client
    let mut gateway = BlobGateway::new(
        bound_client,
        args.batch_size,
        Duration::from_secs(args.poll_interval_secs),
    );

    // Run the gateway
    gateway.run().await?;

    Ok(())
}
