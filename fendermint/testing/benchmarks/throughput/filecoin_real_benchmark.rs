#!/usr/bin/env rust-script

//! Real Filecoin/IPC Subnet Benchmark - ACTUAL BLOCKCHAIN TRANSACTIONS
//!
//! This benchmark tests the performance of your IPC subnet by sending real
//! FIL token transfers using the native Filecoin transaction format.
//!
//! ```cargo
//! [dependencies]
//! tokio = { version = "1", features = ["full"] }
//! clap = { version = "4", features = ["derive"] }
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1.0"
//! chrono = { version = "0.4", features = ["serde"] }
//! anyhow = "1.0"
//! tracing = "0.1"
//! tracing-subscriber = "0.3"
//! tendermint-rpc = "0.34"
//! fendermint_rpc = { path = "../../../rpc" }
//! fendermint_vm_actor_interface = { path = "../../../vm/actor_interface" }
//! fendermint_vm_message = { path = "../../../vm/message" }
//! fendermint_crypto = { path = "../../../crypto" }
//! fvm_shared = "4.0"
//! fvm_ipld_encoding = "0.4"
//! lazy_static = "1.4"
//! futures = "0.3"
//! rand = "0.8"
//! indicatif = "0.17"
//! ```

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use fendermint_rpc::{
    client::FendermintClient,
    message::{GasParams, SignedMessageFactory},
    query::QueryClient,
    tx::{TxClient, TxCommit},
};
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_shared::{
    address::Address,
    chainid::ChainID,
    econ::TokenAmount,
};
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tendermint_rpc::Url;
use tokio::time::{sleep, timeout};
use tracing::info;

lazy_static! {
    /// Gas parameters optimized for benchmarking
    static ref GAS_PARAMS: GasParams = GasParams {
        gas_limit: 10_000_000_000,
        gas_fee_cap: TokenAmount::from_atto(1000),
        gas_premium: TokenAmount::from_atto(1000),
    };
}

#[derive(Parser)]
#[command(name = "filecoin_real_benchmark")]
#[command(about = "Real Filecoin/IPC subnet throughput benchmark - ACTUAL TRANSACTIONS")]
struct Args {
    #[arg(short, long, default_value = "http://127.0.0.1:26657")]
    endpoint: String,

    #[arg(short, long, default_value = "100")]
    target_tps: u64,

    #[arg(short, long, default_value = "30")]
    duration: u64,

    #[arg(short, long, default_value = "10")]
    concurrent_users: u32,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransactionResult {
    success: bool,
    latency_ms: u64,
    gas_used: u64,
    error: Option<String>,
    block_height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkMetrics {
    test_type: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    duration_seconds: f64,
    target_tps: u64,
    actual_tps: f64,
    total_transactions: u64,
    successful_transactions: u64,
    failed_transactions: u64,
    success_rate: f64,
    avg_latency_ms: f64,
    p95_latency_ms: f64,
    p99_latency_ms: f64,
    total_gas_used: u64,
    avg_gas_per_tx: f64,
    endpoint: String,
    concurrent_users: u32,
    chain_id: Option<u64>,
    network_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkReport {
    summary: BenchmarkMetrics,
    transaction_results: Vec<TransactionResult>,
    performance_score: f64,
    errors: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();

    // Validate endpoint URL
    let url: Url = args.endpoint.parse()
        .context("Invalid endpoint URL")?;

    info!("🚀 Starting Filecoin/IPC Real Blockchain Benchmark");
    info!("📡 Connecting to: {}", args.endpoint);
    info!("🎯 Target TPS: {}", args.target_tps);
    info!("⏱️  Duration: {} seconds", args.duration);
    info!("👥 Concurrent Users: {}", args.concurrent_users);

    // Connect to Filecoin/IPC node
    let client = FendermintClient::new_http(url, None)
        .context("Failed to create Fendermint client")?;

    // Get chain information
    let chain_id = get_chain_id(&client).await?;
    info!("🔗 Chain ID: {}", chain_id);

    // Generate test wallets
    let wallets = generate_test_wallets(args.concurrent_users)?;
    info!("💰 Generated {} test wallets", wallets.len());

    // Check if we can connect and get basic info
    if let Err(e) = test_connection(&client).await {
        return Err(anyhow!("Connection test failed: {}", e));
    }
    info!("✅ Connection test successful");

    // Run the benchmark
    let result = run_benchmark(&client, &args, &wallets, chain_id).await?;

    // Display results
    display_results(&result);

    // Save results if output specified
    if let Some(output_path) = &args.output {
        save_results(&result, output_path)?;
        info!("📊 Results saved to: {}", output_path);
    }

    Ok(())
}

/// Test connection to the Filecoin/IPC node
async fn test_connection(client: &FendermintClient) -> Result<()> {
    let _params = client
        .state_params(FvmQueryHeight::default())
        .await
        .context("Failed to get state params")?;
    Ok(())
}

/// Get chain ID from the node
async fn get_chain_id(client: &FendermintClient) -> Result<u64> {
    let params = client
        .state_params(FvmQueryHeight::default())
        .await
        .context("Failed to get state params")?;
    Ok(params.value.chain_id)
}

/// Generate test wallets for the benchmark
fn generate_test_wallets(count: u32) -> Result<Vec<fendermint_crypto::SecretKey>> {
    let mut wallets = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..count {
        let mut seed = [0u8; 32];
        rng.fill(&mut seed);
        let sk = fendermint_crypto::SecretKey::from_seed(&seed);
        wallets.push(sk);
    }

    Ok(wallets)
}

/// Run the main benchmark
async fn run_benchmark(
    client: &FendermintClient,
    args: &Args,
    wallets: &[fendermint_crypto::SecretKey],
    chain_id: u64,
) -> Result<BenchmarkReport> {
    let start_time = Utc::now();
    let results = Arc::new(Mutex::new(Vec::new()));
    let errors = Arc::new(Mutex::new(Vec::new()));

    // Create progress bar
    let total_expected = (args.target_tps * args.duration) as u64;
    let progress = ProgressBar::new(total_expected);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} TPS: {per_sec} ETA: {eta}")
            .unwrap()
    );

    // Calculate timing for target TPS
    let interval_ms = if args.target_tps > 0 {
        1000.0 / args.target_tps as f64
    } else {
        1000.0
    };

    // Spawn worker tasks
    let mut tasks = Vec::new();

    for (i, wallet) in wallets.iter().enumerate() {
        let client = client.clone();
        let wallet = wallet.clone();
        let results = Arc::clone(&results);
        let errors = Arc::clone(&errors);
        let progress = progress.clone();
        let chain_id = ChainID::from(chain_id);

        let task = tokio::spawn(async move {
            let worker_id = i;
            let worker_tps = args.target_tps / args.concurrent_users as u64;
            let worker_interval = Duration::from_millis(
                (1000.0 / worker_tps.max(1) as f64) as u64
            );

            let mut transaction_count = 0;
            let end_time = Instant::now() + Duration::from_secs(args.duration);

            while Instant::now() < end_time {
                let tx_result = send_fil_transaction(
                    &client,
                    &wallet,
                    worker_id,
                    transaction_count,
                    chain_id,
                ).await;

                match tx_result {
                    Ok(result) => {
                        results.lock().unwrap().push(result);
                        progress.inc(1);
                    }
                    Err(e) => {
                        errors.lock().unwrap().push(format!("Worker {}: {}", worker_id, e));
                        results.lock().unwrap().push(TransactionResult {
                            success: false,
                            latency_ms: 0,
                            gas_used: 0,
                            error: Some(e.to_string()),
                            block_height: None,
                        });
                        progress.inc(1);
                    }
                }

                transaction_count += 1;
                sleep(worker_interval).await;
            }
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    join_all(tasks).await;
    progress.finish();

    let end_time = Utc::now();
    let duration_seconds = (end_time - start_time).num_milliseconds() as f64 / 1000.0;

    // Process results
    let all_results = results.lock().unwrap().clone();
    let all_errors = errors.lock().unwrap().clone();

    let metrics = calculate_metrics(
        &all_results,
        start_time,
        end_time,
        duration_seconds,
        args,
        chain_id,
    );

    let performance_score = calculate_performance_score(&metrics);

    Ok(BenchmarkReport {
        summary: metrics,
        transaction_results: all_results,
        performance_score,
        errors: all_errors,
    })
}

/// Send a single FIL transaction
async fn send_fil_transaction(
    client: &FendermintClient,
    wallet: &fendermint_crypto::SecretKey,
    worker_id: usize,
    tx_count: u64,
    chain_id: ChainID,
) -> Result<TransactionResult> {
    let start_time = Instant::now();

    // Generate addresses
    let pk = wallet.public_key();
    let from_addr = Address::from(EthAddress::from(pk));
    let to_addr = Address::new_secp256k1(&pk.serialize())?; // Transfer to f1 address

    // Get current sequence number
    let sequence = get_sequence(client, &from_addr).await?;

    // Create message factory
    let mf = SignedMessageFactory::new(
        wallet.clone(),
        from_addr,
        sequence,
        chain_id,
    );

    // Bind client with message factory
    let mut bound_client = client.bind(mf);

    // Small transfer amount (0.001 FIL)
    let transfer_amount = TokenAmount::from_atto(1_000_000_000_000_000u64);

    // Send transaction with timeout
    let tx_result = timeout(
        Duration::from_secs(30),
        TxClient::<TxCommit>::transfer(
            &mut bound_client,
            to_addr,
            transfer_amount,
            GAS_PARAMS.clone(),
        ),
    )
    .await;

    let latency = start_time.elapsed().as_millis() as u64;

    match tx_result {
        Ok(Ok(response)) => {
            let success = response.response.check_tx.code.is_ok()
                && response.response.deliver_tx.code.is_ok();

            Ok(TransactionResult {
                success,
                latency_ms: latency,
                gas_used: response.response.deliver_tx.gas_used as u64,
                error: if success { None } else {
                    Some(format!("TX failed: check={:?}, deliver={:?}",
                        response.response.check_tx.code,
                        response.response.deliver_tx.code))
                },
                block_height: Some(response.response.height.value()),
            })
        }
        Ok(Err(e)) => {
            Ok(TransactionResult {
                success: false,
                latency_ms: latency,
                gas_used: 0,
                error: Some(e.to_string()),
                block_height: None,
            })
        }
        Err(_) => {
            Ok(TransactionResult {
                success: false,
                latency_ms: latency,
                gas_used: 0,
                error: Some("Transaction timeout".to_string()),
                block_height: None,
            })
        }
    }
}

/// Get the sequence number (nonce) for an address
async fn get_sequence(client: &FendermintClient, addr: &Address) -> Result<u64> {
    let state = client
        .actor_state(addr, FvmQueryHeight::default())
        .await
        .context("Failed to get actor state")?;

    match state.value {
        Some((_id, state)) => Ok(state.sequence),
        None => Ok(0), // New account starts with sequence 0
    }
}

/// Calculate benchmark metrics
fn calculate_metrics(
    results: &[TransactionResult],
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    duration_seconds: f64,
    args: &Args,
    chain_id: u64,
) -> BenchmarkMetrics {
    let total_transactions = results.len() as u64;
    let successful_transactions = results.iter().filter(|r| r.success).count() as u64;
    let failed_transactions = total_transactions - successful_transactions;

    let success_rate = if total_transactions > 0 {
        successful_transactions as f64 / total_transactions as f64
    } else {
        0.0
    };

    let actual_tps = total_transactions as f64 / duration_seconds;

    // Calculate latency statistics
    let mut successful_latencies: Vec<u64> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.latency_ms)
        .collect();

    successful_latencies.sort_unstable();

    let avg_latency_ms = if !successful_latencies.is_empty() {
        successful_latencies.iter().sum::<u64>() as f64 / successful_latencies.len() as f64
    } else {
        0.0
    };

    let p95_latency_ms = if !successful_latencies.is_empty() {
        let idx = ((successful_latencies.len() - 1) as f64 * 0.95) as usize;
        successful_latencies[idx] as f64
    } else {
        0.0
    };

    let p99_latency_ms = if !successful_latencies.is_empty() {
        let idx = ((successful_latencies.len() - 1) as f64 * 0.99) as usize;
        successful_latencies[idx] as f64
    } else {
        0.0
    };

    let total_gas_used = results.iter().map(|r| r.gas_used).sum();
    let avg_gas_per_tx = if total_transactions > 0 {
        total_gas_used as f64 / total_transactions as f64
    } else {
        0.0
    };

    BenchmarkMetrics {
        test_type: "Filecoin/IPC Real Blockchain Benchmark".to_string(),
        start_time,
        end_time,
        duration_seconds,
        target_tps: args.target_tps,
        actual_tps,
        total_transactions,
        successful_transactions,
        failed_transactions,
        success_rate,
        avg_latency_ms,
        p95_latency_ms,
        p99_latency_ms,
        total_gas_used,
        avg_gas_per_tx,
        endpoint: args.endpoint.clone(),
        concurrent_users: args.concurrent_users,
        chain_id: Some(chain_id),
        network_info: Some("Filecoin/IPC Subnet".to_string()),
    }
}

/// Calculate performance score (0.0 to 1.0)
fn calculate_performance_score(metrics: &BenchmarkMetrics) -> f64 {
    let tps_score = (metrics.actual_tps / metrics.target_tps as f64).min(1.0);
    let success_score = metrics.success_rate;
    let latency_score = (1000.0 / metrics.avg_latency_ms.max(1.0)).min(1.0);

    // Weighted average
    (tps_score * 0.4) + (success_score * 0.4) + (latency_score * 0.2)
}

/// Display benchmark results
fn display_results(report: &BenchmarkReport) {
    let m = &report.summary;

    println!("\n📊 FILECOIN/IPC BLOCKCHAIN BENCHMARK RESULTS");
    println!("═══════════════════════════════════════════════");
    println!("🌍 Network: {}", m.network_info.as_ref().unwrap_or(&"Unknown".to_string()));
    println!("🔗 Chain ID: {}", m.chain_id.unwrap_or(0));
    println!("📡 Endpoint: {}", m.endpoint);
    println!("⏱️  Duration: {:.2}s", m.duration_seconds);
    println!("👥 Concurrent Users: {}", m.concurrent_users);
    println!();
    println!("🎯 THROUGHPUT METRICS:");
    println!("   Target TPS: {}", m.target_tps);
    println!("   Actual TPS: {:.2}", m.actual_tps);
    println!("   Efficiency: {:.1}%", (m.actual_tps / m.target_tps as f64) * 100.0);
    println!();
    println!("📈 TRANSACTION METRICS:");
    println!("   Total Transactions: {}", m.total_transactions);
    println!("   Successful: {}", m.successful_transactions);
    println!("   Failed: {}", m.failed_transactions);
    println!("   Success Rate: {:.1}%", m.success_rate * 100.0);
    println!();
    println!("⚡ LATENCY METRICS:");
    println!("   Average Latency: {:.2}ms", m.avg_latency_ms);
    println!("   95th Percentile: {:.2}ms", m.p95_latency_ms);
    println!("   99th Percentile: {:.2}ms", m.p99_latency_ms);
    println!();
    println!("⛽ GAS METRICS:");
    println!("   Total Gas Used: {}", m.total_gas_used);
    println!("   Average Gas/TX: {:.0}", m.avg_gas_per_tx);
    println!();
    println!("🏆 PERFORMANCE SCORE: {:.2}/1.0", report.performance_score);

    if !report.errors.is_empty() {
        println!("\n❌ ERRORS:");
        for (i, error) in report.errors.iter().take(5).enumerate() {
            println!("   {}: {}", i + 1, error);
        }
        if report.errors.len() > 5 {
            println!("   ... and {} more errors", report.errors.len() - 5);
        }
    }
}

/// Save results to JSON file
fn save_results(report: &BenchmarkReport, output_path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(output_path, json)?;
    Ok(())
}