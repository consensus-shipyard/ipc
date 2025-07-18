//! Simple Throughput Benchmark CLI

use clap::{Arg, Command};
use std::time::Duration;
use throughput_bench::{SimpleBenchmark, SimpleBenchmarkConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("simple_bench")
        .version("1.0.0")
        .author("IPC Team")
        .about("Simple IPC Throughput Benchmark")
        .arg(
            Arg::new("duration")
                .short('d')
                .long("duration")
                .help("Test duration in seconds")
                .value_name("SECONDS")
                .default_value("30")
        )
        .arg(
            Arg::new("tps")
                .short('t')
                .long("tps")
                .help("Target transactions per second")
                .value_name("TPS")
                .default_value("100")
        )
        .arg(
            Arg::new("connections")
                .short('c')
                .long("connections")
                .help("Number of concurrent connections")
                .value_name("COUNT")
                .default_value("10")
        )
        .get_matches();

    let duration_secs: u64 = matches
        .get_one::<String>("duration")
        .unwrap()
        .parse()?;

    let target_tps: f64 = matches
        .get_one::<String>("tps")
        .unwrap()
        .parse()?;

    let connections: usize = matches
        .get_one::<String>("connections")
        .unwrap()
        .parse()?;

    let config = SimpleBenchmarkConfig {
        duration: Duration::from_secs(duration_secs),
        target_tps,
        concurrent_connections: connections,
    };

    let benchmark = SimpleBenchmark::new(config);
    let results = benchmark.run().await;

    println!("\n=== Benchmark Results ===");
    println!("Duration: {:?}", results.duration);
    println!("Total Transactions: {}", results.total_transactions);
    println!("Successful Transactions: {}", results.successful_transactions);
    println!("Failed Transactions: {}", results.failed_transactions);
    println!("Actual TPS: {:.2}", results.actual_tps);
    println!("Success Rate: {:.2}%", results.success_rate * 100.0);

    Ok(())
}