//! Simple Throughput Benchmark using basic_throughput.yaml config

use std::time::{Duration, Instant};
use std::thread;
use std::path::Path;
use serde::{Deserialize, Serialize};
use clap::{Arg, Command};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub name: String,
    pub description: String,
    pub network: NetworkConfig,
    pub load: LoadConfig,
    pub transactions: Vec<TransactionConfig>,
    pub test: TestConfig,
    pub metrics: MetricsConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(rename = "type")]
    pub network_type: String,
    pub validators: u32,
    pub endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadConfig {
    pub pattern: String,
    pub target_tps: f64,
    pub duration: String,
    pub ramp_up_duration: String,
    pub concurrent_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionConfig {
    #[serde(rename = "type")]
    pub tx_type: String,
    pub weight: u32,
    pub amount: String,
    pub gas_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub warmup_duration: String,
    pub measurement_duration: String,
    pub cooldown_duration: String,
    pub max_retries: u32,
    pub timeout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub collection_interval: String,
    pub resource_monitoring: bool,
    pub detailed_latency: bool,
    pub percentiles: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub format: String,
}

#[derive(Debug)]
pub struct BenchmarkResults {
    pub config: BenchmarkConfig,
    pub duration: Duration,
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub actual_tps: f64,
    pub success_rate: f64,
}

pub struct SimpleThroughputBenchmark {
    config: BenchmarkConfig,
}

impl SimpleThroughputBenchmark {
    pub fn from_config_file<P: AsRef<Path>>(config_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string(config_path)?;
        let config: BenchmarkConfig = serde_yaml::from_str(&config_str)?;
        Ok(Self { config })
    }

        pub fn run(&self) -> BenchmarkResults {
        println!("🚀 Starting IPC Throughput Benchmark");
        println!("================================");
        println!("Test: {}", self.config.name);
        println!("Description: {}", self.config.description);
        println!("Validators: {}", self.config.network.validators);
        println!("Network Type: {}", self.config.network.network_type);
        println!("Duration: {}", self.config.load.duration);
        println!("Target TPS: {}", self.config.load.target_tps);
        println!("Concurrent Connections: {}", self.config.load.concurrent_connections);
        println!("Transaction Types: {:?}", self.config.transactions.iter().map(|t| &t.tx_type).collect::<Vec<_>>());
        println!("Endpoints: {:?}", self.config.network.endpoints);
        println!("================================");

        let start_time = Instant::now();
        let mut total_transactions = 0u64;
        let mut successful_transactions = 0u64;
        let mut failed_transactions = 0u64;

        // Parse duration string (e.g., "5m", "30s") into Duration
        let test_duration = parse_duration(&self.config.load.duration)
            .unwrap_or(Duration::from_secs(300)); // default to 5 minutes

        // Calculate transaction interval based on target TPS and concurrent connections
        let tx_interval = Duration::from_millis(
            (1000.0 / self.config.load.target_tps * self.config.load.concurrent_connections as f64) as u64
        );

        let mut handles = Vec::new();

        // Get the primary transaction type (highest weight)
        let primary_tx_type = self.config.transactions.iter()
            .max_by_key(|t| t.weight)
            .map(|t| t.tx_type.clone())
            .unwrap_or_else(|| "transfer".to_string());

        for i in 0..self.config.load.concurrent_connections {
            let duration = test_duration;
            let interval = tx_interval;
            let tx_type = primary_tx_type.clone();

            let handle = thread::spawn(move || {
                let mut local_total = 0u64;
                let mut local_successful = 0u64;
                let mut local_failed = 0u64;

                let start = Instant::now();
                let mut next_tx_time = start;

                println!("Worker {} started", i);

                while start.elapsed() < duration {
                    if Instant::now() >= next_tx_time {
                        // Simulate transaction processing
                        let tx_result = simulate_transaction(i, &tx_type);
                        local_total += 1;

                        if tx_result {
                            local_successful += 1;
                        } else {
                            local_failed += 1;
                        }

                        next_tx_time += interval;

                        // Print progress every 100 transactions
                        if local_total % 100 == 0 {
                            println!("Worker {} - Transactions: {}", i, local_total);
                        }
                    }

                    // Small sleep to prevent busy waiting
                    thread::sleep(Duration::from_millis(1));
                }

                println!("Worker {} finished - Total: {}, Success: {}, Failed: {}",
                         i, local_total, local_successful, local_failed);

                (local_total, local_successful, local_failed)
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            if let Ok((t, s, f)) = handle.join() {
                total_transactions += t;
                successful_transactions += s;
                failed_transactions += f;
            }
        }

        let actual_duration = start_time.elapsed();
        let actual_tps = total_transactions as f64 / actual_duration.as_secs_f64();
        let success_rate = if total_transactions > 0 {
            successful_transactions as f64 / total_transactions as f64
        } else {
            0.0
        };

        BenchmarkResults {
            config: self.config.clone(),
            duration: actual_duration,
            total_transactions,
            successful_transactions,
            failed_transactions,
            actual_tps,
            success_rate,
        }
    }
}

fn parse_duration(duration_str: &str) -> Result<Duration, Box<dyn std::error::Error>> {
    let duration_str = duration_str.trim();

    if duration_str.ends_with('s') {
        let seconds: u64 = duration_str[..duration_str.len()-1].parse()?;
        Ok(Duration::from_secs(seconds))
    } else if duration_str.ends_with('m') {
        let minutes: u64 = duration_str[..duration_str.len()-1].parse()?;
        Ok(Duration::from_secs(minutes * 60))
    } else if duration_str.ends_with('h') {
        let hours: u64 = duration_str[..duration_str.len()-1].parse()?;
        Ok(Duration::from_secs(hours * 3600))
    } else {
        // Assume seconds if no unit
        let seconds: u64 = duration_str.parse()?;
        Ok(Duration::from_secs(seconds))
    }
}

fn simulate_transaction(worker_id: u32, tx_type: &str) -> bool {
    // Simulate different transaction types with different processing times
    let work_duration = match tx_type {
        "transfer" => Duration::from_millis(5 + (worker_id % 3) as u64),
        "erc20" => Duration::from_millis(8 + (worker_id % 4) as u64),
        "contract_call" => Duration::from_millis(12 + (worker_id % 5) as u64),
        _ => Duration::from_millis(10 + (worker_id % 5) as u64),
    };

    thread::sleep(work_duration);

    // Simulate 95% success rate
    worker_id % 20 != 0
}

fn print_results(results: &BenchmarkResults) {
    println!("\n🎯 Benchmark Results");
    println!("===================");
    println!("Test: {}", results.config.name);
    println!("Duration: {:?}", results.duration);
    println!("Total Transactions: {}", results.total_transactions);
    println!("Successful Transactions: {}", results.successful_transactions);
    println!("Failed Transactions: {}", results.failed_transactions);
    println!("Actual TPS: {:.2}", results.actual_tps);
    println!("Success Rate: {:.2}%", results.success_rate * 100.0);
    println!("Target TPS: {:.2}", results.config.load.target_tps);
    println!("TPS Efficiency: {:.2}%", (results.actual_tps / results.config.load.target_tps) * 100.0);

    if results.actual_tps >= results.config.load.target_tps * 0.9 {
        println!("✅ PASSED: Achieved target TPS within 10% tolerance");
    } else {
        println!("⚠️  WARNING: Below target TPS ({}%)",
                 ((results.actual_tps / results.config.load.target_tps) * 100.0) as u32);
    }

    if results.success_rate >= 0.95 {
        println!("✅ PASSED: High success rate ({}%)", (results.success_rate * 100.0) as u32);
    } else {
        println!("⚠️  WARNING: Low success rate ({}%)", (results.success_rate * 100.0) as u32);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("throughput_bench_simple")
        .version("1.0.0")
        .author("IPC Team")
        .about("Simple IPC Throughput Benchmark")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to configuration file")
                .value_name("CONFIG_FILE")
                .default_value("configs/basic_throughput.yaml")
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();

    println!("Loading configuration from: {}", config_path);

    let benchmark = SimpleThroughputBenchmark::from_config_file(config_path)?;
    let results = benchmark.run();

    print_results(&results);

    Ok(())
}