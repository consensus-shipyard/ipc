//! Latency Benchmark CLI
//!
//! Command-line interface for running IPC subnet latency tests.
//! This tool measures various types of latency including:
//! - End-to-end transaction latency
//! - Block confirmation latency
//! - Network propagation latency
//! - Cross-subnet message latency

use clap::{ArgAction, Command, Arg, ArgMatches, value_parser};
use serde_json;
use std::{path::PathBuf, time::Duration};
use tokio;
use throughput_bench::{
    LatencyConfig, LatencyTestRunner, LatencyTestResults,
    NetworkEndpoints, TransactionType, format_duration
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        Some(("run", sub_matches)) => run_latency_test(sub_matches).await,
        Some(("config", sub_matches)) => generate_config(sub_matches),
        Some(("analyze", sub_matches)) => analyze_results(sub_matches),
        Some(("validate", sub_matches)) => validate_config(sub_matches),
        _ => {
            eprintln!("No subcommand specified. Use --help for usage information.");
            std::process::exit(1);
        }
    }
}

fn build_cli() -> Command {
    Command::new("latency_bench")
        .version("1.0.0")
        .author("IPC Team")
        .about("IPC Subnet Latency Benchmark Tool")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(ArgAction::SetTrue)
                .global(true),
        )
        .subcommand(
            Command::new("run")
                .about("Run latency benchmark")
                .arg(
                    Arg::new("config")
                        .short('c')
                        .long("config")
                        .help("Configuration file path")
                        .value_name("FILE")
                        .value_parser(value_parser!(PathBuf))
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output file for results")
                        .value_name("FILE")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("duration")
                        .short('d')
                        .long("duration")
                        .help("Test duration (e.g., 5m, 30s)")
                        .value_name("DURATION"),
                )
                .arg(
                    Arg::new("samples")
                        .short('s')
                        .long("samples")
                        .help("Number of samples per transaction type")
                        .value_name("COUNT")
                        .value_parser(value_parser!(usize)),
                )
                .arg(
                    Arg::new("endpoints")
                        .short('e')
                        .long("endpoints")
                        .help("Network endpoints (comma-separated)")
                        .value_name("ENDPOINTS")
                        .action(ArgAction::Append),
                )
                .arg(
                    Arg::new("tx-types")
                        .short('t')
                        .long("tx-types")
                        .help("Transaction types to test (transfer,erc20,deploy,contract_call,cross_subnet)")
                        .value_name("TYPES")
                        .action(ArgAction::Append),
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .help("Output format")
                        .value_name("FORMAT")
                        .value_parser(["json", "yaml", "table"])
                        .default_value("json"),
                ),
        )
        .subcommand(
            Command::new("config")
                .about("Generate sample configuration")
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output configuration file")
                        .value_name("FILE")
                        .value_parser(value_parser!(PathBuf))
                        .required(true),
                )
                .arg(
                    Arg::new("template")
                        .short('t')
                        .long("template")
                        .help("Configuration template type")
                        .value_name("TYPE")
                        .value_parser(["basic", "comprehensive", "network_test"])
                        .default_value("basic"),
                ),
        )
        .subcommand(
            Command::new("analyze")
                .about("Analyze latency test results")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .help("Input results file")
                        .value_name("FILE")
                        .value_parser(value_parser!(PathBuf))
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output analysis report")
                        .value_name("FILE")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .help("Output format")
                        .value_name("FORMAT")
                        .value_parser(["json", "yaml", "markdown", "csv"])
                        .default_value("markdown"),
                ),
        )
        .subcommand(
            Command::new("validate")
                .about("Validate configuration file")
                .arg(
                    Arg::new("config")
                        .short('c')
                        .long("config")
                        .help("Configuration file to validate")
                        .value_name("FILE")
                        .value_parser(value_parser!(PathBuf))
                        .required(true),
                ),
        )
}

async fn run_latency_test(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = matches.get_one::<PathBuf>("config").unwrap();
    let verbose = matches.get_flag("verbose");

    if verbose {
        println!("Loading configuration from: {}", config_path.display());
    }

    // Load configuration
    let config_content = std::fs::read_to_string(config_path)?;
    let mut config: LatencyConfig = if config_path.extension() == Some("yaml".as_ref()) {
        serde_yaml::from_str(&config_content)?
    } else {
        serde_json::from_str(&config_content)?
    };

    // Override configuration with command line arguments
    if let Some(duration_str) = matches.get_one::<String>("duration") {
        config.duration = parse_duration(duration_str)?;
    }

    if let Some(samples) = matches.get_one::<usize>("samples") {
        config.samples_per_type = *samples;
    }

    if let Some(endpoints) = matches.get_many::<String>("endpoints") {
        let endpoint_list: Vec<String> = endpoints
            .flat_map(|s| s.split(','))
            .map(|s| s.trim().to_string())
            .collect();
        config.network.endpoints = endpoint_list;
    }

    if let Some(tx_types) = matches.get_many::<String>("tx-types") {
        let types: Result<Vec<TransactionType>, _> = tx_types
            .flat_map(|s| s.split(','))
            .map(|s| parse_transaction_type(s.trim()))
            .collect();
        config.transaction_types = types?;
    }

    if verbose {
        println!("Configuration loaded successfully");
        println!("Transaction types: {:?}", config.transaction_types);
        println!("Samples per type: {}", config.samples_per_type);
        println!("Test duration: {}", format_duration(config.duration));
        println!("Network endpoints: {:?}", config.network.endpoints);
    }

    // Create and run latency test
    let mut runner = LatencyTestRunner::new(config)?;

    println!("Starting latency test...");
    let results = runner.run_test().await?;

    // Output results
    let output_path = matches.get_one::<PathBuf>("output")
        .map(|p| p.clone())
        .unwrap_or_else(|| {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            PathBuf::from(format!("latency_results_{}.json", timestamp))
        });

    let format = matches.get_one::<String>("format").unwrap();
    save_results(&results, &output_path, format)?;

    // Display summary
    display_summary(&results, verbose);

    println!("Results saved to: {}", output_path.display());

    Ok(())
}

fn generate_config(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = matches.get_one::<PathBuf>("output").unwrap();
    let template = matches.get_one::<String>("template").unwrap();

    let config = match template.as_str() {
        "basic" => create_basic_config(),
        "comprehensive" => create_comprehensive_config(),
        "network_test" => create_network_test_config(),
        _ => return Err("Invalid template type".into()),
    };

    let config_content = if output_path.extension() == Some("yaml".as_ref()) {
        serde_yaml::to_string(&config)?
    } else {
        serde_json::to_string_pretty(&config)?
    };

    std::fs::write(output_path, config_content)?;
    println!("Configuration template saved to: {}", output_path.display());

    Ok(())
}

fn analyze_results(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = matches.get_one::<PathBuf>("input").unwrap();
    let format = matches.get_one::<String>("format").unwrap();

    let results_content = std::fs::read_to_string(input_path)?;
    let results: LatencyTestResults = if input_path.extension() == Some("yaml".as_ref()) {
        serde_yaml::from_str(&results_content)?
    } else {
        serde_json::from_str(&results_content)?
    };

    let analysis = generate_analysis_report(&results, format)?;

    if let Some(output_path) = matches.get_one::<PathBuf>("output") {
        std::fs::write(output_path, analysis)?;
        println!("Analysis report saved to: {}", output_path.display());
    } else {
        println!("{}", analysis);
    }

    Ok(())
}

fn validate_config(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = matches.get_one::<PathBuf>("config").unwrap();

    let config_content = std::fs::read_to_string(config_path)?;
    let _config: LatencyConfig = if config_path.extension() == Some("yaml".as_ref()) {
        serde_yaml::from_str(&config_content)?
    } else {
        serde_json::from_str(&config_content)?
    };

    println!("Configuration is valid: {}", config_path.display());
    Ok(())
}

fn create_basic_config() -> LatencyConfig {
    LatencyConfig {
        transaction_types: vec![TransactionType::Transfer, TransactionType::Erc20],
        samples_per_type: 10,
        test_interval: Duration::from_secs(1),
        confirmation_timeout: Duration::from_secs(30),
        confirmation_depth: 1,
        network: NetworkEndpoints {
            endpoints: vec![
                "http://localhost:8545".to_string(),
                "http://localhost:8546".to_string(),
            ],
        },
        duration: Duration::from_secs(300), // 5 minutes
        cross_subnet_enabled: false,
    }
}

fn create_comprehensive_config() -> LatencyConfig {
    LatencyConfig {
        transaction_types: vec![
            TransactionType::Transfer,
            TransactionType::Erc20,
            TransactionType::Deploy,
            TransactionType::ContractCall,
        ],
        samples_per_type: 20,
        test_interval: Duration::from_millis(500),
        confirmation_timeout: Duration::from_secs(60),
        confirmation_depth: 3,
        network: NetworkEndpoints {
            endpoints: vec![
                "http://localhost:8545".to_string(),
                "http://localhost:8546".to_string(),
                "http://localhost:8547".to_string(),
                "http://localhost:8548".to_string(),
            ],
        },
        duration: Duration::from_secs(600), // 10 minutes
        cross_subnet_enabled: true,
    }
}

fn create_network_test_config() -> LatencyConfig {
    LatencyConfig {
        transaction_types: vec![TransactionType::Transfer],
        samples_per_type: 50,
        test_interval: Duration::from_millis(200),
        confirmation_timeout: Duration::from_secs(30),
        confirmation_depth: 1,
        network: NetworkEndpoints {
            endpoints: vec![
                "http://localhost:8545".to_string(),
                "http://localhost:8546".to_string(),
                "http://localhost:8547".to_string(),
                "http://localhost:8548".to_string(),
                "http://localhost:8549".to_string(),
                "http://localhost:8550".to_string(),
                "http://localhost:8551".to_string(),
            ],
        },
        duration: Duration::from_secs(300), // 5 minutes
        cross_subnet_enabled: false,
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
        let seconds: u64 = duration_str.parse()?;
        Ok(Duration::from_secs(seconds))
    }
}

fn parse_transaction_type(type_str: &str) -> Result<TransactionType, Box<dyn std::error::Error>> {
    match type_str.to_lowercase().as_str() {
        "transfer" => Ok(TransactionType::Transfer),
        "erc20" => Ok(TransactionType::Erc20),
        "deploy" => Ok(TransactionType::Deploy),
        "contract_call" => Ok(TransactionType::ContractCall),
        "cross_subnet" => Ok(TransactionType::CrossSubnet),
        _ => Err(format!("Unknown transaction type: {}", type_str).into()),
    }
}

fn save_results(results: &LatencyTestResults, path: &PathBuf, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = match format {
        "json" => serde_json::to_string_pretty(results)?,
        "yaml" => serde_yaml::to_string(results)?,
        "table" => format_results_as_table(results),
        _ => return Err("Unsupported format".into()),
    };

    std::fs::write(path, content)?;
    Ok(())
}

fn format_results_as_table(results: &LatencyTestResults) -> String {
    let mut output = String::new();

    output.push_str("# Latency Test Results\n\n");

    // Test overview
    output.push_str(&format!("**Test Duration:** {}\n", format_duration(results.duration)));
    output.push_str(&format!("**Total Transactions:** {}\n", results.total_transactions));
    output.push_str(&format!("**Success Rate:** {:.2}%\n", results.success_rate * 100.0));
    output.push_str("\n");

    // Overall latency metrics
    output.push_str("## Overall Latency Metrics\n\n");
    output.push_str("| Metric | Average (ms) | P50 (ms) | P95 (ms) | P99 (ms) |\n");
    output.push_str("|--------|-------------|----------|----------|----------|\n");

    let stats = &results.stats;
    output.push_str(&format!("| End-to-End | {:.2} | {:.2} | {:.2} | {:.2} |\n",
        stats.end_to_end.avg_ms,
        stats.end_to_end.percentiles.get("P50").unwrap_or(&0.0),
        stats.end_to_end.percentiles.get("P95").unwrap_or(&0.0),
        stats.end_to_end.percentiles.get("P99").unwrap_or(&0.0)));

    output.push_str(&format!("| Mempool | {:.2} | {:.2} | {:.2} | {:.2} |\n",
        stats.mempool.avg_ms,
        stats.mempool.percentiles.get("P50").unwrap_or(&0.0),
        stats.mempool.percentiles.get("P95").unwrap_or(&0.0),
        stats.mempool.percentiles.get("P99").unwrap_or(&0.0)));

    output.push_str(&format!("| Block Inclusion | {:.2} | {:.2} | {:.2} | {:.2} |\n",
        stats.block_inclusion.avg_ms,
        stats.block_inclusion.percentiles.get("P50").unwrap_or(&0.0),
        stats.block_inclusion.percentiles.get("P95").unwrap_or(&0.0),
        stats.block_inclusion.percentiles.get("P99").unwrap_or(&0.0)));

    output.push_str(&format!("| Confirmation | {:.2} | {:.2} | {:.2} | {:.2} |\n",
        stats.confirmation.avg_ms,
        stats.confirmation.percentiles.get("P50").unwrap_or(&0.0),
        stats.confirmation.percentiles.get("P95").unwrap_or(&0.0),
        stats.confirmation.percentiles.get("P99").unwrap_or(&0.0)));

    output.push_str("\n");

    // Transaction type breakdown
    output.push_str("## Latency by Transaction Type\n\n");
    output.push_str("| Transaction Type | Samples | Average (ms) | P95 (ms) | P99 (ms) |\n");
    output.push_str("|-----------------|---------|-------------|----------|----------|\n");

    for (tx_type, metrics) in &stats.by_transaction_type {
        output.push_str(&format!("| {:?} | {} | {:.2} | {:.2} | {:.2} |\n",
            tx_type,
            metrics.samples,
            metrics.avg_ms,
            metrics.percentiles.get("P95").unwrap_or(&0.0),
            metrics.percentiles.get("P99").unwrap_or(&0.0)));
    }

    output
}

fn generate_analysis_report(results: &LatencyTestResults, format: &str) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        "json" => Ok(serde_json::to_string_pretty(results)?),
        "yaml" => Ok(serde_yaml::to_string(results)?),
        "markdown" => Ok(format_results_as_table(results)),
        "csv" => generate_csv_report(results),
        _ => Err("Unsupported format".into()),
    }
}

fn generate_csv_report(results: &LatencyTestResults) -> Result<String, Box<dyn std::error::Error>> {
    let mut output = String::new();

    // Header
    output.push_str("measurement_type,samples,avg_ms,min_ms,max_ms,p50_ms,p95_ms,p99_ms\n");

    // Overall metrics
    let stats = &results.stats;
    output.push_str(&format!("end_to_end,{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
        stats.end_to_end.samples,
        stats.end_to_end.avg_ms,
        stats.end_to_end.min_ms,
        stats.end_to_end.max_ms,
        stats.end_to_end.percentiles.get("P50").unwrap_or(&0.0),
        stats.end_to_end.percentiles.get("P95").unwrap_or(&0.0),
        stats.end_to_end.percentiles.get("P99").unwrap_or(&0.0)));

    output.push_str(&format!("mempool,{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
        stats.mempool.samples,
        stats.mempool.avg_ms,
        stats.mempool.min_ms,
        stats.mempool.max_ms,
        stats.mempool.percentiles.get("P50").unwrap_or(&0.0),
        stats.mempool.percentiles.get("P95").unwrap_or(&0.0),
        stats.mempool.percentiles.get("P99").unwrap_or(&0.0)));

    output.push_str(&format!("block_inclusion,{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
        stats.block_inclusion.samples,
        stats.block_inclusion.avg_ms,
        stats.block_inclusion.min_ms,
        stats.block_inclusion.max_ms,
        stats.block_inclusion.percentiles.get("P50").unwrap_or(&0.0),
        stats.block_inclusion.percentiles.get("P95").unwrap_or(&0.0),
        stats.block_inclusion.percentiles.get("P99").unwrap_or(&0.0)));

    Ok(output)
}

fn display_summary(results: &LatencyTestResults, verbose: bool) {
    println!("\n=== Latency Test Summary ===");
    println!("Test Duration: {}", format_duration(results.duration));
    println!("Total Transactions: {}", results.total_transactions);
    println!("Successful Transactions: {}", results.successful_transactions);
    println!("Failed Transactions: {}", results.failed_transactions);
    println!("Success Rate: {:.2}%", results.success_rate * 100.0);
    println!();

    let stats = &results.stats;
    println!("End-to-End Latency:");
    println!("  Average: {:.2}ms", stats.end_to_end.avg_ms);
    println!("  P95: {:.2}ms", stats.end_to_end.percentiles.get("P95").unwrap_or(&0.0));
    println!("  P99: {:.2}ms", stats.end_to_end.percentiles.get("P99").unwrap_or(&0.0));
    println!();

    if verbose {
        println!("Detailed Latency Breakdown:");
        println!("  Mempool: {:.2}ms avg, {:.2}ms P95",
            stats.mempool.avg_ms,
            stats.mempool.percentiles.get("P95").unwrap_or(&0.0));
        println!("  Block Inclusion: {:.2}ms avg, {:.2}ms P95",
            stats.block_inclusion.avg_ms,
            stats.block_inclusion.percentiles.get("P95").unwrap_or(&0.0));
        println!("  Confirmation: {:.2}ms avg, {:.2}ms P95",
            stats.confirmation.avg_ms,
            stats.confirmation.percentiles.get("P95").unwrap_or(&0.0));
        println!();

        println!("By Transaction Type:");
        for (tx_type, metrics) in &stats.by_transaction_type {
            println!("  {:?}: {:.2}ms avg, {:.2}ms P95 ({} samples)",
                tx_type,
                metrics.avg_ms,
                metrics.percentiles.get("P95").unwrap_or(&0.0),
                metrics.samples);
        }
    }
}