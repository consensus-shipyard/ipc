// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! CLI for running throughput benchmarks

use std::path::PathBuf;
use std::time::Duration;
use clap::{Parser, Subcommand};
use tracing::{info, error};
use fendermint_benchmarks_throughput::{
    BenchmarkConfig, BenchmarkRunner, TransactionType, LoadPattern,
    NetworkEndpoints, TransactionConfig, utils::*,
};

#[derive(Parser)]
#[command(
    name = "throughput-bench",
    about = "IPC Subnet Throughput Benchmarking Tool",
    version,
    long_about = "A comprehensive benchmarking tool for measuring transaction throughput, latency, and resource utilization in IPC subnets."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a basic throughput benchmark
    Run {
        /// Number of validators
        #[arg(short = 'v', long, default_value = "4")]
        validators: usize,

        /// Transaction type to benchmark
        #[arg(short = 't', long, default_value = "transfer")]
        transaction_type: String,

        /// Test duration (e.g., 300s, 5m, 1h)
        #[arg(short = 'd', long, default_value = "300s")]
        duration: String,

        /// Number of concurrent users
        #[arg(short = 'c', long, default_value = "100")]
        concurrent_users: usize,

        /// Target transactions per second
        #[arg(long, default_value = "1000")]
        target_tps: u64,

        /// Load pattern (constant, ramp-up, burst, periodic)
        #[arg(short = 'p', long, default_value = "constant")]
        load_pattern: String,

        /// Ethereum RPC endpoint
        #[arg(long, default_value = "http://localhost:8545")]
        eth_rpc: String,

        /// CometBFT RPC endpoint
        #[arg(long, default_value = "http://localhost:26657")]
        cometbft_rpc: String,

        /// Output file for results
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },

    /// Generate a configuration file
    Config {
        /// Output path for the configuration file
        #[arg(short = 'o', long, default_value = "benchmark-config.yaml")]
        output: PathBuf,

        /// Configuration template (default, high-throughput, low-latency)
        #[arg(short = 't', long, default_value = "default")]
        template: String,
    },

    /// Validate a configuration file
    Validate {
        /// Configuration file to validate
        #[arg(short = 'f', long)]
        file: PathBuf,
    },

    /// Analyze benchmark results
    Analyze {
        /// Results file to analyze
        #[arg(short = 'f', long)]
        file: PathBuf,

        /// Generate detailed report
        #[arg(short = 'r', long)]
        report: bool,

        /// Compare with another results file
        #[arg(short = 'c', long)]
        compare: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();

    match cli.command {
        Commands::Run {
            validators,
            transaction_type,
            duration,
            concurrent_users,
            target_tps,
            load_pattern,
            eth_rpc,
            cometbft_rpc,
            output,
        } => {
            run_benchmark(
                validators,
                transaction_type,
                duration,
                concurrent_users,
                target_tps,
                load_pattern,
                eth_rpc,
                cometbft_rpc,
                output,
                cli.config,
            ).await?;
        }
        Commands::Config { output, template } => {
            generate_config(output, template)?;
        }
        Commands::Validate { file } => {
            validate_config(file)?;
        }
        Commands::Analyze { file, report, compare } => {
            analyze_results(file, report, compare)?;
        }
    }

    Ok(())
}

async fn run_benchmark(
    validators: usize,
    transaction_type: String,
    duration: String,
    concurrent_users: usize,
    target_tps: u64,
    load_pattern: String,
    eth_rpc: String,
    cometbft_rpc: String,
    output: Option<PathBuf>,
    config_file: Option<PathBuf>,
) -> anyhow::Result<()> {
    info!("Starting throughput benchmark");

    // Parse inputs
    let duration = parse_duration(&duration)?;
    let tx_type = parse_transaction_type(&transaction_type)?;
    let load_pattern = parse_load_pattern(&load_pattern)?;

    // Validate configuration
    validate_benchmark_config(validators, duration, concurrent_users, target_tps)?;

    // Create benchmark configuration
    let mut config = if let Some(config_path) = config_file {
        info!("Loading configuration from: {}", config_path.display());
        BenchmarkConfig::from_yaml_file(config_path.to_str().unwrap())?
    } else {
        BenchmarkConfig::default()
    };

    // Override with CLI parameters
    config.validators = validators;
    config.transaction_type = tx_type;
    config.duration = duration;
    config.concurrent_users = concurrent_users;
    config.target_tps = target_tps;
    config.load_pattern = load_pattern;
    config.network_endpoints = NetworkEndpoints {
        eth_rpc_url: eth_rpc,
        cometbft_rpc_url: cometbft_rpc,
        metrics_url: Some("http://localhost:9184/metrics".to_string()),
    };

    // Create and run benchmark
    let mut runner = BenchmarkRunner::new(config).await?;
    let results = runner.run().await?;

    // Display results summary
    display_results_summary(&results);

    // Save results if output file specified
    if let Some(output_path) = output {
        runner.save_results(&results, output_path.to_str().unwrap())?;
        info!("Results saved to: {}", output_path.display());
    }

    Ok(())
}

fn generate_config(output: PathBuf, template: String) -> anyhow::Result<()> {
    info!("Generating configuration file: {}", output.display());

    let config = match template.as_str() {
        "default" => BenchmarkConfig::default(),
        "high-throughput" => BenchmarkConfig::builder()
            .validators(1)
            .target_tps(5000)
            .concurrent_users(200)
            .duration(Duration::from_secs(600))
            .build()?,
        "low-latency" => BenchmarkConfig::builder()
            .validators(4)
            .target_tps(1000)
            .concurrent_users(50)
            .duration(Duration::from_secs(300))
            .transaction_timeout(Duration::from_secs(1))
            .build()?,
        "stress-test" => BenchmarkConfig::builder()
            .validators(7)
            .target_tps(2000)
            .concurrent_users(500)
            .duration(Duration::from_secs(1800))
            .load_pattern(LoadPattern::RampUp)
            .build()?,
        _ => return Err(anyhow::anyhow!("Unknown template: {}", template)),
    };

    config.to_yaml_file(output.to_str().unwrap())?;
    info!("Configuration file generated successfully");

    Ok(())
}

fn validate_config(file: PathBuf) -> anyhow::Result<()> {
    info!("Validating configuration file: {}", file.display());

    let config = BenchmarkConfig::from_yaml_file(file.to_str().unwrap())?;

    // Additional validation
    validate_benchmark_config(
        config.validators,
        config.duration,
        config.concurrent_users,
        config.target_tps,
    )?;

    info!("Configuration file is valid");
    println!("✓ Configuration validation passed");
    println!("  - Validators: {}", config.validators);
    println!("  - Transaction type: {}", config.transaction_type);
    println!("  - Duration: {}", format_duration(config.duration));
    println!("  - Concurrent users: {}", config.concurrent_users);
    println!("  - Target TPS: {}", config.target_tps);

    Ok(())
}

fn analyze_results(file: PathBuf, report: bool, compare: Option<PathBuf>) -> anyhow::Result<()> {
    info!("Analyzing results file: {}", file.display());

    let results = BenchmarkRunner::load_results(file.to_str().unwrap())?;

    if report {
        generate_detailed_report(&results)?;
    } else {
        display_results_summary(&results);
    }

    if let Some(compare_file) = compare {
        let other_results = BenchmarkRunner::load_results(compare_file.to_str().unwrap())?;
        compare_results(&results, &other_results)?;
    }

    Ok(())
}

fn display_results_summary(results: &fendermint_benchmarks_throughput::BenchmarkResults) {
    println!("\n{}", "=".repeat(60));
    println!("BENCHMARK RESULTS SUMMARY");
    println!("{}", "=".repeat(60));

    let summary_data = [
        ("Configuration", format!("{} validators, {} users", results.config.validators, results.config.concurrent_users)),
        ("Transaction Type", results.config.transaction_type.to_string()),
        ("Duration", format_duration(results.duration)),
        ("Total Transactions", format_number(results.total_transactions)),
        ("Successful", format_number(results.successful_transactions)),
        ("Failed", format_number(results.failed_transactions)),
        ("Success Rate", format!("{:.2}%", calculate_success_rate(results.successful_transactions, results.total_transactions))),
        ("Average TPS", format!("{:.2}", results.average_tps)),
        ("Peak TPS", format!("{:.2}", results.peak_tps)),
        ("Latency P50", format!("{:.2}ms", results.latency_p50)),
        ("Latency P95", format!("{:.2}ms", results.latency_p95)),
        ("Latency P99", format!("{:.2}ms", results.latency_p99)),
        ("Avg CPU Usage", format!("{:.2}%", results.resource_metrics.avg_cpu_usage)),
        ("Peak CPU Usage", format!("{:.2}%", results.resource_metrics.peak_cpu_usage)),
        ("Avg Memory Usage", format!("{:.2}MB", results.resource_metrics.avg_memory_usage)),
    ];

    let summary_table = create_summary_table(
        &summary_data.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>()
    );

    println!("{}", summary_table);

    if !results.error_breakdown.is_empty() {
        println!("Error Breakdown:");
        for (error, count) in &results.error_breakdown {
            println!("  • {}: {}", error, count);
        }
    }
}

fn generate_detailed_report(results: &fendermint_benchmarks_throughput::BenchmarkResults) -> anyhow::Result<()> {
    println!("\n{}", "=".repeat(80));
    println!("DETAILED BENCHMARK REPORT");
    println!("{}", "=".repeat(80));

    // Display basic summary
    display_results_summary(results);

    // TPS over time analysis
    println!("\nTPS Over Time:");
    println!("{}", "-".repeat(40));

    let window_size = 10; // 10 data points
    let step = results.time_series.len() / window_size.min(results.time_series.len());

    for (i, point) in results.time_series.iter().enumerate() {
        if i % step == 0 {
            let progress = progress_bar(i as u64, results.time_series.len() as u64, 20);
            println!("{:6.1}s: {} {:.2} TPS",
                     point.timestamp, progress, point.tps);
        }
    }

    // Performance characteristics
    println!("\nPerformance Characteristics:");
    println!("{}", "-".repeat(40));

    let efficiency = results.average_tps / results.config.target_tps as f64;
    println!("Target Efficiency: {:.2}% ({:.2}/{} TPS)",
             efficiency * 100.0, results.average_tps, results.config.target_tps);

    let tps_per_validator = results.average_tps / results.config.validators as f64;
    println!("TPS per Validator: {:.2}", tps_per_validator);

    let latency_vs_tps = results.latency_p95 / results.average_tps;
    println!("Latency/TPS Ratio: {:.2}ms per TPS", latency_vs_tps);

    // Resource utilization
    println!("\nResource Utilization:");
    println!("{}", "-".repeat(40));

    let cpu_efficiency = results.average_tps / results.resource_metrics.avg_cpu_usage;
    println!("CPU Efficiency: {:.2} TPS per 1% CPU", cpu_efficiency);

    let memory_efficiency = results.average_tps / results.resource_metrics.avg_memory_usage;
    println!("Memory Efficiency: {:.2} TPS per MB", memory_efficiency);

    Ok(())
}

fn compare_results(
    results1: &fendermint_benchmarks_throughput::BenchmarkResults,
    results2: &fendermint_benchmarks_throughput::BenchmarkResults,
) -> anyhow::Result<()> {
    println!("\n{}", "=".repeat(60));
    println!("BENCHMARK COMPARISON");
    println!("{}", "=".repeat(60));

    let compare_data = [
        ("Average TPS", results1.average_tps, results2.average_tps),
        ("Peak TPS", results1.peak_tps, results2.peak_tps),
        ("Latency P50", results1.latency_p50, results2.latency_p50),
        ("Latency P95", results1.latency_p95, results2.latency_p95),
        ("Latency P99", results1.latency_p99, results2.latency_p99),
        ("Success Rate", calculate_success_rate(results1.successful_transactions, results1.total_transactions),
         calculate_success_rate(results2.successful_transactions, results2.total_transactions)),
    ];

    println!("{:<15} {:<15} {:<15} {:<15}", "Metric", "Run 1", "Run 2", "Difference");
    println!("{}", "-".repeat(60));

    for (metric, val1, val2) in compare_data {
        let diff = val2 - val1;
        let diff_percent = if val1 != 0.0 { (diff / val1) * 100.0 } else { 0.0 };

        let diff_str = if diff > 0.0 {
            format!("+{:.2} ({:+.1}%)", diff, diff_percent)
        } else {
            format!("{:.2} ({:.1}%)", diff, diff_percent)
        };

        println!("{:<15} {:<15.2} {:<15.2} {:<15}", metric, val1, val2, diff_str);
    }

    Ok(())
}

fn parse_transaction_type(s: &str) -> anyhow::Result<TransactionType> {
    match s.to_lowercase().as_str() {
        "transfer" => Ok(TransactionType::Transfer),
        "erc20" => Ok(TransactionType::Erc20),
        "deploy" => Ok(TransactionType::Deploy),
        "contract_call" | "contract-call" => Ok(TransactionType::ContractCall),
        "cross_subnet" | "cross-subnet" => Ok(TransactionType::CrossSubnet),
        _ => Err(anyhow::anyhow!("Unknown transaction type: {}", s)),
    }
}

fn parse_load_pattern(s: &str) -> anyhow::Result<LoadPattern> {
    match s.to_lowercase().as_str() {
        "constant" => Ok(LoadPattern::Constant),
        "ramp-up" | "ramp_up" => Ok(LoadPattern::RampUp),
        "burst" => Ok(LoadPattern::Burst),
        "periodic" => Ok(LoadPattern::Periodic),
        _ => Err(anyhow::anyhow!("Unknown load pattern: {}", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_transaction_type() {
        assert!(matches!(parse_transaction_type("transfer").unwrap(), TransactionType::Transfer));
        assert!(matches!(parse_transaction_type("erc20").unwrap(), TransactionType::Erc20));
        assert!(matches!(parse_transaction_type("deploy").unwrap(), TransactionType::Deploy));
        assert!(parse_transaction_type("invalid").is_err());
    }

    #[test]
    fn test_parse_load_pattern() {
        assert!(matches!(parse_load_pattern("constant").unwrap(), LoadPattern::Constant));
        assert!(matches!(parse_load_pattern("ramp-up").unwrap(), LoadPattern::RampUp));
        assert!(matches!(parse_load_pattern("burst").unwrap(), LoadPattern::Burst));
        assert!(parse_load_pattern("invalid").is_err());
    }
}