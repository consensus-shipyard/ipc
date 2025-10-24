// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Test CLI for the proof cache service with multiple subcommands

use clap::{Parser, Subcommand};
use fendermint_vm_topdown_proof_service::{launch_service, ProofCache, ProofServiceConfig};
use fvm_ipld_encoding;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about = "Proof cache service test CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the proof generation service
    Run {
        /// Parent chain RPC URL
        #[arg(long)]
        rpc_url: String,

        /// Subnet ID (e.g., "mysubnet")
        #[arg(long)]
        subnet_id: String,

        /// Gateway actor ID on parent chain
        #[arg(long)]
        gateway_actor_id: u64,

        /// Number of instances to look ahead
        #[arg(long, default_value = "5")]
        lookahead: u64,

        /// Initial F3 instance ID to start from
        #[arg(long, default_value = "0")]
        initial_instance: u64,

        /// Polling interval in seconds
        #[arg(long, default_value = "10")]
        poll_interval: u64,

        /// Optional database path for persistence
        #[arg(long)]
        db_path: Option<PathBuf>,
    },

    /// Inspect cache contents
    Inspect {
        /// Database path
        #[arg(long)]
        db_path: PathBuf,
    },

    /// Show cache statistics
    Stats {
        /// Database path
        #[arg(long)]
        db_path: PathBuf,
    },

    /// Get specific proof by instance ID
    Get {
        /// Database path
        #[arg(long)]
        db_path: PathBuf,

        /// Instance ID to fetch
        #[arg(long)]
        instance_id: u64,
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
            gateway_actor_id,
            lookahead,
            initial_instance,
            poll_interval,
            db_path,
        } => {
            run_service(
                rpc_url,
                subnet_id,
                gateway_actor_id,
                lookahead,
                initial_instance,
                poll_interval,
                db_path,
            )
            .await
        }
        Commands::Inspect { db_path } => inspect_cache(&db_path),
        Commands::Stats { db_path } => show_stats(&db_path),
        Commands::Get {
            db_path,
            instance_id,
        } => get_proof(&db_path, instance_id),
    }
}

async fn run_service(
    rpc_url: String,
    subnet_id: String,
    gateway_actor_id: u64,
    lookahead: u64,
    initial_instance: u64,
    poll_interval: u64,
    db_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    println!("=== Proof Cache Service ===");
    println!("Configuration:");
    println!("  RPC URL: {}", rpc_url);
    println!("  Subnet ID: {}", subnet_id);
    println!("  Gateway Actor ID: {}", gateway_actor_id);
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
    // NOTE: The light client state is initialized at 'initial_instance' and ready to validate from there
    let current_state = temp_client.get_state().await;
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
        f3_network_name: "calibrationnet".to_string(), // TODO: make this a CLI argument
        subnet_id: Some(subnet_id),
        gateway_actor_id: Some(gateway_actor_id),
        lookahead_instances: lookahead,
        polling_interval: Duration::from_secs(poll_interval),
        retention_instances: 2,
        max_cache_size_bytes: 0,
        fallback_rpc_urls: vec![],
    };

    // Use the VALIDATED instance (742410), not the advanced state (742411)
    // The service will start generating proofs for 742411+
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

        // Clear screen for clean display
        print!("\x1B[2J\x1B[1;1H");

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
            let proof_bundle_size = fvm_ipld_encoding::to_vec(&entry.proof_bundle)
                .map(|v| v.len())
                .unwrap_or(0);
            println!("  Proof bundle size: {} bytes", proof_bundle_size);
            println!("  Generated at: {:?}", entry.generated_at);
        } else {
            println!("No uncommitted proofs available yet...");
        }
        println!();

        let instances = cache.cached_instances();
        if !instances.is_empty() {
            println!("Cached Instances:");
            for (i, instance_id) in instances.iter().enumerate() {
                if i > 0 && i % 10 == 0 {
                    println!();
                }
                print!("  {}", instance_id);
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
        let epochs_str = format!("[{:?}]", entry.finalized_epochs);
        let epochs_display = if epochs_str.len() > 18 {
            format!("{}...", &epochs_str[..15])
        } else {
            epochs_str
        };

        // Serialize proof bundle to get size
        let proof_bundle_size = fvm_ipld_encoding::to_vec(&entry.proof_bundle)
            .map(|v| v.len())
            .unwrap_or(0);

        println!(
            "{:<12} {:<20} {:<15} {:<15}",
            entry.instance_id,
            epochs_display,
            format!("{} bytes", proof_bundle_size),
            format!("{} signers", entry.certificate.signers.len())
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
        let min_instance = entries.iter().map(|e| e.instance_id).min().unwrap();
        let max_instance = entries.iter().map(|e| e.instance_id).max().unwrap();
        let total_proof_size: usize = entries
            .iter()
            .map(|e| {
                fvm_ipld_encoding::to_vec(&e.proof_bundle)
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
    use fendermint_vm_topdown_proof_service::config::CacheConfig;

    println!("=== Get Proof ===");
    println!("Database: {}", db_path.display());
    println!("Instance ID: {}", instance_id);
    println!();

    // Load cache with persistence
    let cache_config = CacheConfig {
        lookahead_instances: 10,
        retention_instances: 2,
        max_size_bytes: 0,
    };

    let cache = ProofCache::new_with_persistence(cache_config, db_path, 0)?;

    match cache.get(instance_id) {
        Some(entry) => {
            println!("Found proof for instance {}", instance_id);
            println!();

            // Certificate Details
            println!("═══ F3 Certificate ═══");
            println!("  Instance ID: {}", entry.certificate.instance_id);
            println!(
                "  Finalized Epochs: {:?}",
                entry.certificate.finalized_epochs
            );
            println!("  Power Table CID: {}", entry.certificate.power_table_cid);
            println!(
                "  BLS Signature: {} bytes",
                entry.certificate.signature.len()
            );
            println!("  Signers: {} validators", entry.certificate.signers.len());
            println!();

            // Proof Bundle Summary
            println!("═══ Proof Bundle Summary ═══");
            let proof_bundle_size = fvm_ipld_encoding::to_vec(&entry.proof_bundle)
                .map(|v| v.len())
                .unwrap_or(0);
            println!(
                "  Total Size: {} bytes ({:.2} KB)",
                proof_bundle_size,
                proof_bundle_size as f64 / 1024.0
            );
            println!(
                "  Storage Proofs: {}",
                entry.proof_bundle.storage_proofs.len()
            );
            println!("  Event Proofs: {}", entry.proof_bundle.event_proofs.len());
            println!("  Witness Blocks: {}", entry.proof_bundle.blocks.len());
            println!();

            // Proof Bundle Details - show structure
            println!("═══ Detailed Proof Structure ═══");
            println!(
                "Storage Proofs ({}):",
                entry.proof_bundle.storage_proofs.len()
            );
            for (i, sp) in entry.proof_bundle.storage_proofs.iter().enumerate() {
                println!("  [{}] {:?}", i, sp);
            }
            println!();

            println!("Event Proofs ({}):", entry.proof_bundle.event_proofs.len());
            for (i, ep) in entry.proof_bundle.event_proofs.iter().enumerate() {
                println!("  [{}] {:?}", i, ep);
            }
            println!();

            println!("Witness Blocks ({}):", entry.proof_bundle.blocks.len());
            println!("  (First and last blocks shown)");
            for (i, block) in entry.proof_bundle.blocks.iter().enumerate() {
                if i < 2 || i >= entry.proof_bundle.blocks.len() - 2 {
                    println!("  [{}] {:?}", i, block);
                } else if i == 2 {
                    println!(
                        "  ... ({} more blocks)",
                        entry.proof_bundle.blocks.len() - 4
                    );
                }
            }
            println!();

            // Metadata
            println!("═══ Metadata ═══");
            println!("  Generated At: {:?}", entry.generated_at);
            println!("  Source RPC: {}", entry.source_rpc);
            println!();

            // Full JSON dump
            println!("═══ Full Proof Bundle (JSON) ═══");
            if let Ok(json) = serde_json::to_string_pretty(&entry.proof_bundle) {
                println!("{}", json);
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
