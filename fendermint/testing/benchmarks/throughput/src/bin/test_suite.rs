//! Test Suite CLI
//!
//! Command-line interface for running complete IPC subnet benchmark test suites.
//! This tool orchestrates both throughput and latency tests with comprehensive
//! configuration management, result aggregation, and automated reporting.

use clap::{ArgAction, Command, Arg, ArgMatches, value_parser};
use serde_json;
use std::{path::PathBuf, time::Duration};
use tokio;
use throughput_bench::{
    AutomatedTestRunner, TestSuiteConfig, TestSuiteResults, TestConfigs,
    BenchmarkConfig, LatencyConfig, NetworkEndpoints, TransactionType,
    ReportingConfig, CleanupConfig, create_default_test_suite_config,
    format_duration, format_number,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = build_cli().get_matches();

    match matches.subcommand() {
        Some(("run", sub_matches)) => run_test_suite(sub_matches).await,
        Some(("config", sub_matches)) => generate_config(sub_matches),
        Some(("validate", sub_matches)) => validate_config(sub_matches),
        Some(("analyze", sub_matches)) => analyze_results(sub_matches),
        Some(("status", sub_matches)) => show_status(sub_matches),
        _ => {
            eprintln!("No subcommand specified. Use --help for usage information.");
            std::process::exit(1);
        }
    }
}

fn build_cli() -> Command {
    Command::new("test_suite")
        .version("1.0.0")
        .author("IPC Team")
        .about("IPC Subnet Benchmark Test Suite")
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
                .about("Run complete test suite")
                .arg(
                    Arg::new("config")
                        .short('c')
                        .long("config")
                        .help("Test suite configuration file")
                        .value_name("FILE")
                        .value_parser(value_parser!(PathBuf))
                        .required(true),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output directory for results")
                        .value_name("DIR")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("throughput-only")
                        .long("throughput-only")
                        .help("Run only throughput tests")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("latency-only")
                        .long("latency-only")
                        .help("Run only latency tests")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("timeout")
                        .long("timeout")
                        .help("Maximum test duration (e.g., 2h, 30m)")
                        .value_name("DURATION"),
                )
                .arg(
                    Arg::new("parallel")
                        .long("parallel")
                        .help("Run tests in parallel where possible")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("config")
                .about("Generate test suite configuration")
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
                        .help("Configuration template")
                        .value_name("TYPE")
                        .value_parser(["minimal", "comprehensive", "performance", "stability"])
                        .default_value("comprehensive"),
                )
                .arg(
                    Arg::new("validators")
                        .long("validators")
                        .help("Number of validators to test")
                        .value_name("COUNT")
                        .value_parser(value_parser!(usize))
                        .default_value("4"),
                )
                .arg(
                    Arg::new("endpoints")
                        .long("endpoints")
                        .help("Network endpoints (comma-separated)")
                        .value_name("ENDPOINTS")
                        .default_value("http://localhost:8545,http://localhost:8546"),
                ),
        )
        .subcommand(
            Command::new("validate")
                .about("Validate test suite configuration")
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
        .subcommand(
            Command::new("analyze")
                .about("Analyze test suite results")
                .arg(
                    Arg::new("input")
                        .short('i')
                        .long("input")
                        .help("Test suite results file")
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
                        .value_parser(["json", "html", "csv", "markdown"])
                        .default_value("html"),
                )
                .arg(
                    Arg::new("compare")
                        .long("compare")
                        .help("Compare with previous results")
                        .value_name("FILE")
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("status")
                .about("Show test suite status")
                .arg(
                    Arg::new("results-dir")
                        .short('d')
                        .long("results-dir")
                        .help("Results directory to scan")
                        .value_name("DIR")
                        .value_parser(value_parser!(PathBuf))
                        .default_value("./benchmark_results"),
                )
                .arg(
                    Arg::new("recent")
                        .long("recent")
                        .help("Show only recent results (last N days)")
                        .value_name("DAYS")
                        .value_parser(value_parser!(u32))
                        .default_value("7"),
                ),
        )
}

async fn run_test_suite(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = matches.get_one::<PathBuf>("config").unwrap();
    let verbose = matches.get_flag("verbose");

    if verbose {
        println!("Loading test suite configuration from: {}", config_path.display());
    }

    // Load configuration
    let config_content = std::fs::read_to_string(config_path)?;
    let mut config: TestSuiteConfig = if config_path.extension() == Some("yaml".as_ref()) {
        serde_yaml::from_str(&config_content)?
    } else {
        serde_json::from_str(&config_content)?
    };

    // Override configuration with command line arguments
    if let Some(output_dir) = matches.get_one::<PathBuf>("output") {
        config.output_dir = output_dir.clone();
    }

    if matches.get_flag("throughput-only") {
        config.run_throughput_tests = true;
        config.run_latency_tests = false;
    } else if matches.get_flag("latency-only") {
        config.run_throughput_tests = false;
        config.run_latency_tests = true;
    }

    if let Some(timeout_str) = matches.get_one::<String>("timeout") {
        config.max_test_duration = parse_duration(timeout_str)?;
    }

    if verbose {
        println!("Test suite configuration loaded successfully");
        println!("Name: {}", config.name);
        println!("Description: {}", config.description);
        println!("Output directory: {}", config.output_dir.display());
        println!("Run throughput tests: {}", config.run_throughput_tests);
        println!("Run latency tests: {}", config.run_latency_tests);
        println!("Max test duration: {}", format_duration(config.max_test_duration));
        println!("Throughput test configurations: {}", config.test_configs.throughput.len());
        println!("Latency test configurations: {}", config.test_configs.latency.len());
    }

    // Create and run automated test runner
    let mut runner = AutomatedTestRunner::new(config);

    println!("Starting automated test suite...");
    let results = runner.run_test_suite().await?;

    // Display final summary
    display_final_summary(&results, verbose);

    Ok(())
}

fn generate_config(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = matches.get_one::<PathBuf>("output").unwrap();
    let template = matches.get_one::<String>("template").unwrap();
    let validators = *matches.get_one::<usize>("validators").unwrap();
    let endpoints_str = matches.get_one::<String>("endpoints").unwrap();

    let endpoints: Vec<String> = endpoints_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let config = match template.as_str() {
        "minimal" => create_minimal_config(validators, endpoints),
        "comprehensive" => create_comprehensive_config(validators, endpoints),
        "performance" => create_performance_config(validators, endpoints),
        "stability" => create_stability_config(validators, endpoints),
        _ => return Err("Invalid template type".into()),
    };

    let config_content = if output_path.extension() == Some("yaml".as_ref()) {
        serde_yaml::to_string(&config)?
    } else {
        serde_json::to_string_pretty(&config)?
    };

    std::fs::write(output_path, config_content)?;
    println!("Test suite configuration saved to: {}", output_path.display());

    Ok(())
}

fn validate_config(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = matches.get_one::<PathBuf>("config").unwrap();

    let config_content = std::fs::read_to_string(config_path)?;
    let config: TestSuiteConfig = if config_path.extension() == Some("yaml".as_ref()) {
        serde_yaml::from_str(&config_content)?
    } else {
        serde_json::from_str(&config_content)?
    };

    // Validate configuration
    let mut errors = Vec::new();

    if config.name.is_empty() {
        errors.push("Test suite name is required".to_string());
    }

    if config.test_configs.throughput.is_empty() && config.test_configs.latency.is_empty() {
        errors.push("At least one test configuration is required".to_string());
    }

    if config.max_test_duration.as_secs() == 0 {
        errors.push("Maximum test duration must be greater than 0".to_string());
    }

    // Validate throughput configurations
    for (i, throughput_config) in config.test_configs.throughput.iter().enumerate() {
        if throughput_config.name.is_empty() {
            errors.push(format!("Throughput config {} is missing name", i + 1));
        }

        if throughput_config.load.target_tps == 0.0 {
            errors.push(format!("Throughput config {} has invalid target TPS", i + 1));
        }
    }

    // Validate latency configurations
    for (i, latency_config) in config.test_configs.latency.iter().enumerate() {
        if latency_config.network.endpoints.is_empty() {
            errors.push(format!("Latency config {} is missing network endpoints", i + 1));
        }

        if latency_config.samples_per_type == 0 {
            errors.push(format!("Latency config {} has invalid samples per type", i + 1));
        }
    }

    if errors.is_empty() {
        println!("Configuration is valid: {}", config_path.display());

        // Display configuration summary
        println!("\nConfiguration Summary:");
        println!("  Name: {}", config.name);
        println!("  Description: {}", config.description);
        println!("  Throughput tests: {}", config.test_configs.throughput.len());
        println!("  Latency tests: {}", config.test_configs.latency.len());
        println!("  Max test duration: {}", format_duration(config.max_test_duration));
    } else {
        println!("Configuration validation failed:");
        for error in errors {
            println!("  - {}", error);
        }
        std::process::exit(1);
    }

    Ok(())
}

fn analyze_results(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = matches.get_one::<PathBuf>("input").unwrap();
    let format = matches.get_one::<String>("format").unwrap();

    let results_content = std::fs::read_to_string(input_path)?;
    let results: TestSuiteResults = if input_path.extension() == Some("yaml".as_ref()) {
        serde_yaml::from_str(&results_content)?
    } else {
        serde_json::from_str(&results_content)?
    };

    // Generate analysis report
    let analysis = generate_analysis_report(&results, format)?;

    if let Some(output_path) = matches.get_one::<PathBuf>("output") {
        std::fs::write(output_path, analysis)?;
        println!("Analysis report saved to: {}", output_path.display());
    } else {
        println!("{}", analysis);
    }

    // Handle comparison if requested
    if let Some(compare_path) = matches.get_one::<PathBuf>("compare") {
        let compare_content = std::fs::read_to_string(compare_path)?;
        let compare_results: TestSuiteResults = if compare_path.extension() == Some("yaml".as_ref()) {
            serde_yaml::from_str(&compare_content)?
        } else {
            serde_json::from_str(&compare_content)?
        };

        let comparison = generate_comparison_report(&results, &compare_results)?;
        println!("\n=== Comparison Report ===");
        println!("{}", comparison);
    }

    Ok(())
}

fn show_status(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let results_dir = matches.get_one::<PathBuf>("results-dir").unwrap();
    let recent_days = *matches.get_one::<u32>("recent").unwrap();

    if !results_dir.exists() {
        println!("Results directory not found: {}", results_dir.display());
        return Ok(());
    }

    println!("IPC Benchmark Test Suite Status");
    println!("Results directory: {}", results_dir.display());
    println!("Recent results (last {} days):", recent_days);
    println!();

    // Scan for recent results
    let cutoff_time = std::time::SystemTime::now() - std::time::Duration::from_secs(recent_days as u64 * 24 * 3600);

    let mut recent_results = Vec::new();
    for entry in std::fs::read_dir(results_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension() == Some("json".as_ref()) {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified > cutoff_time {
                        recent_results.push((path, modified));
                    }
                }
            }
        }
    }

    recent_results.sort_by(|a, b| b.1.cmp(&a.1));

    if recent_results.is_empty() {
        println!("No recent test results found.");
        return Ok(());
    }

    for (path, modified) in recent_results {
        if let Some(name) = path.file_name() {
            println!("  {} (modified: {:?})", name.to_string_lossy(), modified);

            // Try to load and display brief summary
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(results) = serde_json::from_str::<TestSuiteResults>(&content) {
                    println!("    Duration: {}", format_duration(results.duration));
                    println!("    Success Rate: {:.1}%", results.summary.success_rate * 100.0);
                    println!("    Peak TPS: {:.1}", results.summary.performance_highlights.peak_tps);
                    println!("    Best P95 Latency: {:.1}ms", results.summary.performance_highlights.best_latency_p95);
                }
            }
            println!();
        }
    }

    Ok(())
}

fn create_minimal_config(validators: usize, endpoints: Vec<String>) -> TestSuiteConfig {
    let mut config = create_default_test_suite_config();
    config.name = "Minimal Test Suite".to_string();
    config.description = "Basic throughput and latency testing".to_string();

    // Single throughput test
    let throughput_config = BenchmarkConfig {
        name: "Basic Throughput".to_string(),
        description: "Basic throughput test".to_string(),
        network: NetworkEndpoints { endpoints: endpoints.clone() },
        load: crate::LoadConfig {
            pattern: crate::LoadPattern::Constant,
            target_tps: 500.0,
            duration: Duration::from_secs(300),
            ramp_up_duration: Duration::from_secs(30),
            concurrent_connections: 50,
            start_tps: 0.0,
            max_tps: 0.0,
        },
        transactions: vec![
            crate::TransactionConfig {
                tx_type: TransactionType::Transfer,
                weight: 100,
                amount: 1000000000000000000u64,
                gas_limit: 21000,
                gas_price: 1000000000u64,
                data: Vec::new(),
            }
        ],
        test: crate::TestConfig {
            duration: Duration::from_secs(300),
            warmup_duration: Duration::from_secs(60),
            measurement_duration: Duration::from_secs(180),
            cooldown_duration: Duration::from_secs(60),
            max_failures: 100,
            timeout: Duration::from_secs(30),
        },
        metrics: crate::MetricsConfig {
            collection_interval: Duration::from_secs(1),
            detailed_latency: true,
            resource_monitoring: true,
            percentiles: vec![50.0, 90.0, 95.0, 99.0],
        },
        output: crate::OutputConfig {
            format: "json".to_string(),
            file: "minimal_throughput.json".to_string(),
            detailed: false,
        },
    };

    config.test_configs.throughput.push(throughput_config);

    // Single latency test
    let latency_config = LatencyConfig {
        transaction_types: vec![TransactionType::Transfer],
        samples_per_type: 10,
        test_interval: Duration::from_secs(1),
        confirmation_timeout: Duration::from_secs(30),
        confirmation_depth: 1,
        network: NetworkEndpoints { endpoints },
        duration: Duration::from_secs(300),
        cross_subnet_enabled: false,
    };

    config.test_configs.latency.push(latency_config);

    config
}

fn create_comprehensive_config(validators: usize, endpoints: Vec<String>) -> TestSuiteConfig {
    let mut config = create_default_test_suite_config();
    config.name = "Comprehensive Test Suite".to_string();
    config.description = "Comprehensive performance testing across multiple scenarios".to_string();

    // Multiple throughput tests with different patterns
    let throughput_configs = vec![
        ("Constant Load", crate::LoadPattern::Constant, 1000.0),
        ("Ramp Up", crate::LoadPattern::RampUp, 2000.0),
        ("Burst", crate::LoadPattern::Burst, 1500.0),
    ];

    for (name, pattern, target_tps) in throughput_configs {
        let throughput_config = BenchmarkConfig {
            name: name.to_string(),
            description: format!("{} throughput test", name),
            network: NetworkEndpoints { endpoints: endpoints.clone() },
            load: crate::LoadConfig {
                pattern,
                target_tps,
                duration: Duration::from_secs(600),
                ramp_up_duration: Duration::from_secs(60),
                concurrent_connections: 100,
                start_tps: 100.0,
                max_tps: target_tps,
            },
            transactions: vec![
                crate::TransactionConfig {
                    tx_type: TransactionType::Transfer,
                    weight: 60,
                    amount: 1000000000000000000u64,
                    gas_limit: 21000,
                    gas_price: 1000000000u64,
                    data: Vec::new(),
                },
                crate::TransactionConfig {
                    tx_type: TransactionType::Erc20,
                    weight: 30,
                    amount: 1000000000000000000u64,
                    gas_limit: 65000,
                    gas_price: 1000000000u64,
                    data: Vec::new(),
                },
                crate::TransactionConfig {
                    tx_type: TransactionType::ContractCall,
                    weight: 10,
                    amount: 0u64,
                    gas_limit: 100000,
                    gas_price: 1000000000u64,
                    data: Vec::new(),
                },
            ],
            test: crate::TestConfig {
                duration: Duration::from_secs(600),
                warmup_duration: Duration::from_secs(60),
                measurement_duration: Duration::from_secs(480),
                cooldown_duration: Duration::from_secs(60),
                max_failures: 200,
                timeout: Duration::from_secs(30),
            },
            metrics: crate::MetricsConfig {
                collection_interval: Duration::from_millis(500),
                detailed_latency: true,
                resource_monitoring: true,
                percentiles: vec![50.0, 90.0, 95.0, 99.0, 99.9],
            },
            output: crate::OutputConfig {
                format: "json".to_string(),
                file: format!("{}_throughput.json", name.to_lowercase().replace(' ', "_")),
                detailed: true,
            },
        };

        config.test_configs.throughput.push(throughput_config);
    }

    // Multiple latency tests
    let latency_configs = vec![
        ("Basic Latency", vec![TransactionType::Transfer], 20),
        ("Mixed Latency", vec![TransactionType::Transfer, TransactionType::Erc20, TransactionType::ContractCall], 15),
    ];

    for (name, tx_types, samples) in latency_configs {
        let latency_config = LatencyConfig {
            transaction_types: tx_types,
            samples_per_type: samples,
            test_interval: Duration::from_millis(500),
            confirmation_timeout: Duration::from_secs(60),
            confirmation_depth: 3,
            network: NetworkEndpoints { endpoints: endpoints.clone() },
            duration: Duration::from_secs(600),
            cross_subnet_enabled: false,
        };

        config.test_configs.latency.push(latency_config);
    }

    config
}

fn create_performance_config(validators: usize, endpoints: Vec<String>) -> TestSuiteConfig {
    let mut config = create_default_test_suite_config();
    config.name = "Performance Test Suite".to_string();
    config.description = "High-performance testing to find maximum throughput".to_string();

    // High-performance throughput test
    let throughput_config = BenchmarkConfig {
        name: "High Performance".to_string(),
        description: "Maximum throughput stress test".to_string(),
        network: NetworkEndpoints { endpoints: endpoints.clone() },
        load: crate::LoadConfig {
            pattern: crate::LoadPattern::RampUp,
            target_tps: 5000.0,
            duration: Duration::from_secs(1200),
            ramp_up_duration: Duration::from_secs(120),
            concurrent_connections: 500,
            start_tps: 500.0,
            max_tps: 5000.0,
        },
        transactions: vec![
            crate::TransactionConfig {
                tx_type: TransactionType::Transfer,
                weight: 100,
                amount: 100000000000000000u64,
                gas_limit: 21000,
                gas_price: 1000000000u64,
                data: Vec::new(),
            }
        ],
        test: crate::TestConfig {
            duration: Duration::from_secs(1200),
            warmup_duration: Duration::from_secs(120),
            measurement_duration: Duration::from_secs(960),
            cooldown_duration: Duration::from_secs(120),
            max_failures: 500,
            timeout: Duration::from_secs(60),
        },
        metrics: crate::MetricsConfig {
            collection_interval: Duration::from_millis(200),
            detailed_latency: true,
            resource_monitoring: true,
            percentiles: vec![50.0, 90.0, 95.0, 99.0, 99.9, 99.99],
        },
        output: crate::OutputConfig {
            format: "json".to_string(),
            file: "performance_throughput.json".to_string(),
            detailed: true,
        },
    };

    config.test_configs.throughput.push(throughput_config);

    // High-frequency latency test
    let latency_config = LatencyConfig {
        transaction_types: vec![TransactionType::Transfer],
        samples_per_type: 100,
        test_interval: Duration::from_millis(100),
        confirmation_timeout: Duration::from_secs(30),
        confirmation_depth: 1,
        network: NetworkEndpoints { endpoints },
        duration: Duration::from_secs(600),
        cross_subnet_enabled: false,
    };

    config.test_configs.latency.push(latency_config);

    config
}

fn create_stability_config(validators: usize, endpoints: Vec<String>) -> TestSuiteConfig {
    let mut config = create_default_test_suite_config();
    config.name = "Stability Test Suite".to_string();
    config.description = "Long-running stability testing".to_string();

    // Long-running stability test
    let throughput_config = BenchmarkConfig {
        name: "Stability Test".to_string(),
        description: "Long-running stability test".to_string(),
        network: NetworkEndpoints { endpoints: endpoints.clone() },
        load: crate::LoadConfig {
            pattern: crate::LoadPattern::Constant,
            target_tps: 1000.0,
            duration: Duration::from_secs(3600), // 1 hour
            ramp_up_duration: Duration::from_secs(300),
            concurrent_connections: 200,
            start_tps: 0.0,
            max_tps: 0.0,
        },
        transactions: vec![
            crate::TransactionConfig {
                tx_type: TransactionType::Transfer,
                weight: 70,
                amount: 1000000000000000000u64,
                gas_limit: 21000,
                gas_price: 1000000000u64,
                data: Vec::new(),
            },
            crate::TransactionConfig {
                tx_type: TransactionType::Erc20,
                weight: 30,
                amount: 1000000000000000000u64,
                gas_limit: 65000,
                gas_price: 1000000000u64,
                data: Vec::new(),
            },
        ],
        test: crate::TestConfig {
            duration: Duration::from_secs(3600),
            warmup_duration: Duration::from_secs(300),
            measurement_duration: Duration::from_secs(3000),
            cooldown_duration: Duration::from_secs(300),
            max_failures: 1000,
            timeout: Duration::from_secs(30),
        },
        metrics: crate::MetricsConfig {
            collection_interval: Duration::from_secs(5),
            detailed_latency: true,
            resource_monitoring: true,
            percentiles: vec![50.0, 90.0, 95.0, 99.0],
        },
        output: crate::OutputConfig {
            format: "json".to_string(),
            file: "stability_throughput.json".to_string(),
            detailed: true,
        },
    };

    config.test_configs.throughput.push(throughput_config);

    config
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

fn generate_analysis_report(results: &TestSuiteResults, format: &str) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        "json" => Ok(serde_json::to_string_pretty(results)?),
        "html" => Ok(generate_html_analysis(results)),
        "csv" => Ok(generate_csv_analysis(results)),
        "markdown" => Ok(generate_markdown_analysis(results)),
        _ => Err("Unsupported format".into()),
    }
}

fn generate_html_analysis(results: &TestSuiteResults) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Test Suite Analysis - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background: #f5f5f5; padding: 15px; border-radius: 5px; }}
        .highlight {{ background: #e8f5e8; padding: 10px; margin: 10px 0; border-radius: 3px; }}
        .warning {{ background: #fff3cd; padding: 10px; margin: 10px 0; border-radius: 3px; }}
        table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <h1>Test Suite Analysis: {}</h1>

    <div class="summary">
        <h2>Summary</h2>
        <p><strong>Duration:</strong> {}</p>
        <p><strong>Success Rate:</strong> {:.1}%</p>
        <p><strong>Peak TPS:</strong> {:.1}</p>
        <p><strong>Best P95 Latency:</strong> {:.1}ms</p>
    </div>

    <div class="highlight">
        <h2>Key Findings</h2>
        <p>Most stable configuration: {}</p>
        <p>Best performing transaction type: {:?}</p>
    </div>

    <h2>Recommendations</h2>
    {}

</body>
</html>
        "#,
        results.config.name,
        results.config.name,
        format_duration(results.duration),
        results.summary.success_rate * 100.0,
        results.summary.performance_highlights.peak_tps,
        results.summary.performance_highlights.best_latency_p95,
        results.summary.performance_highlights.most_stable_config,
        results.summary.performance_highlights.best_tx_type,
        results.summary.recommendations.iter()
            .map(|r| format!("<div class=\"warning\">• {}</div>", r))
            .collect::<Vec<_>>()
            .join("")
    )
}

fn generate_csv_analysis(results: &TestSuiteResults) -> String {
    format!(
        "metric,value\n\
        duration_seconds,{}\n\
        success_rate,{:.4}\n\
        peak_tps,{:.2}\n\
        avg_tps,{:.2}\n\
        best_latency_p95_ms,{:.2}\n\
        avg_latency_p95_ms,{:.2}\n\
        total_tests,{}\n\
        successful_tests,{}\n\
        failed_tests,{}\n",
        results.duration.as_secs(),
        results.summary.success_rate,
        results.summary.performance_highlights.peak_tps,
        results.summary.performance_highlights.avg_tps,
        results.summary.performance_highlights.best_latency_p95,
        results.summary.performance_highlights.avg_latency_p95,
        results.summary.total_tests,
        results.summary.successful_tests,
        results.summary.failed_tests
    )
}

fn generate_markdown_analysis(results: &TestSuiteResults) -> String {
    let mut md = String::new();

    md.push_str(&format!("# Test Suite Analysis: {}\n\n", results.config.name));
    md.push_str(&format!("**Description:** {}\n\n", results.config.description));

    md.push_str("## Summary\n\n");
    md.push_str(&format!("- **Duration:** {}\n", format_duration(results.duration)));
    md.push_str(&format!("- **Success Rate:** {:.1}%\n", results.summary.success_rate * 100.0));
    md.push_str(&format!("- **Peak TPS:** {:.1}\n", results.summary.performance_highlights.peak_tps));
    md.push_str(&format!("- **Best P95 Latency:** {:.1}ms\n", results.summary.performance_highlights.best_latency_p95));
    md.push_str("\n");

    md.push_str("## Performance Highlights\n\n");
    md.push_str(&format!("- **Most Stable Config:** {}\n", results.summary.performance_highlights.most_stable_config));
    md.push_str(&format!("- **Best Transaction Type:** {:?}\n", results.summary.performance_highlights.best_tx_type));
    md.push_str("\n");

    if !results.summary.recommendations.is_empty() {
        md.push_str("## Recommendations\n\n");
        for rec in &results.summary.recommendations {
            md.push_str(&format!("- {}\n", rec));
        }
        md.push_str("\n");
    }

    md
}

fn generate_comparison_report(current: &TestSuiteResults, previous: &TestSuiteResults) -> Result<String, Box<dyn std::error::Error>> {
    let mut report = String::new();

    report.push_str("## Performance Comparison\n\n");

    let tps_change = ((current.summary.performance_highlights.peak_tps / previous.summary.performance_highlights.peak_tps) - 1.0) * 100.0;
    let latency_change = ((current.summary.performance_highlights.best_latency_p95 / previous.summary.performance_highlights.best_latency_p95) - 1.0) * 100.0;
    let success_rate_change = (current.summary.success_rate - previous.summary.success_rate) * 100.0;

    report.push_str(&format!("- **Peak TPS Change:** {:.1}%\n", tps_change));
    report.push_str(&format!("- **Latency Change:** {:.1}%\n", latency_change));
    report.push_str(&format!("- **Success Rate Change:** {:.1}%\n", success_rate_change));

    if tps_change > 5.0 {
        report.push_str("✅ Significant throughput improvement\n");
    } else if tps_change < -5.0 {
        report.push_str("⚠️ Throughput degradation detected\n");
    }

    if latency_change < -5.0 {
        report.push_str("✅ Latency improvement detected\n");
    } else if latency_change > 5.0 {
        report.push_str("⚠️ Latency degradation detected\n");
    }

    Ok(report)
}

fn display_final_summary(results: &TestSuiteResults, verbose: bool) {
    println!("\n=== Test Suite Complete ===");
    println!("Suite: {}", results.config.name);
    println!("Duration: {}", format_duration(results.duration));
    println!("Success Rate: {:.1}%", results.summary.success_rate * 100.0);
    println!();

    println!("Performance Highlights:");
    println!("  Peak TPS: {:.1}", results.summary.performance_highlights.peak_tps);
    println!("  Average TPS: {:.1}", results.summary.performance_highlights.avg_tps);
    println!("  Best P95 Latency: {:.1}ms", results.summary.performance_highlights.best_latency_p95);
    println!("  Average P95 Latency: {:.1}ms", results.summary.performance_highlights.avg_latency_p95);
    println!("  Most Stable Config: {}", results.summary.performance_highlights.most_stable_config);
    println!();

    if !results.summary.recommendations.is_empty() {
        println!("Recommendations:");
        for rec in &results.summary.recommendations {
            println!("  • {}", rec);
        }
        println!();
    }

    if verbose {
        println!("Test Results Summary:");
        println!("  Throughput Tests: {}", results.throughput_results.len());
        println!("  Latency Tests: {}", results.latency_results.len());
        println!("  Total Tests: {}", results.summary.total_tests);
        println!("  Successful Tests: {}", results.summary.successful_tests);
        println!("  Failed Tests: {}", results.summary.failed_tests);
        println!();
    }

    println!("Results saved to: {}", results.config.output_dir.display());
}