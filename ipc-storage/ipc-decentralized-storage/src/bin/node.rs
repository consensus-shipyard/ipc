// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Binary for running a decentralized storage node

use anyhow::{Context, Result};
use bls_signatures::{PrivateKey as BlsPrivateKey, Serialize as BlsSerialize};
use bytes::Bytes;
use clap::{Parser, Subcommand};
use ethers::abi::{encode as abi_encode, Token as AbiToken};
use ethers::types::{Address as EthAddress, U256 as EthU256};
use fendermint_actor_blobs_shared::execution::{
    ExecutionJob, GetJobParams, JobStatus, ListJobsParams, ListJobsReturn, CLAIM_JOB_SELECTOR,
    COMPLETE_JOB_SELECTOR, FAIL_JOB_SELECTOR,
};
use fendermint_actor_blobs_shared::method::Method;
use fendermint_actor_blobs_shared::operators::RegisterNodeOperatorParams;
use fendermint_actor_blobs_shared::BLOBS_ACTOR_ADDR;
use fendermint_rpc::message::{GasParams, SignedMessageFactory};
use fendermint_rpc::tx::{BoundClient, TxClient, TxCommit};
use fendermint_rpc::FendermintClient;
use fendermint_rpc::QueryClient;
use fendermint_vm_actor_interface::eam::EthAddress as FvmEthAddress;
use fendermint_vm_actor_interface::system;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::{set_current_network, Address, Network};
use fvm_shared::bigint::Zero;
use fvm_shared::chainid::ChainID;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use ipc_decentralized_storage::node::{launch, NodeConfig};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tendermint_rpc::Url;
use tokio::process::Command as TokioCommand;
use tracing::info;

const REGISTER_NODE_OPERATOR_SELECTOR: [u8; 4] = [0x71, 0x3b, 0x10, 0xcf];

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
    /// Run execution worker loop over blobs actor jobs
    RunExecutor(RunExecutorArgs),
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

#[derive(Parser, Debug)]
struct RunExecutorArgs {
    /// Path to file containing the secp256k1 secret key in Base64 format
    #[arg(long, env = "SECRET_KEY_FILE", required = true)]
    secret_key_file: PathBuf,

    /// Tendermint RPC URL for the chain
    #[arg(long, default_value = "http://localhost:26657")]
    rpc_url: String,

    /// Polling interval in seconds
    #[arg(long, default_value = "5")]
    poll_interval_secs: u64,

    /// Gateway URL for downloading/uploading ipc:// storage objects
    #[arg(long, env = "IPC_STORAGE_GATEWAY")]
    gateway_url: Option<String>,
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
        Commands::RunExecutor(args) => run_executor(args).await,
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
    let from_f1 = Address::new_secp256k1(&pk.serialize()).context("failed to create f1 address")?;
    let from_eth = FvmEthAddress::new_secp256k1(&pk.serialize())
        .context("failed to derive delegated address from secret key")?;
    let from_f410 =
        Address::new_delegated(10, &from_eth.0).context("failed to create f410 address")?;
    info!("Sender f1 address: {}", from_f1);
    info!("Sender f410 address: {}", from_f410);

    // Parse chain RPC URL
    let chain_rpc_url =
        Url::from_str(&args.chain_rpc_url).context("failed to parse chain RPC URL")?;

    // Create Fendermint client
    let client = FendermintClient::new_http(chain_rpc_url, None)
        .context("failed to create Fendermint client")?;

    // Ensure blobs actor exists on this subnet.
    let blobs_actor_state = client
        .actor_state(&BLOBS_ACTOR_ADDR, FvmQueryHeight::default())
        .await
        .context("failed to query blobs actor state")?;
    if blobs_actor_state.value.is_none() {
        anyhow::bail!(
            "blobs actor {} is not deployed on this subnet. Recreate/start the subnet with ipc-storage enabled (fendermint_app built with --features ipc-storage), then retry register-operator.",
            BLOBS_ACTOR_ADDR
        );
    }

    // Query the chain ID
    let chain_id = client
        .state_params(FvmQueryHeight::default())
        .await
        .context("failed to get state params")?
        .value
        .chain_id;

    info!("Chain ID: {}", chain_id);

    // Prepare registration parameters
    let params = RegisterNodeOperatorParams {
        bls_pubkey: bls_pubkey.clone(),
        rpc_url: args.operator_rpc_url.clone(),
    };

    let gas_params = GasParams {
        gas_limit: 10_000_000,
        gas_fee_cap: TokenAmount::from_atto(1_000_000),
        gas_premium: TokenAmount::from_atto(100_000),
    };

    let tx_hash = if let Some(sequence) = get_sequence_opt(&client, &from_f410)
        .await
        .context("failed to get delegated account sequence")?
    {
        info!("Using delegated sender (f410) via InvokeContract facade");
        info!("Account sequence: {}", sequence);
        let mf = SignedMessageFactory::new(sk, from_f410, sequence, ChainID::from(chain_id));
        let mut client = client.bind(mf);
        let calldata = encode_register_node_operator_calldata(&params);
        let res = TxClient::<TxCommit>::fevm_invoke(
            &mut client,
            BLOBS_ACTOR_ADDR,
            calldata,
            TokenAmount::from_atto(0),
            gas_params,
        )
        .await
        .context("failed to send delegated RegisterNodeOperator transaction")?;
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
        info!("Sent RegisterNodeOperator transaction with delegated path");
        res.response.hash.to_string()
    } else {
        anyhow::bail!(
            "delegated sender {} not found on-chain; cross-fund this delegated address and retry (native f1 {} is intentionally not used)",
            from_f410, from_f1
        );
    };

    info!("✓ Successfully registered as node operator!");
    info!(
        "  BLS Public key: {}",
        hex::encode(bls_private_key.public_key().as_bytes())
    );
    info!("  RPC URL: {}", args.operator_rpc_url);
    info!("  Tx hash: {}", tx_hash);

    Ok(())
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

fn encode_register_node_operator_calldata(params: &RegisterNodeOperatorParams) -> Bytes {
    let args = abi_encode(&[
        AbiToken::Bytes(params.bls_pubkey.clone()),
        AbiToken::String(params.rpc_url.clone()),
    ]);
    let mut calldata = Vec::with_capacity(4 + args.len());
    calldata.extend_from_slice(&REGISTER_NODE_OPERATOR_SELECTOR);
    calldata.extend_from_slice(&args);
    Bytes::from(calldata)
}

fn encode_claim_job_calldata(id: u64) -> Bytes {
    let args = abi_encode(&[AbiToken::Uint(EthU256::from(id))]);
    let mut calldata = Vec::with_capacity(4 + args.len());
    calldata.extend_from_slice(&CLAIM_JOB_SELECTOR);
    calldata.extend_from_slice(&args);
    Bytes::from(calldata)
}

fn encode_complete_job_calldata(
    id: u64,
    output_refs: Vec<String>,
    output_commitment: [u8; 32],
    exit_code: i32,
) -> Bytes {
    let args = abi_encode(&[
        AbiToken::Uint(EthU256::from(id)),
        AbiToken::Array(output_refs.into_iter().map(AbiToken::String).collect()),
        AbiToken::FixedBytes(output_commitment.to_vec()),
        AbiToken::Int(abi_int256_from_i32(exit_code)),
    ]);
    let mut calldata = Vec::with_capacity(4 + args.len());
    calldata.extend_from_slice(&COMPLETE_JOB_SELECTOR);
    calldata.extend_from_slice(&args);
    Bytes::from(calldata)
}

fn encode_fail_job_calldata(id: u64, reason: String, exit_code: i32) -> Bytes {
    let args = abi_encode(&[
        AbiToken::Uint(EthU256::from(id)),
        AbiToken::String(reason),
        AbiToken::Int(abi_int256_from_i32(exit_code)),
    ]);
    let mut calldata = Vec::with_capacity(4 + args.len());
    calldata.extend_from_slice(&FAIL_JOB_SELECTOR);
    calldata.extend_from_slice(&args);
    Bytes::from(calldata)
}

fn abi_int256_from_i32(value: i32) -> EthU256 {
    if value >= 0 {
        EthU256::from(value as u32)
    } else {
        EthU256::MAX - EthU256::from((-value) as u32) + EthU256::from(1u8)
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

async fn run_executor(args: RunExecutorArgs) -> Result<()> {
    info!("Starting execution worker loop");

    let rpc_url = Url::from_str(&args.rpc_url).context("failed to parse RPC URL")?;
    let client = FendermintClient::new_http(rpc_url, None).context("failed to create client")?;

    let sk = SignedMessageFactory::read_secret_key(&args.secret_key_file)
        .context("failed to read secret key")?;
    let pk = sk.public_key();
    let from_eth = FvmEthAddress::new_secp256k1(&pk.serialize())
        .context("failed to derive delegated address from secret key")?;
    let from_f410 =
        Address::new_delegated(10, &from_eth.0).context("failed to create f410 address")?;

    let chain_id = client
        .state_params(FvmQueryHeight::default())
        .await
        .context("failed to get state params")?
        .value
        .chain_id;

    let sequence = get_sequence_opt(&client, &from_f410)
        .await
        .context("failed to get delegated account sequence")?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "delegated sender {} not found on-chain; cross-fund this delegated address and retry",
                from_f410
            )
        })?;

    info!("Executor sender: {}", from_f410);
    info!("Executor chain ID: {}", chain_id);
    info!("Executor sequence: {}", sequence);

    let mf = SignedMessageFactory::new(sk, from_f410, sequence, ChainID::from(chain_id));
    let mut tx_client = client.bind(mf);

    let poll_interval = Duration::from_secs(args.poll_interval_secs);

    loop {
        let processed = process_pending_jobs(
            &mut tx_client,
            &from_f410,
            args.gateway_url.as_deref(),
            &args.rpc_url,
        )
        .await;

        match processed {
            Ok(0) => {
                tokio::time::sleep(poll_interval).await;
            }
            Ok(n) => {
                info!("Processed {} job(s)", n);
            }
            Err(e) => {
                tracing::error!("Executor tick error: {:#}", e);
                if let Err(sync_err) = resync_sequence(&mut tx_client, &from_f410).await {
                    tracing::error!("Failed to resync sequence after error: {:#}", sync_err);
                }
                tokio::time::sleep(poll_interval).await;
            }
        }
    }
}

const MAX_TX_RETRIES: u32 = 3;

/// Re-query the on-chain nonce and update the local message factory.
async fn resync_sequence(
    tx_client: &mut (impl BoundClient + QueryClient),
    sender: &Address,
) -> Result<()> {
    let state = tx_client
        .actor_state(sender, FvmQueryHeight::default())
        .await
        .context("failed to query actor state for sequence resync")?;
    let sequence = state
        .value
        .map(|(_, s)| s.sequence)
        .ok_or_else(|| anyhow::anyhow!("sender {} not found during sequence resync", sender))?;
    tx_client.message_factory_mut().set_sequence(sequence);
    info!("Resynced sequence to {}", sequence);
    Ok(())
}

/// Send a transaction to the blobs actor with retry and automatic sequence resync.
///
/// Returns `Ok(true)` if the transaction was delivered successfully (deliver_tx ok),
/// `Ok(false)` if deliver_tx rejected it (nonce was consumed, move on),
/// `Err` only on unrecoverable failures.
async fn send_executor_tx(
    tx_client: &mut (impl BoundClient + QueryClient + TxClient<TxCommit>),
    sender: &Address,
    calldata: Bytes,
    label: &str,
) -> Result<bool> {
    let gas_params = GasParams {
        gas_limit: 10_000_000,
        gas_fee_cap: TokenAmount::from_atto(1_000_000),
        gas_premium: TokenAmount::from_atto(100_000),
    };

    for attempt in 0..MAX_TX_RETRIES {
        let res = TxClient::<TxCommit>::fevm_invoke(
            tx_client,
            BLOBS_ACTOR_ADDR,
            calldata.clone(),
            TokenAmount::zero(),
            gas_params.clone(),
        )
        .await;

        match res {
            Ok(commit_res) => {
                if commit_res.response.check_tx.code.is_err() {
                    // check_tx rejection: nonce NOT consumed on-chain but WAS incremented locally.
                    let log = &commit_res.response.check_tx.log;
                    tracing::warn!(
                        "{} check_tx rejected (attempt {}): code={:?} log={}",
                        label,
                        attempt + 1,
                        commit_res.response.check_tx.code,
                        if log.is_empty() { "<empty>" } else { log.as_str() },
                    );
                    resync_sequence(tx_client, sender).await?;
                    tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                    continue;
                }
                if commit_res.response.deliver_tx.code.is_err() {
                    // deliver_tx failure: nonce WAS consumed, sequence is correct. Not retryable.
                    let log = &commit_res.response.deliver_tx.log;
                    let info_str = &commit_res.response.deliver_tx.info;
                    tracing::warn!(
                        "{} deliver_tx failed: code={:?} log={} info={}",
                        label,
                        commit_res.response.deliver_tx.code,
                        if log.is_empty() { "<empty>" } else { log.as_str() },
                        if info_str.is_empty() { "<empty>" } else { info_str.as_str() },
                    );
                    return Ok(false);
                }
                return Ok(true);
            }
            Err(e) => {
                // Network/transport error: sequence state is unknown.
                tracing::warn!(
                    "{} network error (attempt {}): {:#}",
                    label,
                    attempt + 1,
                    e
                );
                resync_sequence(tx_client, sender).await?;
                tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
            }
        }
    }

    anyhow::bail!("{} failed after {} retries", label, MAX_TX_RETRIES)
}

async fn process_pending_jobs(
    tx_client: &mut (impl BoundClient + QueryClient + TxClient<TxCommit>),
    sender: &Address,
    gateway_url: Option<&str>,
    rpc_url: &str,
) -> Result<usize> {
    let pending_jobs = list_pending_jobs(tx_client).await?;
    if pending_jobs.is_empty() {
        return Ok(0);
    }

    let mut processed = 0;

    for job in &pending_jobs {
        info!(
            "Found candidate job {} binary_ref={} args={:?}",
            job.id, job.binary_ref, job.args
        );

        // Re-check the job is still pending (another executor may have claimed it).
        let latest = get_job(tx_client, job.id).await?;
        let Some(latest) = latest else {
            info!("Skipping job {}: no longer exists", job.id);
            continue;
        };
        if latest.status != JobStatus::Pending {
            info!(
                "Skipping job {}: status is {:?}",
                latest.id, latest.status
            );
            continue;
        }

        // --- Claim ---
        let claimed = send_executor_tx(
            tx_client,
            sender,
            encode_claim_job_calldata(job.id),
            &format!("ClaimJob({})", job.id),
        )
        .await?;

        if !claimed {
            info!("Job {} could not be claimed, skipping", job.id);
            continue;
        }
        info!("Claimed job {}", job.id);

        // --- Execute ---
        let run_result = execute_job(job, gateway_url, rpc_url).await;

        match run_result {
            Ok((exit_code, stdout, stderr)) => {
                // Always print job output.
                if !stdout.is_empty() {
                    info!("Job {} stdout:\n{}", job.id, stdout);
                }
                if !stderr.is_empty() {
                    info!("Job {} stderr:\n{}", job.id, stderr);
                }
                info!("Job {} exited with code {}", job.id, exit_code);

                if exit_code == 0 {
                    let combined = [stdout.as_bytes(), stderr.as_bytes()].concat();
                    let output_commitment =
                        fendermint_actor_blobs_shared::bytes::B256(*blake3::hash(&combined).as_bytes());
                    let output_refs =
                        vec![format!("inline://stdout/{}", hex::encode(output_commitment.0))];

                    let ok = send_executor_tx(
                        tx_client,
                        sender,
                        encode_complete_job_calldata(
                            job.id,
                            output_refs,
                            output_commitment.0,
                            exit_code,
                        ),
                        &format!("CompleteJob({})", job.id),
                    )
                    .await?;

                    if ok {
                        info!("Job {} completed successfully", job.id);
                    } else {
                        tracing::warn!("CompleteJob deliver_tx rejected for job {}", job.id);
                    }
                } else {
                    let reason =
                        format!("process exited with code {}: {}", exit_code, truncate(&stderr, 512));
                    let ok = send_executor_tx(
                        tx_client,
                        sender,
                        encode_fail_job_calldata(job.id, reason, exit_code),
                        &format!("FailJob({})", job.id),
                    )
                    .await?;

                    if ok {
                        info!("Job {} reported as failed (exit code {})", job.id, exit_code);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Job {} execution error: {:#}", job.id, e);
                let reason = format!("execution error: {}", truncate(&e.to_string(), 512));
                let _ = send_executor_tx(
                    tx_client,
                    sender,
                    encode_fail_job_calldata(job.id, reason, -1),
                    &format!("FailJob({})", job.id),
                )
                .await;
            }
        }

        processed += 1;
    }

    Ok(processed)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...(truncated)", &s[..max])
    }
}

/// Execute a job: download ipc:// inputs, run binary with env/timeout, upload outputs.
async fn execute_job(
    job: &ExecutionJob,
    gateway_url: Option<&str>,
    _rpc_url: &str,
) -> Result<(i32, String, String)> {
    let work_dir = tempfile::tempdir().context("failed to create temp working directory")?;
    let input_dir = work_dir.path().join("input");
    let output_dir = work_dir.path().join("output");
    std::fs::create_dir_all(&input_dir)?;
    std::fs::create_dir_all(&output_dir)?;

    let mut env_vars: Vec<(String, String)> = job.env.clone();

    // Download ipc:// input files.
    let http_client = reqwest::Client::new();
    for (i, input_ref) in job.input_refs.iter().enumerate() {
        if input_ref.starts_with("ipc://") {
            let gw = gateway_url.ok_or_else(|| {
                anyhow::anyhow!(
                    "Job {} has ipc:// input {} but no --gateway-url configured",
                    job.id,
                    input_ref
                )
            })?;

            let (bucket, key) = parse_ipc_uri(input_ref)?;
            let file_name = key.rsplit('/').next().unwrap_or(&key);
            let local_path = input_dir.join(file_name);

            info!("Downloading input {} -> {}", input_ref, local_path.display());
            let url = format!(
                "{}/v1/objects/{}/{}",
                gw.trim_end_matches('/'),
                bucket,
                urlencoding::encode(&key)
            );
            let resp = http_client
                .get(&url)
                .send()
                .await
                .with_context(|| format!("failed to download {}", input_ref))?;
            if !resp.status().is_success() {
                anyhow::bail!(
                    "Gateway returned {} downloading {}",
                    resp.status(),
                    input_ref
                );
            }
            let data = resp.bytes().await?;
            std::fs::write(&local_path, &data)?;

            env_vars.push((format!("IPC_INPUT_{}", i), local_path.to_string_lossy().to_string()));
        } else {
            env_vars.push((format!("IPC_INPUT_{}", i), input_ref.clone()));
        }
    }

    // Prepare output file paths for any IPC_OUTPUT_N env vars.
    let mut output_uploads: Vec<(String, PathBuf)> = Vec::new();
    for (key, value) in &env_vars {
        if key.starts_with("IPC_OUTPUT_") && key != "IPC_OUTPUT_DIR" && value.starts_with("ipc://") {
            let idx = key.trim_start_matches("IPC_OUTPUT_");
            let local_out = output_dir.join(format!("output_{}", idx));
            output_uploads.push((value.clone(), local_out.clone()));
        }
    }

    // Set IPC_OUTPUT_FILE_N vars pointing to writable local paths,
    // and IPC_OUTPUT_DIR for convenience.
    for (i, (_, local_path)) in output_uploads.iter().enumerate() {
        env_vars.push((
            format!("IPC_OUTPUT_FILE_{}", i),
            local_path.to_string_lossy().to_string(),
        ));
    }
    env_vars.push(("IPC_OUTPUT_DIR".to_string(), output_dir.to_string_lossy().to_string()));

    // Resolve binary.
    let binary = job
        .binary_ref
        .strip_prefix("local://")
        .unwrap_or(&job.binary_ref)
        .to_string();

    let timeout = if job.timeout_secs > 0 {
        Duration::from_secs(job.timeout_secs)
    } else {
        Duration::from_secs(300)
    };

    info!(
        "Executing: {} {:?} (timeout {}s)",
        binary,
        job.args,
        timeout.as_secs()
    );

    let child_fut = TokioCommand::new(&binary)
        .args(&job.args)
        .envs(env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output();

    let output = tokio::time::timeout(timeout, child_fut)
        .await
        .map_err(|_| anyhow::anyhow!("job timed out after {}s", timeout.as_secs()))?
        .context("failed to spawn/run process")?;

    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Upload output files to ipc:// destinations if job succeeded.
    if code == 0 && !output_uploads.is_empty() {
        if let Some(gw) = gateway_url {
            for (ipc_uri, local_path) in &output_uploads {
                if !local_path.exists() {
                    tracing::warn!(
                        "Output file {} not created by job, skipping upload to {}",
                        local_path.display(),
                        ipc_uri
                    );
                    continue;
                }
                info!("Uploading output {} -> {}", local_path.display(), ipc_uri);
                let data = std::fs::read(local_path)
                    .with_context(|| format!("failed to read output {}", local_path.display()))?;

                let form = reqwest::multipart::Form::new()
                    .text("size", data.len().to_string())
                    .part(
                        "data",
                        reqwest::multipart::Part::bytes(data)
                            .file_name("upload")
                            .mime_str("application/octet-stream")?,
                    );
                let url = format!("{}/v1/objects", gw.trim_end_matches('/'));
                let resp = http_client.post(&url).multipart(form).send().await?;
                if resp.status().is_success() {
                    info!("Uploaded output to gateway for {}", ipc_uri);
                } else {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    tracing::warn!(
                        "Failed to upload output to {}: {} {}",
                        ipc_uri,
                        status,
                        body
                    );
                }
            }
        } else {
            tracing::warn!(
                "Job produced output files but no --gateway-url configured; skipping upload"
            );
        }
    }

    Ok((code, stdout, stderr))
}

fn parse_ipc_uri(uri: &str) -> Result<(String, String)> {
    let path = uri
        .strip_prefix("ipc://")
        .ok_or_else(|| anyhow::anyhow!("not an ipc:// URI: {}", uri))?;
    let (bucket, key) = path.split_once('/').ok_or_else(|| {
        anyhow::anyhow!("ipc:// URI must include a key: {}", uri)
    })?;
    Ok((bucket.to_string(), key.to_string()))
}

async fn list_pending_jobs(client: &impl QueryClient) -> Result<Vec<ExecutionJob>> {
    let params = ListJobsParams {
        status: Some(JobStatus::Pending),
        limit: 50,
    };
    let params = RawBytes::serialize(params).context("failed to serialize ListJobs params")?;

    let msg = Message {
        version: Default::default(),
        from: system::SYSTEM_ACTOR_ADDR,
        to: BLOBS_ACTOR_ADDR,
        sequence: 0,
        value: TokenAmount::zero(),
        method_num: Method::ListJobs as u64,
        params,
        gas_limit: 10_000_000_000,
        gas_fee_cap: TokenAmount::zero(),
        gas_premium: TokenAmount::zero(),
    };

    let response = client
        .call(msg, FvmQueryHeight::default())
        .await
        .context("failed to execute ListJobs query")?;
    if response.value.code.is_err() {
        anyhow::bail!("ListJobs query failed: {}", response.value.info);
    }
    let return_data = fendermint_rpc::response::decode_data(&response.value.data)
        .context("failed to decode ListJobs response data")?;
    let jobs = fvm_ipld_encoding::from_slice::<ListJobsReturn>(&return_data)
        .context("failed to decode ListJobs return type")?;
    // Double-check client-side in case the actor ignores the status filter.
    Ok(jobs
        .jobs
        .into_iter()
        .filter(|j| j.status == JobStatus::Pending)
        .collect())
}

async fn get_job(client: &impl QueryClient, id: u64) -> Result<Option<ExecutionJob>> {
    let params =
        RawBytes::serialize(GetJobParams { id }).context("failed to serialize GetJob params")?;

    let msg = Message {
        version: Default::default(),
        from: system::SYSTEM_ACTOR_ADDR,
        to: BLOBS_ACTOR_ADDR,
        sequence: 0,
        value: TokenAmount::zero(),
        method_num: Method::GetJob as u64,
        params,
        gas_limit: 10_000_000_000,
        gas_fee_cap: TokenAmount::zero(),
        gas_premium: TokenAmount::zero(),
    };

    let response = client
        .call(msg, FvmQueryHeight::default())
        .await
        .context("failed to execute GetJob query")?;
    if response.value.code.is_err() {
        anyhow::bail!("GetJob query failed: {}", response.value.info);
    }
    let return_data = fendermint_rpc::response::decode_data(&response.value.data)
        .context("failed to decode GetJob response data")?;
    let job = fvm_ipld_encoding::from_slice::<Option<ExecutionJob>>(&return_data)
        .context("failed to decode GetJob return type")?;
    Ok(job)
}
