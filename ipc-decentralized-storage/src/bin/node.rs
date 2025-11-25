// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Binary for running a decentralized storage node

use anyhow::{Context, Result};
use bls_signatures::{PrivateKey as BlsPrivateKey, Serialize as BlsSerialize};
use clap::{Parser, Subcommand};
use fendermint_actor_blobs_shared::method::Method;
use fendermint_actor_blobs_shared::operators::RegisterNodeOperatorParams;
use fendermint_actor_blobs_shared::BLOBS_ACTOR_ADDR;
use fendermint_rpc::FendermintClient;
use fendermint_vm_actor_interface::system;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use ipc_decentralized_storage::node::{launch, NodeConfig};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tendermint_rpc::Url;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "ipc-storage-node")]
#[command(about = "Decentralized storage node CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the storage node
    Run(RunArgs),
    /// Register as a node operator
    RegisterOperator(RegisterOperatorArgs),
}

#[derive(Parser, Debug)]
struct RunArgs {
    /// Path to file containing BLS private key in hex format (96 characters)
    /// If not provided, a new key will be generated and saved to this path
    #[arg(long, env = "BLS_KEY_FILE")]
    secret_key_file: Option<PathBuf>,

    /// Path to store Iroh data
    #[arg(long, default_value = "./iroh_data")]
    iroh_path: PathBuf,

    /// IPv4 bind address for Iroh (e.g., 0.0.0.0:11204)
    #[arg(long)]
    iroh_v4_addr: Option<SocketAddrV4>,

    /// IPv6 bind address for Iroh (e.g., [::]:11204)
    #[arg(long)]
    iroh_v6_addr: Option<SocketAddrV6>,

    /// Tendermint RPC URL
    #[arg(long, default_value = "http://localhost:26657")]
    rpc_url: String,

    /// Number of blobs to fetch per query
    #[arg(long, default_value = "10")]
    batch_size: u32,

    /// Polling interval in seconds
    #[arg(long, default_value = "5")]
    poll_interval_secs: u64,

    /// Maximum concurrent blob downloads
    #[arg(long, default_value = "10")]
    max_concurrent_downloads: usize,

    /// Address to bind the RPC server for signature queries
    #[arg(long, default_value = "127.0.0.1:8080")]
    rpc_bind_addr: SocketAddr,
}

#[derive(Parser, Debug)]
struct RegisterOperatorArgs {
    /// Path to file containing BLS private key in hex format (96 characters)
    #[arg(long, env = "BLS_KEY_FILE", required = true)]
    secret_key_file: PathBuf,

    /// RPC URL where this operator's node will be listening (e.g., http://my-node.example.com:8080)
    #[arg(long, required = true)]
    operator_rpc_url: String,

    /// Tendermint RPC URL for the chain
    #[arg(long, default_value = "http://localhost:26657")]
    chain_rpc_url: String,

    /// Operator's Ethereum address (if not provided, will use system actor)
    #[arg(long)]
    from_address: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => run_node(args).await,
        Commands::RegisterOperator(args) => register_operator(args).await,
    }
}

async fn run_node(args: RunArgs) -> Result<()> {
    // Parse or generate BLS private key
    let bls_private_key = if let Some(key_file) = &args.secret_key_file {
        if key_file.exists() {
            info!("Reading BLS private key from: {}", key_file.display());
            let key_hex = std::fs::read_to_string(key_file)
                .context("failed to read BLS private key file")?
                .trim()
                .to_string();

            let key_bytes = hex::decode(&key_hex)
                .context("failed to decode BLS private key hex string from file")?;

            BlsPrivateKey::from_bytes(&key_bytes)
                .map_err(|e| anyhow::anyhow!("failed to parse BLS private key: {:?}", e))?
        } else {
            info!("Key file not found, generating a new BLS private key");
            let key = BlsPrivateKey::generate(&mut rand::thread_rng());
            let key_hex = hex::encode(key.as_bytes());

            // Save the key to the file
            std::fs::write(key_file, &key_hex)
                .context("failed to write BLS private key to file")?;

            info!("Generated and saved new BLS private key to: {}", key_file.display());
            info!("Public key: {}", hex::encode(key.public_key().as_bytes()));

            key
        }
    } else {
        info!("No private key file provided, generating a new temporary key (will not be persisted)");
        let key = BlsPrivateKey::generate(&mut rand::thread_rng());
        info!("Generated temporary BLS private key");
        info!("Public key: {}", hex::encode(key.public_key().as_bytes()));
        info!("WARNING: This key will not be saved and will be lost when the node stops!");
        key
    };

    // Parse RPC URL
    let rpc_url = Url::from_str(&args.rpc_url)
        .context("failed to parse RPC URL")?;

    // Create node configuration
    let config = NodeConfig {
        iroh_path: args.iroh_path,
        iroh_v4_addr: args.iroh_v4_addr,
        iroh_v6_addr: args.iroh_v6_addr,
        rpc_url,
        batch_size: args.batch_size,
        poll_interval: Duration::from_secs(args.poll_interval_secs),
        max_concurrent_downloads: args.max_concurrent_downloads,
        bls_private_key,
        rpc_bind_addr: args.rpc_bind_addr,
    };

    info!("Starting node with configuration: {:?}", config);

    // Launch the node
    launch(config).await
}

async fn register_operator(args: RegisterOperatorArgs) -> Result<()> {
    info!("Registering as node operator");

    // Read BLS private key
    info!("Reading BLS private key from: {}", args.secret_key_file.display());
    let key_hex = std::fs::read_to_string(&args.secret_key_file)
        .context("failed to read BLS private key file")?
        .trim()
        .to_string();

    let key_bytes = hex::decode(&key_hex)
        .context("failed to decode BLS private key hex string from file")?;

    let bls_private_key = BlsPrivateKey::from_bytes(&key_bytes)
        .map_err(|e| anyhow::anyhow!("failed to parse BLS private key: {:?}", e))?;

    // Get BLS public key
    let bls_pubkey = bls_private_key.public_key().as_bytes().to_vec();

    info!("BLS public key: {}", hex::encode(&bls_pubkey));
    info!("Operator RPC URL: {}", args.operator_rpc_url);

    // Parse chain RPC URL
    let chain_rpc_url = Url::from_str(&args.chain_rpc_url)
        .context("failed to parse chain RPC URL")?;

    // Create Fendermint client
    let client = FendermintClient::new_http(chain_rpc_url, None)
        .context("failed to create Fendermint client")?;

    // Prepare registration parameters
    let params = RegisterNodeOperatorParams {
        bls_pubkey,
        rpc_url: args.operator_rpc_url.clone(),
    };

    let params_bytes = RawBytes::serialize(params)
        .context("failed to serialize RegisterNodeOperatorParams")?;

    // Determine the from address
    let from_address = if let Some(addr_str) = args.from_address {
        Address::from_str(&addr_str)
            .context("failed to parse from_address")?
    } else {
        system::SYSTEM_ACTOR_ADDR
    };

    // Create the message
    let msg = Message {
        version: Default::default(),
        from: from_address,
        to: BLOBS_ACTOR_ADDR,
        sequence: 0,
        value: TokenAmount::from_atto(0),
        method_num: Method::RegisterNodeOperator as u64,
        params: params_bytes,
        gas_limit: 10_000_000_000,
        gas_fee_cap: TokenAmount::from_atto(0),
        gas_premium: TokenAmount::from_atto(0),
    };

    info!("Sending RegisterNodeOperator transaction...");

    // Send the transaction
    let response = client
        .call(msg, FvmQueryHeight::default())
        .await
        .context("failed to send RegisterNodeOperator transaction")?;

    if response.value.code.is_err() {
        anyhow::bail!(
            "RegisterNodeOperator transaction failed: {}",
            response.value.info
        );
    }

    info!("✓ Successfully registered as node operator!");
    info!("  Public key: {}", hex::encode(bls_private_key.public_key().as_bytes()));
    info!("  RPC URL: {}", args.operator_rpc_url);

    Ok(())
}
