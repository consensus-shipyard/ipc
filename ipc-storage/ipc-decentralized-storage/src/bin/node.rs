// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Binary for running a decentralized storage node

use anyhow::{anyhow, Context, Result};
use bls_signatures::{PrivateKey as BlsPrivateKey, Serialize as BlsSerialize};
use clap::{Parser, Subcommand};
use ethers::types::Address as EthAddress;
use fendermint_actor_blobs_shared::method::Method;
use fendermint_actor_blobs_shared::operators::RegisterNodeOperatorParams;
use fendermint_actor_blobs_shared::BLOBS_ACTOR_ADDR;
use fendermint_rpc::message::{GasParams, SignedMessageFactory};
use fendermint_rpc::tx::{TxClient, TxCommit};
use fendermint_rpc::FendermintClient;
use fendermint_rpc::QueryClient;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::{set_current_network, Address, Network};
use fvm_shared::chainid::ChainID;
use fvm_shared::econ::TokenAmount;
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
    /// Set the FVM Address Network: "mainnet" (f) or "testnet" (t)
    #[arg(short, long, default_value = "testnet", env = "FM_NETWORK")]
    network: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the storage node
    Run(RunArgs),
    /// Register as a node operator
    RegisterOperator(RegisterOperatorArgs),
    /// Generate a new BLS private key
    GenerateBlsKey(GenerateBlsKeyArgs),
    /// Query a blob by its hash
    QueryBlob(QueryBlobArgs),
    /// Query an object from a bucket by key
    QueryObject(QueryObjectArgs),
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

    /// Ethereum JSON-RPC URL (Fendermint ETH API endpoint)
    #[arg(long, default_value = "http://localhost:8545")]
    eth_rpc_url: String,

    /// Blobs actor address for event filtering (hex format with 0x prefix)
    #[arg(long, default_value = "0xff00000000000000000000000000000000000064")]
    blobs_actor_address: String,

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
    bls_key_file: PathBuf,

    /// Path to file containing the secp256k1 secret key in Base64 format (for signing transactions)
    #[arg(long, env = "SECRET_KEY_FILE", required = true)]
    secret_key_file: PathBuf,

    /// RPC URL where this operator's node will be listening (e.g., http://my-node.example.com:8080)
    #[arg(long, required = true)]
    operator_rpc_url: String,

    /// Tendermint RPC URL for the chain
    #[arg(long, default_value = "http://localhost:26657")]
    chain_rpc_url: String,
}

#[derive(Parser, Debug)]
struct GenerateBlsKeyArgs {
    /// Path to save the generated BLS private key (hex format)
    #[arg(long, short = 'o', default_value = "./bls_key.hex")]
    output: PathBuf,

    /// Overwrite existing file if it exists
    #[arg(long, short = 'f')]
    force: bool,
}

#[derive(Parser, Debug)]
struct QueryBlobArgs {
    /// Blob hash to query (hex string, with or without 0x prefix)
    #[arg(long, required = true)]
    hash: String,

    /// Tendermint RPC URL for the chain
    #[arg(long, default_value = "http://localhost:26657")]
    rpc_url: String,

    /// Block height to query at (default: latest committed)
    #[arg(long)]
    height: Option<u64>,
}

#[derive(Parser, Debug)]
struct QueryObjectArgs {
    /// Bucket address (f-address or eth-address format)
    #[arg(long, required = true)]
    bucket: String,

    /// Object key/path within the bucket
    #[arg(long, required = true)]
    key: String,

    /// Tendermint RPC URL for the chain
    #[arg(long, default_value = "http://localhost:26657")]
    rpc_url: String,

    /// Block height to query at (default: latest committed)
    #[arg(long)]
    height: Option<u64>,
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

    // Set the network for address display (f for mainnet, t for testnet)
    let network = match cli.network.to_lowercase().as_str() {
        "main" | "mainnet" | "f" => Network::Mainnet,
        "test" | "testnet" | "t" => Network::Testnet,
        _ => {
            anyhow::bail!(
                "Invalid network: {}. Use 'mainnet' or 'testnet'",
                cli.network
            );
        }
    };
    set_current_network(network);
    info!("Using network: {:?}", network);

    match cli.command {
        Commands::Run(args) => run_node(args).await,
        Commands::RegisterOperator(args) => register_operator(args).await,
        Commands::GenerateBlsKey(args) => generate_bls_key(args),
        Commands::QueryBlob(args) => query_blob(args).await,
        Commands::QueryObject(args) => query_object(args).await,
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

            info!(
                "Generated and saved new BLS private key to: {}",
                key_file.display()
            );
            info!("Public key: {}", hex::encode(key.public_key().as_bytes()));

            key
        }
    } else {
        info!(
            "No private key file provided, generating a new temporary key (will not be persisted)"
        );
        let key = BlsPrivateKey::generate(&mut rand::thread_rng());
        info!("Generated temporary BLS private key");
        info!("Public key: {}", hex::encode(key.public_key().as_bytes()));
        info!("WARNING: This key will not be saved and will be lost when the node stops!");
        key
    };

    // Parse RPC URL
    let rpc_url = Url::from_str(&args.rpc_url).context("failed to parse RPC URL")?;

    // Parse blobs actor address
    let blobs_actor_address: EthAddress = args
        .blobs_actor_address
        .parse()
        .context("failed to parse blobs actor address")?;

    // Create node configuration
    let config = NodeConfig {
        iroh_path: args.iroh_path,
        iroh_v4_addr: args.iroh_v4_addr,
        iroh_v6_addr: args.iroh_v6_addr,
        rpc_url,
        eth_rpc_url: args.eth_rpc_url,
        batch_size: args.batch_size,
        poll_interval: Duration::from_secs(args.poll_interval_secs),
        max_concurrent_downloads: args.max_concurrent_downloads,
        bls_private_key,
        rpc_bind_addr: args.rpc_bind_addr,
        blobs_actor_address,
    };

    info!("Starting node with configuration: {:?}", config);

    // Launch the node
    launch(config).await
}

async fn register_operator(args: RegisterOperatorArgs) -> Result<()> {
    info!("Registering as node operator");

    // Read BLS private key
    info!(
        "Reading BLS private key from: {}",
        args.bls_key_file.display()
    );
    let key_hex = std::fs::read_to_string(&args.bls_key_file)
        .context("failed to read BLS private key file")?
        .trim()
        .to_string();

    let key_bytes =
        hex::decode(&key_hex).context("failed to decode BLS private key hex string from file")?;

    let bls_private_key = BlsPrivateKey::from_bytes(&key_bytes)
        .map_err(|e| anyhow::anyhow!("failed to parse BLS private key: {:?}", e))?;

    // Get BLS public key
    let bls_pubkey = bls_private_key.public_key().as_bytes().to_vec();

    info!("BLS public key: {}", hex::encode(&bls_pubkey));
    info!("Operator RPC URL: {}", args.operator_rpc_url);

    // Read secp256k1 secret key for signing
    info!(
        "Reading secret key from: {}",
        args.secret_key_file.display()
    );
    let sk = SignedMessageFactory::read_secret_key(&args.secret_key_file)
        .context("failed to read secret key")?;

    let pk = sk.public_key();
    // Use f1 address (secp256k1) instead of f410 (delegated/ethereum) because we're calling
    // a native FVM actor with CBOR params, not an EVM contract with calldata
    let from_addr =
        Address::new_secp256k1(&pk.serialize()).context("failed to create f1 address")?;
    info!("Sender address: {}", from_addr);

    // Parse chain RPC URL
    let chain_rpc_url =
        Url::from_str(&args.chain_rpc_url).context("failed to parse chain RPC URL")?;

    // Create Fendermint client
    let client = FendermintClient::new_http(chain_rpc_url, None)
        .context("failed to create Fendermint client")?;

    // Query the account nonce from the state
    let sequence = get_sequence(&client, &from_addr)
        .await
        .context("failed to get account sequence")?;

    // Query the chain ID
    let chain_id = client
        .state_params(FvmQueryHeight::default())
        .await
        .context("failed to get state params")?
        .value
        .chain_id;

    info!("Chain ID: {}", chain_id);
    info!("Account sequence: {}", sequence);

    // Create signed message factory
    let mf = SignedMessageFactory::new(sk, from_addr, sequence, ChainID::from(chain_id));

    // Bind the client with the message factory
    let mut client = client.bind(mf);

    // Prepare registration parameters
    let params = RegisterNodeOperatorParams {
        bls_pubkey: bls_pubkey.clone(),
        rpc_url: args.operator_rpc_url.clone(),
    };

    let params_bytes =
        RawBytes::serialize(params).context("failed to serialize RegisterNodeOperatorParams")?;

    // Gas params
    let gas_params = GasParams {
        gas_limit: 10_000_000,
        gas_fee_cap: TokenAmount::from_atto(100),
        gas_premium: TokenAmount::from_atto(100),
    };

    info!("Sending RegisterNodeOperator transaction...");

    // Send the transaction
    let res = TxClient::<TxCommit>::transaction(
        &mut client,
        BLOBS_ACTOR_ADDR,
        Method::RegisterNodeOperator as u64,
        params_bytes,
        TokenAmount::from_atto(0),
        gas_params,
    )
    .await
    .context("failed to send RegisterNodeOperator transaction")?;

    if res.response.check_tx.code.is_err() {
        anyhow::bail!(
            "RegisterNodeOperator check_tx failed: {}",
            res.response.check_tx.log
        );
    }

    if res.response.deliver_tx.code.is_err() {
        anyhow::bail!(
            "RegisterNodeOperator deliver_tx failed: code={:?}, log={}, info={}, gas_used={}",
            res.response.deliver_tx.code,
            res.response.deliver_tx.log,
            res.response.deliver_tx.info,
            res.response.deliver_tx.gas_used
        );
    }

    info!("✓ Successfully registered as node operator!");
    info!(
        "  BLS Public key: {}",
        hex::encode(bls_private_key.public_key().as_bytes())
    );
    info!("  RPC URL: {}", args.operator_rpc_url);
    info!("  Tx hash: {}", res.response.hash);

    Ok(())
}

/// Get the next sequence number (nonce) of an account.
async fn get_sequence(client: &impl QueryClient, addr: &Address) -> Result<u64> {
    let state = client
        .actor_state(addr, FvmQueryHeight::default())
        .await
        .context("failed to get actor state")?;

    match state.value {
        Some((_id, state)) => Ok(state.sequence),
        None => Err(anyhow!("cannot find actor {addr}")),
    }
}

/// Generate a new BLS private key and save it to a file.
fn generate_bls_key(args: GenerateBlsKeyArgs) -> Result<()> {
    // Check if file already exists
    if args.output.exists() && !args.force {
        anyhow::bail!(
            "File {} already exists. Use --force to overwrite.",
            args.output.display()
        );
    }

    info!("Generating new BLS private key...");

    // Generate the key
    let key = BlsPrivateKey::generate(&mut rand::thread_rng());
    let key_hex = hex::encode(key.as_bytes());
    let pubkey_hex = hex::encode(key.public_key().as_bytes());

    // Save the key to the file
    std::fs::write(&args.output, &key_hex).context("failed to write BLS private key to file")?;

    info!("✓ BLS private key generated successfully!");
    info!("  Private key saved to: {}", args.output.display());
    info!("  Public key: {}", pubkey_hex);

    Ok(())
}

/// Query a blob by its hash from the blobs actor.
async fn query_blob(args: QueryBlobArgs) -> Result<()> {
    use fendermint_actor_blobs_shared::bytes::B256;
    use fendermint_rpc::message::GasParams;
    use fvm_shared::econ::TokenAmount;

    info!("Querying blob with hash: {}", args.hash);

    // Parse blob hash - strip 0x prefix if present
    let blob_hash_hex = args.hash.strip_prefix("0x").unwrap_or(&args.hash);

    let blob_hash_bytes =
        hex::decode(blob_hash_hex).context("failed to decode blob hash hex string")?;

    if blob_hash_bytes.len() != 32 {
        anyhow::bail!(
            "blob hash must be 32 bytes, got {} bytes",
            blob_hash_bytes.len()
        );
    }

    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&blob_hash_bytes);
    let blob_hash = B256(hash_array);

    // Parse RPC URL
    let rpc_url = Url::from_str(&args.rpc_url).context("failed to parse RPC URL")?;

    // Create Fendermint client
    let mut client =
        FendermintClient::new_http(rpc_url, None).context("failed to create Fendermint client")?;

    // Set query height
    let height = args
        .height
        .map(FvmQueryHeight::from)
        .unwrap_or(FvmQueryHeight::Committed);

    // Gas params for the query call
    let gas_params = GasParams {
        gas_limit: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    // Query the blob
    let maybe_blob = client
        .blob_get_call(blob_hash, TokenAmount::default(), gas_params, height)
        .await
        .context("failed to query blob")?;

    match maybe_blob {
        Some(blob) => {
            println!("Blob found!");
            println!("  Hash: 0x{}", hex::encode(blob_hash.0));
            println!("  Size: {} bytes", blob.size);
            println!("  Metadata hash: 0x{}", hex::encode(blob.metadata_hash.0));
            println!("  Status: {:?}", blob.status);
            println!("  Subscribers: {}", blob.subscribers.len());

            // Print subscriber details (subscription_id -> expiry epoch)
            for (subscription_id, expiry) in &blob.subscribers {
                println!("    - Subscription ID: {}", subscription_id);
                println!("      Expiry epoch: {}", expiry);
            }
        }
        None => {
            println!("Blob not found with hash: 0x{}", hex::encode(blob_hash.0));
        }
    }

    Ok(())
}

/// Query an object from a bucket by its key.
async fn query_object(args: QueryObjectArgs) -> Result<()> {
    use fendermint_actor_bucket::GetParams;
    use fendermint_rpc::message::GasParams;
    use fvm_shared::address::{Error as NetworkError, Network};
    use fvm_shared::econ::TokenAmount;
    use ipc_api::ethers_address_to_fil_address;

    info!(
        "Querying object from bucket: {} with key: {}",
        args.bucket, args.key
    );

    // Parse bucket address (supports both f-address and eth-address formats)
    let bucket_address = Network::Mainnet
        .parse_address(&args.bucket)
        .or_else(|e| match e {
            NetworkError::UnknownNetwork => Network::Testnet.parse_address(&args.bucket),
            _ => Err(e),
        })
        .or_else(|_| {
            let addr = ethers::types::Address::from_str(&args.bucket)
                .context("failed to parse as eth address")?;
            ethers_address_to_fil_address(&addr)
        })
        .context("failed to parse bucket address")?;

    info!("Parsed bucket address: {}", bucket_address);

    // Parse RPC URL
    let rpc_url = Url::from_str(&args.rpc_url).context("failed to parse RPC URL")?;

    // Create Fendermint client
    let mut client =
        FendermintClient::new_http(rpc_url, None).context("failed to create Fendermint client")?;

    // Set query height
    let height = args
        .height
        .map(FvmQueryHeight::from)
        .unwrap_or(FvmQueryHeight::Committed);

    // Gas params for the query call
    let gas_params = GasParams {
        gas_limit: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    // Query the object
    let params = GetParams(args.key.as_bytes().to_vec());
    let maybe_object = client
        .os_get_call(
            bucket_address,
            params,
            TokenAmount::default(),
            gas_params,
            height,
        )
        .await
        .context("failed to query object")?;

    match maybe_object {
        Some(object) => {
            println!("Object found!");
            println!("  Key: {}", args.key);
            println!("  Hash: 0x{}", hex::encode(object.hash.0));
            println!("  Recovery hash: 0x{}", hex::encode(object.recovery_hash.0));
            println!("  Size: {} bytes", object.size);
            println!("  Expiry epoch: {}", object.expiry);
            if !object.metadata.is_empty() {
                println!("  Metadata:");
                for (key, value) in &object.metadata {
                    println!("    {}: {}", key, value);
                }
            }
        }
        None => {
            println!("Object not found with key: {}", args.key);
        }
    }

    Ok(())
}
