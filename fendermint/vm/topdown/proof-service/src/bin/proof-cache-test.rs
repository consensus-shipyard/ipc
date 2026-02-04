// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Development/testing binary for the proof cache service
//!
//! NOTE: For production use, use `fendermint proof-cache` commands instead.
//! This binary is for development and CI testing only.

use clap::{Parser, Subcommand};
use fendermint_vm_topdown_proof_service::config::{CacheConfig, GatewayId, ProofServiceConfig};
use fendermint_vm_topdown_proof_service::launch_service;
use fendermint_vm_topdown_proof_service::ProofCache;
use fvm_ipld_encoding;
use fvm_shared::clock::ChainEpoch;
use ipc_api::subnet_id::SubnetID;
use std::path::PathBuf;
use std::str::FromStr;
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

        /// Initial committed epoch
        #[arg(long)]
        initial_committed_epoch: u64,

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
            initial_committed_epoch,
            poll_interval,
            db_path,
        } => {
            run_service(
                rpc_url,
                subnet_id,
                gateway_address,
                lookahead,
                initial_committed_epoch,
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
    initial_committed_epoch: u64,
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
    let power_table = current_state.power_table.clone();

    println!("Power table fetched: {} entries", power_table.0.len());
    println!(
        "F3 state initialized at instance {} (ready to validate {} onwards)",
        initial_instance, initial_instance
    );

    let subnet_id_parsed = SubnetID::from_str(&subnet_id)?;

    let config = ProofServiceConfig {
        enabled: true,
        polling_interval: Duration::from_secs(poll_interval),
        cache_config: CacheConfig {
            lookahead_instances: lookahead,
            retention_epochs: 2,
        },
        parent_rpc_url: rpc_url,
        gateway_id: GatewayId::EthAddress(gateway_address),
    };

    let initial_committed_epoch = initial_instance as ChainEpoch;
    let (cache, _handle) = launch_service(
        config,
        subnet_id_parsed,
        initial_committed_epoch,
        initial_instance,
        power_table,
        0,
        0,
        db_path,
    )
    .await?
    .expect("Service should be enabled");
    println!("Service started successfully!");
    println!("Monitoring parent chain for F3 certificates...");
    println!();

    // Monitor cache status
    let mut last_size = 0;
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;

        let size = cache.epoch_proof_count();
        let highest = cache.highest_cached_instance();
        let instances = cache.cached_certificate_instances();

        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        println!("=== Proof Cache Status ===");
        println!(
            "Timestamp: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        println!();
        println!("Cache Statistics:");
        println!("  Entries in cache: {}", size);
        println!(
            "  Highest cached instance: {}",
            highest.map_or("None".to_string(), |h| h.to_string())
        );
        println!();

        if size > last_size {
            println!("✓ New proofs generated: {}", size - last_size);
            last_size = size;
        }

        if let Some(&latest_instance) = instances.last() {
            if let Some(cert_entry) = cache.get_certificate(latest_instance) {
                println!("Latest Cached Certificate:");
                println!("  Instance ID: {}", cert_entry.certificate.gpbft_instance);
                println!(
                    "  EC Chain tipsets: {}",
                    cert_entry.certificate.ec_chain.len()
                );
                println!("  Source RPC: {}", cert_entry.source_rpc);
                println!("  Fetched at: {:?}", cert_entry.fetched_at);
                println!();
            }
        } else {
            println!("No proofs cached yet...");
            println!();
        }

        if size > 0 {
            println!("Cached Instances:");
            print!("  ");
            for instance in instances {
                print!("{}  ", instance);
            }
            println!();
        }

        println!();
        println!("Press Ctrl+C to stop...");
    }
}
fn inspect_cache(db_path: &PathBuf) -> anyhow::Result<()> {
    use fendermint_vm_topdown_proof_service::persistence::ProofCachePersistence;

    println!("=== Cache Inspection ===");
    println!("Database: {}", db_path.display());
    println!();

    let persistence = ProofCachePersistence::open(db_path)?;

    // Load last committed
    let last_committed = persistence.load_last_committed()?;
    println!(
        "Last Committed Instance: {}",
        last_committed.map_or("None".to_string(), |i| i.to_string())
    );
    println!();

    // Load all entries
    let entries = persistence.load_all_entries()?;
    println!("Total Entries: {}", entries.len());
    println!();

    if entries.is_empty() {
        println!("Cache is empty.");
        return Ok(());
    }

    println!("Entries:");
    println!(
        "{:<12} {:<20} {:<15} {:<15}",
        "Instance ID", "Epochs", "Proof Size", "Signers"
    );
    println!("{}", "-".repeat(70));

    for entry in &entries {
        let epochs_str = format!("[{:?}]", entry.finalized_epochs());
        let epochs_display = if epochs_str.len() > 18 {
            format!("{}...", &epochs_str[..15])
        } else {
            epochs_str
        };

        // Serialize proof bundle to get size
        let proof_bundle_size = entry
            .proof_bundle
            .as_ref()
            .and_then(|bundle| fvm_ipld_encoding::to_vec(bundle).ok())
            .map(|v| v.len())
            .unwrap_or(0);

        // Format strings needed for table alignment
        #[allow(clippy::uninlined_format_args)]
        let proof_bundle_size_str = format!("{proof_bundle_size} bytes");
        #[allow(clippy::uninlined_format_args)]
        let signers_str = format!("{} signers", entry.certificate.signers.len());
        println!(
            "{:<12} {:<20} {:<15} {:<15}",
            entry.instance_id(),
            epochs_display,
            proof_bundle_size_str,
            signers_str
        );
    }

    Ok(())
}

fn show_stats(db_path: &PathBuf) -> anyhow::Result<()> {
    use fendermint_vm_topdown_proof_service::persistence::ProofCachePersistence;

    println!("=== Cache Statistics ===");
    println!("Database: {}", db_path.display());
    println!();

    let persistence = ProofCachePersistence::open(db_path)?;

    let last_committed = persistence.load_last_committed()?;
    let entries = persistence.load_all_entries()?;

    println!("General:");
    println!(
        "  Last Committed: {}",
        last_committed.map_or("None".to_string(), |i| i.to_string())
    );
    println!("  Total Entries: {}", entries.len());
    println!();

    if !entries.is_empty() {
        let min_instance = entries.iter().map(|e| e.instance_id()).min().unwrap();
        let max_instance = entries.iter().map(|e| e.instance_id()).max().unwrap();
        let total_proof_size: usize = entries
            .iter()
            .map(|e| {
                e.proof_bundle
                    .as_ref()
                    .and_then(|bundle| fvm_ipld_encoding::to_vec(bundle).ok())
                    .map(|v| v.len())
                    .unwrap_or(0)
            })
            .sum();
        let avg_proof_size = total_proof_size / entries.len();

        println!("Instances:");
        println!("  Min Instance ID: {}", min_instance);
        println!("  Max Instance ID: {}", max_instance);
        println!("  Range: {}", max_instance - min_instance + 1);
        println!();

        println!("Proof Bundles:");
        println!(
            "  Total Size: {} bytes ({:.2} KB)",
            total_proof_size,
            total_proof_size as f64 / 1024.0
        );
        println!("  Average Size: {} bytes", avg_proof_size);
        println!(
            "  Min Size: {} bytes",
            entries
                .iter()
                .map(|e| fvm_ipld_encoding::to_vec(&e.proof_bundle)
                    .map(|v| v.len())
                    .unwrap_or(0))
                .min()
                .unwrap()
        );
        println!(
            "  Max Size: {} bytes",
            entries
                .iter()
                .map(|e| fvm_ipld_encoding::to_vec(&e.proof_bundle)
                    .map(|v| v.len())
                    .unwrap_or(0))
                .max()
                .unwrap()
        );
        println!();

        println!("Epochs:");
        let total_epochs: usize = entries.iter().map(|e| e.finalized_epochs.len()).sum();
        println!("  Total Finalized Epochs: {}", total_epochs);
        println!(
            "  Avg Epochs per Instance: {:.1}",
            total_epochs as f64 / entries.len() as f64
        );
    }

    Ok(())
}

fn get_proof(db_path: &PathBuf, instance_id: u64) -> anyhow::Result<()> {
    use fendermint_vm_topdown_proof_service::persistence::ProofCachePersistence;

    println!("=== Get Proof ===");
    println!("Database: {}", db_path.display());
    println!("Instance ID: {}", instance_id);
    println!();

    // Load entries from persistence
    let persistence = ProofCachePersistence::open(db_path)?;
    let entries = persistence.load_all_entries()?;

    let entry = entries.iter().find(|e| e.instance_id() == instance_id);

    match entry {
        Some(entry) => {
            println!("Found proof for instance {}", instance_id);
            println!();
            println!("Details:");
            println!("  Instance ID: {}", entry.instance_id());
            println!("  Finalized Epochs: {:?}", entry.finalized_epochs());
            let proof_bundle_size = entry
                .proof_bundle
                .as_ref()
                .and_then(|bundle| fvm_ipld_encoding::to_vec(bundle).ok())
                .map(|v| v.len())
                .unwrap_or(0);
            println!("  Proof Bundle Size: {} bytes", proof_bundle_size);
            if let Some(ref proof_bundle) = entry.proof_bundle {
                println!(
                    "    - Storage Proofs: {}",
                    proof_bundle.storage_proofs.len()
                );
                println!("    - Event Proofs: {}", proof_bundle.event_proofs.len());
                println!("    - Witness Blocks: {}", proof_bundle.blocks.len());
            } else {
                println!("    - No proof bundle available");
            }
            println!("  Generated At: {:?}", entry.generated_at);
            println!("  Source RPC: {}", entry.source_rpc);
            println!();
            println!("Certificate:");
            println!("  Instance ID: {}", entry.certificate.gpbft_instance);
            println!(
                "  Finalized Epochs: {:?}",
                entry
                    .certificate
                    .ec_chain
                    .iter()
                    .map(|t| t.epoch)
                    .collect::<Vec<_>>()
            );
            println!(
                "  BLS Signature: {} bytes",
                entry.certificate.signature.len()
            );
            println!("  Signers: {} validators", entry.certificate.signers.len());
            println!();

            // Proof Bundle Summary
            if let Some(ref proof_bundle) = entry.proof_bundle {
                println!("═══ Proof Bundle Summary ═══");
                let proof_bundle_size = fvm_ipld_encoding::to_vec(proof_bundle)
                    .map(|v| v.len())
                    .unwrap_or(0);
                println!(
                    "  Total Size: {} bytes ({:.2} KB)",
                    proof_bundle_size,
                    proof_bundle_size as f64 / 1024.0
                );
                println!("  Storage Proofs: {}", proof_bundle.storage_proofs.len());
                println!("  Event Proofs: {}", proof_bundle.event_proofs.len());
                println!("  Witness Blocks: {}", proof_bundle.blocks.len());
                println!();

                // Proof Bundle Details - show structure
                println!("═══ Detailed Proof Structure ═══");
                println!("Storage Proofs ({}):", proof_bundle.storage_proofs.len());
                for (i, sp) in proof_bundle.storage_proofs.iter().enumerate() {
                    println!("  [{}] {:?}", i, sp);
                }
                println!();

                println!("Event Proofs ({}):", proof_bundle.event_proofs.len());
                for (i, ep) in proof_bundle.event_proofs.iter().enumerate() {
                    println!("  [{}] {:?}", i, ep);
                }
                println!();

                println!("Witness Blocks ({}):", proof_bundle.blocks.len());
                println!("  (First and last blocks shown)");
                for (i, block) in proof_bundle.blocks.iter().enumerate() {
                    if i < 2 || i >= proof_bundle.blocks.len() - 2 {
                        println!("  [{}] {:?}", i, block);
                    } else if i == 2 {
                        println!("  ... ({} more blocks)", proof_bundle.blocks.len() - 4);
                    }
                }
                println!();
            } else {
                println!("═══ Proof Bundle Summary ═══");
                println!("  No proof bundle available for this instance");
                println!();
            }

            // Metadata
            println!("═══ Metadata ═══");
            println!("  Generated At: {:?}", entry.generated_at);
            println!("  Source RPC: {}", entry.source_rpc);
            println!();

            // Full JSON dump
            if let Some(ref proof_bundle) = entry.proof_bundle {
                println!("═══ Full Proof Bundle (JSON) ═══");
                if let Ok(json) = serde_json::to_string_pretty(proof_bundle) {
                    println!("{}", json);
                }
            }
        }
        None => {
            println!("No proof found for instance {}", instance_id);
            println!();
            println!("Available instances: {:?}", cache.cached_instances());
        }
    }

    Ok(())
}
