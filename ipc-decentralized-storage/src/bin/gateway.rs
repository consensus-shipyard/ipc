// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! CLI for running the blob gateway

use anyhow::{Context, Result};
use bls_signatures::{PrivateKey as BlsPrivateKey, Serialize as BlsSerialize};
use clap::Parser;
use fendermint_rpc::FendermintClient;
use ipc_decentralized_storage::gateway::BlobGateway;
use std::path::PathBuf;
use std::time::Duration;
use tendermint_rpc::Url;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser, Debug)]
#[command(name = "gateway")]
#[command(about = "Run the blob gateway to query pending blobs from the FVM chain")]
struct Args {
    /// Path to file containing BLS private key in hex format (96 characters)
    /// If not provided, a new key will be generated and saved to this path
    #[arg(long, env = "BLS_KEY_FILE")]
    secret_key_file: Option<PathBuf>,

    /// Tendermint RPC URL
    #[arg(short, long, default_value = "http://localhost:26657")]
    rpc_url: Url,

    /// Number of pending blobs to fetch per query
    #[arg(short, long, default_value = "10")]
    batch_size: u32,

    /// Polling interval in seconds
    #[arg(short = 'i', long, default_value = "5")]
    poll_interval_secs: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    // Parse or generate BLS private key if provided
    let _bls_private_key = if let Some(key_file) = &args.secret_key_file {
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
            tracing::info!("Public key: {}", hex::encode(key.public_key().as_bytes()));
            Some(key)
        } else {
            tracing::info!("Key file not found, generating a new BLS private key");
            let key = BlsPrivateKey::generate(&mut rand::thread_rng());
            let key_hex = hex::encode(key.as_bytes());

            // Save the key to the file
            std::fs::write(key_file, &key_hex)
                .context("failed to write BLS private key to file")?;

            tracing::info!("Generated and saved new BLS private key to: {}", key_file.display());
            tracing::info!("Public key: {}", hex::encode(key.public_key().as_bytes()));
            Some(key)
        }
    } else {
        tracing::info!("No BLS private key file provided, running without key");
        None
    };

    tracing::info!("Starting blob gateway");
    tracing::info!("RPC URL: {}", args.rpc_url);
    tracing::info!("Batch size: {}", args.batch_size);
    tracing::info!("Poll interval: {}s", args.poll_interval_secs);

    // Create the Fendermint RPC client
    let client = FendermintClient::new_http(args.rpc_url, None)
        .context("failed to create Fendermint client")?;

    // Create the gateway
    let mut gateway = BlobGateway::new(
        client,
        args.batch_size,
        Duration::from_secs(args.poll_interval_secs),
    );

    // Run the gateway
    gateway.run().await?;

    Ok(())
}
