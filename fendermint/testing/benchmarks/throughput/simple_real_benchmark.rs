#!/usr/bin/env rust-script

//! ```cargo
//! [dependencies]
//! tokio = { version = "1", features = ["full"] }
//! ethers = "2.0"
//! clap = { version = "4", features = ["derive"] }
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1.0"
//! chrono = { version = "0.4", features = ["serde"] }
//! anyhow = "1.0"
//! tracing = "0.1"
//! tracing-subscriber = "0.3"
//! rand = "0.8"
//! ```

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use ethers::{
    core::types::*,
    middleware::Middleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::info;

#[derive(Parser)]
#[command(name = "simple_real_benchmark")]
#[command(about = "Real blockchain throughput benchmark - ACTUAL TRANSACTIONS")]
struct Args {
    #[arg(short, long, default_value = "http://localhost:8545")]
    endpoint: String,

    #[arg(short, long, default_value = "100")]
    target_tps: u32,

    #[arg(short, long, default_value = "30")]
    duration: u64,

    #[arg(short, long, default_value = "50")]
    concurrent_users: u32,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct BenchmarkResults {
    test_name: String,
    timestamp: DateTime<Utc>,
    config: TestConfig,
    results: TestResults,
    note: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TestConfig {
    endpoint: String,
    target_tps: u32,
    duration: u64,
    concurrent_users: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct TestResults {
    duration_seconds: f64,
    total_transactions: u64,
    successful_transactions: u64,
    failed_transactions: u64,
    actual_tps: f64,
    success_rate: f64,
    tps_efficiency: f64,
    avg_latency_ms: f64,
    p95_latency_ms: f64,
    p99_latency_ms: f64,
    total_gas_used: u64,
    avg_gas_per_tx: f64,
    chain_id: u64,
    errors: Vec<String>,
}

#[derive(Debug)]
struct TransactionResult {
    success: bool,
    latency_ms: u64,
    gas_used: u64,
    error: Option<String>,
    _tx_hash: Option<H256>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Connect to blockchain
    let provider = Arc::new(Provider::<Http>::try_from(&args.endpoint)?);

    // Verify connection
    let chain_id = provider.get_chainid().await?;
    let latest_block = provider.get_block_number().await?;

    info!("🌐 Connected to blockchain");
    info!("   Chain ID: {}", chain_id);
    info!("   Latest Block: {}", latest_block);
    info!("   Endpoint: {}", args.endpoint);

    // Create test wallets
    let mut wallets = Vec::new();
    for i in 0..args.concurrent_users {
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        wallets.push(wallet);
        if i < 3 {
            info!("   Test Wallet {}: {}", i + 1, wallets[i as usize].address());
        }
    }

    if args.concurrent_users > 3 {
        info!("   ... and {} more wallets", args.concurrent_users - 3);
    }

    // Start benchmark
    info!("🚀 Starting REAL blockchain throughput benchmark");
    info!("   Target TPS: {}", args.target_tps);
    info!("   Duration: {}s", args.duration);
    info!("   Concurrent Users: {}", args.concurrent_users);
    info!("   WARNING: This will send REAL transactions to the blockchain!");

    // Run the benchmark
    let results = run_benchmark(
        provider,
        wallets,
        args.target_tps,
        args.duration,
        args.concurrent_users,
    ).await?;

    // Calculate metrics
    let total_transactions = results.len() as u64;
    let successful_transactions = results.iter().filter(|r| r.success).count() as u64;
    let failed_transactions = total_transactions - successful_transactions;

    let actual_duration = results.iter()
        .map(|r| r.latency_ms)
        .max()
        .unwrap_or(0) as f64 / 1000.0;

    let actual_tps = total_transactions as f64 / actual_duration;
    let success_rate = successful_transactions as f64 / total_transactions as f64 * 100.0;
    let tps_efficiency = actual_tps / args.target_tps as f64 * 100.0;

    // Calculate latency statistics
    let mut latencies: Vec<u64> = results.iter().map(|r| r.latency_ms).collect();
    latencies.sort();
    let avg_latency_ms = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
    let p95_latency_ms = latencies.get(latencies.len() * 95 / 100).copied().unwrap_or(0) as f64;
    let p99_latency_ms = latencies.get(latencies.len() * 99 / 100).copied().unwrap_or(0) as f64;

    // Calculate gas statistics
    let total_gas_used = results.iter().map(|r| r.gas_used).sum::<u64>();
    let avg_gas_per_tx = total_gas_used as f64 / successful_transactions as f64;

    // Collect errors
    let errors: Vec<String> = results.iter()
        .filter_map(|r| r.error.as_ref())
        .take(10)
        .cloned()
        .collect();

    // Create final results
    let benchmark_results = BenchmarkResults {
        test_name: "Real Blockchain Throughput Test".to_string(),
        timestamp: Utc::now(),
        config: TestConfig {
            endpoint: args.endpoint.clone(),
            target_tps: args.target_tps,
            duration: args.duration,
            concurrent_users: args.concurrent_users,
        },
        results: TestResults {
            duration_seconds: actual_duration,
            total_transactions,
            successful_transactions,
            failed_transactions,
            actual_tps,
            success_rate,
            tps_efficiency,
            avg_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            total_gas_used,
            avg_gas_per_tx,
            chain_id: chain_id.as_u64(),
            errors,
        },
        note: "This test used REAL blockchain transactions, not simulations!".to_string(),
    };

    // Print results
    println!("\n🎯 REAL Blockchain Throughput Test Results");
    println!("==========================================");
    println!("Test: {}", benchmark_results.test_name);
    println!("Timestamp: {}", benchmark_results.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Chain ID: {}", benchmark_results.results.chain_id);
    println!("Blockchain Endpoint: {}", benchmark_results.config.endpoint);
    println!("Duration: {:.2}s", benchmark_results.results.duration_seconds);
    println!("Total Transactions: {}", benchmark_results.results.total_transactions);
    println!("Successful Transactions: {}", benchmark_results.results.successful_transactions);
    println!("Failed Transactions: {}", benchmark_results.results.failed_transactions);
    println!("Actual TPS: {:.2}", benchmark_results.results.actual_tps);
    println!("Success Rate: {:.2}%", benchmark_results.results.success_rate);
    println!("Target TPS: {}", benchmark_results.config.target_tps);
    println!("TPS Efficiency: {:.2}%", benchmark_results.results.tps_efficiency);
    println!("Average Latency: {:.2}ms", benchmark_results.results.avg_latency_ms);
    println!("P95 Latency: {:.2}ms", benchmark_results.results.p95_latency_ms);
    println!("P99 Latency: {:.2}ms", benchmark_results.results.p99_latency_ms);
    println!("Total Gas Used: {}", benchmark_results.results.total_gas_used);
    println!("Average Gas per TX: {:.0}", benchmark_results.results.avg_gas_per_tx);

    if !benchmark_results.results.errors.is_empty() {
        println!("\n❌ Sample Errors:");
        for (i, error) in benchmark_results.results.errors.iter().take(5).enumerate() {
            println!("  {}: {}", i + 1, error);
        }
    }

    // Performance assessment
    println!("\n🏆 Performance Assessment:");
    if benchmark_results.results.tps_efficiency > 90.0 && benchmark_results.results.success_rate > 95.0 {
        println!("✅ EXCELLENT: High throughput with excellent reliability!");
    } else if benchmark_results.results.tps_efficiency > 70.0 && benchmark_results.results.success_rate > 85.0 {
        println!("✅ GOOD: Good throughput with decent reliability");
    } else if benchmark_results.results.tps_efficiency > 50.0 && benchmark_results.results.success_rate > 70.0 {
        println!("⚠️ AVERAGE: Moderate throughput, consider optimizations");
    } else {
        println!("❌ POOR: Low throughput or reliability, needs investigation");
    }

    // Save results
    if let Some(output_path) = &args.output {
        let json = serde_json::to_string_pretty(&benchmark_results)?;
        std::fs::write(output_path, json)?;
        println!("\n📄 Results saved to: {}", output_path);
    }

    println!("\n✅ Real blockchain throughput benchmark completed!");
    println!("   This test used ACTUAL blockchain transactions.");
    println!("   {} transactions were sent to chain ID {}",
             benchmark_results.results.total_transactions,
             benchmark_results.results.chain_id);

    Ok(())
}

async fn run_benchmark(
    provider: Arc<Provider<Http>>,
    wallets: Vec<LocalWallet>,
    target_tps: u32,
    duration_secs: u64,
    concurrent_users: u32,
) -> Result<Vec<TransactionResult>> {
    let start_time = Instant::now();
    let duration = Duration::from_secs(duration_secs);
    let mut results = Vec::new();

    let interval = Duration::from_secs_f64(1.0 / target_tps as f64);
    let mut transaction_count = 0;
    let mut next_transaction_time = start_time;

    while start_time.elapsed() < duration {
        if Instant::now() >= next_transaction_time {
            let wallet_idx = transaction_count % concurrent_users as u64;
            let wallet = &wallets[wallet_idx as usize];

            // Send real transaction
            let result = send_real_transaction(provider.clone(), wallet.clone()).await;
            results.push(result);

            transaction_count += 1;
            next_transaction_time = start_time + interval * transaction_count as u32;

            // Progress indicator
            if transaction_count % 50 == 0 {
                info!("Sent {} transactions...", transaction_count);
            }
        }

        // Small sleep to prevent busy waiting
        sleep(Duration::from_millis(1)).await;
    }

    info!("Completed {} transactions in {:.2}s", results.len(), start_time.elapsed().as_secs_f64());

    Ok(results)
}

async fn send_real_transaction(
    provider: Arc<Provider<Http>>,
    _wallet: LocalWallet,
) -> TransactionResult {
    let start_time = Instant::now();

    // Create a real transaction (small transfer)
    let to_address = Address::random();
    let value = U256::from(1000000000000000u64); // 0.001 ETH

    let tx = TransactionRequest::new()
        .to(to_address)
        .value(value)
        .gas(21000)
        .gas_price(U256::from(1000000000u64)); // 1 gwei

    match provider.send_transaction(tx, None).await {
        Ok(pending_tx) => {
            match pending_tx.await {
                Ok(Some(receipt)) => {
                    let latency = start_time.elapsed().as_millis() as u64;
                    TransactionResult {
                        success: true,
                        latency_ms: latency,
                        gas_used: receipt.gas_used.unwrap_or(U256::zero()).as_u64(),
                        error: None,
                        _tx_hash: Some(receipt.transaction_hash),
                    }
                }
                Ok(None) => {
                    let latency = start_time.elapsed().as_millis() as u64;
                    TransactionResult {
                        success: false,
                        latency_ms: latency,
                        gas_used: 0,
                        error: Some("Transaction receipt not found".to_string()),
                        _tx_hash: None,
                    }
                }
                Err(e) => {
                    let latency = start_time.elapsed().as_millis() as u64;
                    TransactionResult {
                        success: false,
                        latency_ms: latency,
                        gas_used: 0,
                        error: Some(e.to_string()),
                        _tx_hash: None,
                    }
                }
            }
        }
        Err(e) => {
            let latency = start_time.elapsed().as_millis() as u64;
            TransactionResult {
                success: false,
                latency_ms: latency,
                gas_used: 0,
                error: Some(e.to_string()),
                _tx_hash: None,
            }
        }
    }
}