// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Development/testing binary for the proof cache service
//!
//! NOTE: For production use, use `fendermint proof-cache` commands instead.
//! This binary is for development and CI testing only.

use clap::{Parser, Subcommand};
use fendermint_vm_topdown_proof_service::{launch_service, ProofServiceConfig};
use fvm_ipld_encoding;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about = "Proof cache service - DEVELOPMENT TOOL")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the proof generation service (development/testing)
    Run {
        /// Parent RPC URL
        #[arg(long)]
        rpc_url: String,

        /// Subnet ID
        #[arg(long)]
        subnet_id: String,

        /// Gateway address (Ethereum address like 0xE4c61299c16323C4B58376b60A77F68Aa59afC8b)
        #[arg(long)]
        gateway_address: String,

        /// Lookahead window
        #[arg(long, default_value = "3")]
        lookahead: u64,

        /// Initial F3 instance to start from
        #[arg(long)]
        initial_instance: u64,

        /// Poll interval in seconds
        #[arg(long, default_value = "10")]
        poll_interval: u64,

        /// Optional database path for persistence
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fendermint_vm_topdown_proof_service=debug".parse()?),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            rpc_url,
            subnet_id,
            gateway_address,
            lookahead,
            initial_instance,
            poll_interval,
            db_path,
        } => {
            run_service(
                rpc_url,
                subnet_id,
                gateway_address,
                lookahead,
                initial_instance,
                poll_interval,
                db_path,
            )
            .await
        }
    }
}

async fn run_service(
    rpc_url: String,
    subnet_id: String,
    gateway_address: String,
    lookahead: u64,
    initial_instance: u64,
    poll_interval: u64,
    db_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    println!("=== Proof Cache Service (DEVELOPMENT) ===");
    println!("Configuration:");
    println!("  RPC URL: {}", rpc_url);
    println!("  Subnet ID: {}", subnet_id);
    println!("  Gateway Address: {}", gateway_address);
    println!("  Lookahead: {} instances", lookahead);
    println!("  Initial Instance: {}", initial_instance);
    println!("  Poll Interval: {} seconds", poll_interval);
    if let Some(path) = &db_path {
        println!("  Database: {}", path.display());
    } else {
        println!("  Database: In-memory only");
    }
    println!();

    println!("Starting proof cache service...");
    println!();
    println!(
        "Fetching initial power table from F3 RPC (instance {})...",
        initial_instance
    );

    let temp_client = fendermint_vm_topdown_proof_service::f3_client::F3Client::new_from_rpc(
        &rpc_url,
        "calibrationnet",
        initial_instance,
    )
    .await?;

    // Get the power table
    let current_state = temp_client.get_state();
    let power_table = current_state.power_table;

    println!("Power table fetched: {} entries", power_table.0.len());
    println!(
        "F3 state initialized at instance {} (ready to validate {} onwards)",
        initial_instance, initial_instance
    );

    let config = ProofServiceConfig {
        enabled: true,
        parent_rpc_url: rpc_url,
        parent_subnet_id: "/r314159".to_string(),
        f3_network_name: "calibrationnet".to_string(),
        subnet_id: Some(subnet_id),
        gateway_actor_id: None,
        gateway_eth_address: Some(gateway_address),
        lookahead_instances: lookahead,
        polling_interval: Duration::from_secs(poll_interval),
        retention_instances: 2,
        max_cache_size_bytes: 0,
        fallback_rpc_urls: vec![],
        max_epoch_lag: 100,
        rpc_lookback_limit: 900,
    };

    let (cache, _handle) = launch_service(config, initial_instance, power_table, db_path).await?;
    println!("Service started successfully!");
    println!("Monitoring parent chain for F3 certificates...");
    println!();

    // Monitor cache status
    let mut last_size = 0;
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;

        let size = cache.len();
        let last_committed = cache.last_committed_instance();
        let highest = cache.highest_cached_instance();

        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        println!("=== Proof Cache Status ===");
        println!(
            "Timestamp: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        println!();
        println!("Cache Statistics:");
        println!("  Entries in cache: {}", size);
        println!("  Last committed instance: {}", last_committed);
        println!(
            "  Highest cached instance: {}",
            highest.map_or("None".to_string(), |h| h.to_string())
        );
        println!();

        if size > last_size {
            println!("New proofs generated: {}", size - last_size);
            last_size = size;
        }

        if let Some(entry) = cache.get_next_uncommitted() {
            println!("Next Uncommitted Proof:");
            println!("  Instance ID: {}", entry.instance_id);
            println!("  Finalized epochs: {:?}", entry.finalized_epochs);
            let proof_size = fvm_ipld_encoding::to_vec(&entry.proof_bundle)
                .map(|v| v.len())
                .unwrap_or(0);
            println!("  Proof bundle size: {} bytes", proof_size);
            println!("  Generated at: {:?}", entry.generated_at);
            println!();
        } else {
            println!("No uncommitted proofs available yet...");
            println!();
        }

        if size > 0 {
            println!("Cached Instances:");
            print!("  ");
            for instance in cache.cached_instances() {
                print!("{}  ", instance);
            }
            println!();
        }

        println!();
        println!("Press Ctrl+C to stop...");
    }
}
