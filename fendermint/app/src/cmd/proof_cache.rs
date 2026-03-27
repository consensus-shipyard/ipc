// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::cmd;
use crate::options::proof_cache::{ProofCacheArgs, ProofCacheCommands};
use fendermint_vm_topdown_proof_service::persistence::ProofCachePersistence;
use std::path::Path;
use std::path::PathBuf;

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
        ProofCacheCommands::Clear { db_path } => clear_cache(db_path),
    }
}

fn inspect_cache(db_path: &PathBuf) -> anyhow::Result<()> {
    println!("=== Proof Cache Inspection ===");
    println!("Database: {}", db_path.display());
    println!();

    let persistence = ProofCachePersistence::open(db_path)?;
    let entries = persistence.load_all_entries()?;

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
        let proof_size = entry
            .proof_bundle
            .as_ref()
            .and_then(|bundle| fvm_ipld_encoding::to_vec(bundle).ok())
            .map(|v| v.len())
            .unwrap_or(0);

        // Format strings needed for table alignment
        #[allow(clippy::uninlined_format_args)]
        let proof_size_str = format!("{proof_size} bytes");
        #[allow(clippy::uninlined_format_args)]
        let signers_str = format!("{} signers", entry.certificate.signers.len());
        println!(
            "{:<12} {:<20?} {:<15} {:<15}",
            entry.certificate.gpbft_instance,
            entry.certificate.ec_chain.suffix(),
            proof_size_str,
            signers_str
        );
    }

    Ok(())
}

fn show_stats(db_path: &PathBuf) -> anyhow::Result<()> {
    println!("=== Proof Cache Statistics ===");
    println!("Database: {}", db_path.display());
    println!();

    let persistence = ProofCachePersistence::open(db_path)?;
    let entries = persistence.load_all_entries()?;

    if entries.is_empty() {
        println!("Cache is empty.");
        return Ok(());
    }

    println!("Count: {}", entries.len());
    println!(
        "Instances: {} - {}",
        entries
            .first()
            .map(|e| e.certificate.gpbft_instance)
            .unwrap_or(0),
        entries
            .last()
            .map(|e| e.certificate.gpbft_instance)
            .unwrap_or(0)
    );
    println!();

    // Proof size statistics
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
    // Safe to divide: we already checked entries.is_empty() above and returned early
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

fn get_proof(db_path: &PathBuf, instance_id: u64) -> anyhow::Result<()> {
    println!("=== Get Proof for Instance {} ===", instance_id);
    println!("Database: {}", db_path.display());
    println!();

    let persistence = ProofCachePersistence::open(db_path)?;
    let entries = persistence.load_all_entries()?;

    if entries.is_empty() {
        println!("Cache is empty.");
        return Ok(());
    }

    let entry = entries
        .iter()
        .find(|e| e.certificate.gpbft_instance == instance_id);

    if let Some(entry) = entry {
        println!("Found proof for instance {}", instance_id);
        println!();

        // Certificate Details
        println!("F3 Certificate:");
        println!("  Instance ID: {}", entry.certificate.gpbft_instance);
        println!(
            "  Finalized Epochs: {:?}",
            entry.certificate.ec_chain.suffix()
        );
        println!(
            "  BLS Signature: {} bytes",
            entry.certificate.signature.len()
        );
        println!("  Signers: {} validators", entry.certificate.signers.len());
        println!();

        // Proof Bundle Summary
        let proof_bundle_size = entry
            .proof_bundle
            .as_ref()
            .and_then(|bundle| fvm_ipld_encoding::to_vec(bundle).ok())
            .map(|v| v.len())
            .unwrap_or(0);
        println!("Proof Bundle:");
        println!(
            "  Total Size: {} bytes ({:.2} KB)",
            proof_bundle_size,
            proof_bundle_size as f64 / 1024.0
        );

        if let Some(proof_bundle) = &entry.proof_bundle {
            println!("  Storage Proofs: {}", proof_bundle.storage_proofs.len());
            println!("  Event Proofs: {}", proof_bundle.event_proofs.len());
            println!("  Witness Blocks: {}", proof_bundle.blocks.len());
            println!();
        } else {
            println!("  No proof bundle found");
        }

        // Metadata
        println!("Metadata:");
        println!("  Generated At: {:?}", entry.generated_at);
        println!("  Source RPC: {}", entry.source_rpc);
    } else {
        println!("No proof found for instance {}", instance_id);
        println!();
        println!("Available instances: {:?}", entries.len());
    }

    Ok(())
}

fn clear_cache(db_path: &Path) -> anyhow::Result<()> {
    println!("=== Clear Cache ===");
    println!("Database: {}", db_path.display());
    println!();

    let persistence = ProofCachePersistence::open(db_path)?;
    persistence.clear_all_entries()?;

    Ok(())
}
