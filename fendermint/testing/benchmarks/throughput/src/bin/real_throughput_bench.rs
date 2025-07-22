use anyhow::Result;
use ethers::{
    core::types::*,
    middleware::{Middleware, SignerMiddleware},
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    utils::hex,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "real_throughput_bench")]
#[command(about = "Real blockchain throughput benchmark")]
struct Args {
    #[arg(short, long, default_value = "http://localhost:8545")]
    endpoint: String,

    #[arg(short, long, default_value = "1000")]
    target_tps: u32,

    #[arg(short, long, default_value = "60")]
    duration: u64,

    #[arg(short, long, default_value = "100")]
    concurrent_users: u32,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestResults {
    test_name: String,
    timestamp: String,
    config: BenchmarkConfig,
    duration: Duration,
    total_transactions: u64,
    successful_transactions: u64,
    failed_transactions: u64,
    actual_tps: f64,
    success_rate: f64,
    target_tps: u32,
    tps_efficiency: f64,
    transaction_latencies: Vec<u64>,
    avg_latency_ms: f64,
    p95_latency_ms: f64,
    p99_latency_ms: f64,
    gas_used: Vec<u64>,
    avg_gas_used: f64,
    total_gas_used: u64,
    errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BenchmarkConfig {
    endpoint: String,
    target_tps: u32,
    duration: u64,
    concurrent_users: u32,
    transaction_types: Vec<String>,
}

#[derive(Debug, Clone)]
struct TransactionResult {
    success: bool,
    latency_ms: u64,
    gas_used: u64,
    error: Option<String>,
    tx_hash: Option<H256>,
}

struct RealThroughputBenchmark {
    provider: Arc<Provider<Http>>,
    wallets: Vec<LocalWallet>,
    config: BenchmarkConfig,
    results: Vec<TransactionResult>,
}

impl RealThroughputBenchmark {
    async fn new(args: &Args) -> Result<Self> {
        let provider = Arc::new(Provider::<Http>::try_from(&args.endpoint)?);

        // Check if we can connect to the blockchain
        let chain_id = provider.get_chainid().await?;
        info!("Connected to blockchain with chain ID: {}", chain_id);

        // Create test wallets
        let mut wallets = Vec::new();
        for i in 0..args.concurrent_users {
            let wallet = LocalWallet::new(&mut rand::thread_rng());
            wallets.push(wallet);
        }

        let config = BenchmarkConfig {
            endpoint: args.endpoint.clone(),
            target_tps: args.target_tps,
            duration: args.duration,
            concurrent_users: args.concurrent_users,
            transaction_types: vec!["transfer".to_string(), "contract_call".to_string()],
        };

        Ok(Self {
            provider,
            wallets,
            config,
            results: Vec::new(),
        })
    }

    async fn run_benchmark(&mut self) -> Result<TestResults> {
        let start_time = Instant::now();
        let duration = Duration::from_secs(self.config.duration);

        info!("🚀 Starting REAL blockchain throughput benchmark");
        info!("Target TPS: {}", self.config.target_tps);
        info!("Duration: {}s", self.config.duration);
        info!("Concurrent Users: {}", self.config.concurrent_users);
        info!("Blockchain Endpoint: {}", self.config.endpoint);

        // Get initial account balances and nonces
        let mut tasks = Vec::new();
        let transactions_per_second = self.config.target_tps as f64;
        let interval = Duration::from_secs_f64(1.0 / transactions_per_second);

        let mut transaction_count = 0;
        let mut next_transaction_time = start_time;

        while start_time.elapsed() < duration {
            if Instant::now() >= next_transaction_time {
                // Create a real transaction
                let wallet_idx = transaction_count % self.config.concurrent_users as u64;
                let wallet = &self.wallets[wallet_idx as usize];

                let task = self.send_real_transaction(wallet.clone(), transaction_count).await;
                tasks.push(task);

                transaction_count += 1;
                next_transaction_time = start_time + interval * transaction_count as u32;
            }

            // Small sleep to prevent busy waiting
            sleep(Duration::from_millis(1)).await;
        }

        // Wait for all transactions to complete
        info!("Waiting for {} transactions to complete...", tasks.len());
        let mut results = Vec::new();

        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Transaction failed: {}", e);
                    results.push(TransactionResult {
                        success: false,
                        latency_ms: 0,
                        gas_used: 0,
                        error: Some(e.to_string()),
                        tx_hash: None,
                    });
                }
            }
        }

        self.results = results;

        // Calculate metrics
        let test_duration = start_time.elapsed();
        let total_transactions = self.results.len() as u64;
        let successful_transactions = self.results.iter().filter(|r| r.success).count() as u64;
        let failed_transactions = total_transactions - successful_transactions;
        let actual_tps = total_transactions as f64 / test_duration.as_secs_f64();
        let success_rate = successful_transactions as f64 / total_transactions as f64 * 100.0;
        let tps_efficiency = actual_tps / self.config.target_tps as f64 * 100.0;

        // Calculate latency statistics
        let mut latencies: Vec<u64> = self.results.iter().map(|r| r.latency_ms).collect();
        latencies.sort();
        let avg_latency_ms = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
        let p95_latency_ms = latencies.get(latencies.len() * 95 / 100).copied().unwrap_or(0) as f64;
        let p99_latency_ms = latencies.get(latencies.len() * 99 / 100).copied().unwrap_or(0) as f64;

        // Calculate gas usage
        let gas_used: Vec<u64> = self.results.iter().map(|r| r.gas_used).collect();
        let avg_gas_used = gas_used.iter().sum::<u64>() as f64 / gas_used.len() as f64;
        let total_gas_used = gas_used.iter().sum::<u64>();

        // Collect errors
        let errors: Vec<String> = self.results.iter()
            .filter_map(|r| r.error.as_ref())
            .cloned()
            .collect();

        Ok(TestResults {
            test_name: "Real Blockchain Throughput Test".to_string(),
            timestamp: chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string(),
            config: self.config.clone(),
            duration: test_duration,
            total_transactions,
            successful_transactions,
            failed_transactions,
            actual_tps,
            success_rate,
            target_tps: self.config.target_tps,
            tps_efficiency,
            transaction_latencies: latencies,
            avg_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            gas_used,
            avg_gas_used,
            total_gas_used,
            errors,
        })
    }

    async fn send_real_transaction(&self, wallet: LocalWallet, nonce: u64) -> Result<TransactionResult> {
        let start_time = Instant::now();

        // Create a real transaction (simple transfer)
        let to_address = Address::random();
        let value = U256::from(1000000000000000u64); // 0.001 ETH

        let tx = TransactionRequest::new()
            .to(to_address)
            .value(value)
            .gas(21000)
            .gas_price(U256::from(1000000000u64)); // 1 gwei

        match self.provider.send_transaction(tx, None).await {
            Ok(pending_tx) => {
                match pending_tx.await {
                    Ok(Some(receipt)) => {
                        let latency = start_time.elapsed().as_millis() as u64;
                        Ok(TransactionResult {
                            success: true,
                            latency_ms: latency,
                            gas_used: receipt.gas_used.unwrap_or(U256::zero()).as_u64(),
                            error: None,
                            tx_hash: Some(receipt.transaction_hash),
                        })
                    }
                    Ok(None) => {
                        let latency = start_time.elapsed().as_millis() as u64;
                        Ok(TransactionResult {
                            success: false,
                            latency_ms: latency,
                            gas_used: 0,
                            error: Some("Transaction receipt not found".to_string()),
                            tx_hash: None,
                        })
                    }
                    Err(e) => {
                        let latency = start_time.elapsed().as_millis() as u64;
                        Ok(TransactionResult {
                            success: false,
                            latency_ms: latency,
                            gas_used: 0,
                            error: Some(e.to_string()),
                            tx_hash: None,
                        })
                    }
                }
            }
            Err(e) => {
                let latency = start_time.elapsed().as_millis() as u64;
                Ok(TransactionResult {
                    success: false,
                    latency_ms: latency,
                    gas_used: 0,
                    error: Some(e.to_string()),
                    tx_hash: None,
                })
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create and run benchmark
    let mut benchmark = RealThroughputBenchmark::new(&args).await?;
    let results = benchmark.run_benchmark().await?;

    // Print results
    println!("\n🎯 REAL Blockchain Throughput Test Results");
    println!("==========================================");
    println!("Test: {}", results.test_name);
    println!("Timestamp: {}", results.timestamp);
    println!("Blockchain Endpoint: {}", results.config.endpoint);
    println!("Duration: {:.2}s", results.duration.as_secs_f64());
    println!("Total Transactions: {}", results.total_transactions);
    println!("Successful Transactions: {}", results.successful_transactions);
    println!("Failed Transactions: {}", results.failed_transactions);
    println!("Actual TPS: {:.2}", results.actual_tps);
    println!("Success Rate: {:.2}%", results.success_rate);
    println!("Target TPS: {}", results.target_tps);
    println!("TPS Efficiency: {:.2}%", results.tps_efficiency);
    println!("Average Latency: {:.2}ms", results.avg_latency_ms);
    println!("P95 Latency: {:.2}ms", results.p95_latency_ms);
    println!("P99 Latency: {:.2}ms", results.p99_latency_ms);
    println!("Average Gas Used: {:.0}", results.avg_gas_used);
    println!("Total Gas Used: {}", results.total_gas_used);

    if !results.errors.is_empty() {
        println!("\n❌ Errors encountered:");
        for (i, error) in results.errors.iter().take(5).enumerate() {
            println!("  {}: {}", i + 1, error);
        }
        if results.errors.len() > 5 {
            println!("  ... and {} more errors", results.errors.len() - 5);
        }
    }

    // Performance assessment
    println!("\n🏆 Performance Assessment:");
    if results.tps_efficiency > 90.0 && results.success_rate > 95.0 {
        println!("✅ EXCELLENT: High throughput with excellent reliability!");
    } else if results.tps_efficiency > 70.0 && results.success_rate > 85.0 {
        println!("✅ GOOD: Good throughput with decent reliability");
    } else if results.tps_efficiency > 50.0 && results.success_rate > 70.0 {
        println!("⚠️ AVERAGE: Moderate throughput, consider optimizations");
    } else {
        println!("❌ POOR: Low throughput or reliability, needs investigation");
    }

    // Save results if output specified
    if let Some(output_path) = &args.output {
        let json = serde_json::to_string_pretty(&results)?;
        std::fs::write(output_path, json)?;
        println!("\n📄 Results saved to: {}", output_path.display());
    }

    println!("\n✅ Real blockchain throughput benchmark completed!");
    println!("This test used ACTUAL blockchain transactions, not simulations.");

    Ok(())
}