// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::cmd;
use crate::options::proof_cache::{ProofCacheArgs, ProofCacheCommands};
use fendermint_vm_topdown_proof_service::persistence::ProofCachePersistence;
use fendermint_vm_topdown_proof_service::{CacheConfig, ProofCache};
use std::path::{Path, PathBuf};

cmd! {
    ProofCacheArgs(self) {
        handle_proof_cache_command(self)
    }
}

fn handle_proof_cache_command(args: &ProofCacheArgs) -> anyhow::Result<()> {
    match &args.command {
        ProofCacheCommands::Inspect { db_path } => inspect_cache(db_path),
        ProofCacheCommands::Stats { db_path } => show_stats(db_path),
        ProofCacheCommands::Get {
            db_path,
            instance_id,
        } => get_proof(db_path, *instance_id),
    }
}

fn inspect_cache(db_path: &Path) -> anyhow::Result<()> {
    println!("=== Proof Cache Inspection ===");
    println!("Database: {}", db_path.display());
    println!();

    let persistence = ProofCachePersistence::open(db_path)?;

    let last_committed = persistence.load_last_committed()?;
    println!("Last Committed Instance: {:?}", last_committed);
    println!();

    let entries = persistence.load_all_entries()?;
    println!("Total Entries: {}", entries.len());

    if entries.is_empty() {
        println!("\nCache is empty.");
        return Ok(());
    }

    println!("\nEntries:");
    println!(
        "{:<12} {:<20} {:<15} {:<15}",
        "Instance ID", "Epochs", "Proof Size", "Signers"
    );
    println!("{}", "-".repeat(70));

    for entry in &entries {
        let proof_size = fvm_ipld_encoding::to_vec(&entry.proof_bundle)
            .map(|v| v.len())
            .unwrap_or(0);

        println!(
            "{:<12} {:<20?} {:<15} {:<15}",
            entry.instance_id,
            entry.finalized_epochs,
            format!("{} bytes", proof_size),
            format!("{} signers", entry.certificate.signers.len())
        );
    }

    Ok(())
}

fn show_stats(db_path: &Path) -> anyhow::Result<()> {
    println!("=== Proof Cache Statistics ===");
    println!("Database: {}", db_path.display());
    println!();

    let persistence = ProofCachePersistence::open(db_path)?;
    let last_committed = persistence.load_last_committed()?;
    let entries = persistence.load_all_entries()?;

    if entries.is_empty() {
        println!("Cache is empty.");
        return Ok(());
    }

    println!("Count: {}", entries.len());
    println!("Last Committed: {:?}", last_committed);
    println!(
        "Instances: {} - {}",
        entries.first().map(|e| e.instance_id).unwrap_or(0),
        entries.last().map(|e| e.instance_id).unwrap_or(0)
    );
    println!();

    // Proof size statistics
    let total_proof_size: usize = entries
        .iter()
        .map(|e| {
            fvm_ipld_encoding::to_vec(&e.proof_bundle)
                .map(|v| v.len())
                .unwrap_or(0)
        })
        .sum();
    let avg_proof_size = total_proof_size / entries.len();

    println!("Proof Bundle Statistics:");
    println!(
        "  Total Size: {} bytes ({:.2} MB)",
        total_proof_size,
        total_proof_size as f64 / 1024.0 / 1024.0
    );
    println!(
        "  Average Size: {} bytes ({:.2} KB)",
        avg_proof_size,
        avg_proof_size as f64 / 1024.0
    );

    Ok(())
}

fn get_proof(db_path: &Path, instance_id: u64) -> anyhow::Result<()> {
    println!("=== Get Proof for Instance {} ===", instance_id);
    println!("Database: {}", db_path.display());
    println!();

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
            println!("F3 Certificate:");
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
            let proof_bundle_size = fvm_ipld_encoding::to_vec(&entry.proof_bundle)
                .map(|v| v.len())
                .unwrap_or(0);
            println!("Proof Bundle:");
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

            // Metadata
            println!("Metadata:");
            println!("  Generated At: {:?}", entry.generated_at);
            println!("  Source RPC: {}", entry.source_rpc);
        }
        None => {
            println!("No proof found for instance {}", instance_id);
            println!();
            println!("Available instances: {:?}", cache.cached_instances());
        }
    }

    Ok(())
}
