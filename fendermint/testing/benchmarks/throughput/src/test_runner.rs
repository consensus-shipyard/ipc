//! Automated Test Runner
//!
//! This module provides a comprehensive test runner that orchestrates
//! both throughput and latency tests with configuration management,
//! result aggregation, and automated reporting.

use crate::{
    BenchmarkConfig, BenchmarkRunner, BenchmarkResults, BenchmarkSuite,
    LatencyConfig, LatencyTestRunner, LatencyTestResults,
    TransactionType, format_duration, format_number,
};

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use tokio::time::sleep;

/// Test suite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteConfig {
    /// Test suite name
    pub name: String,
    /// Test suite description
    pub description: String,
    /// Output directory for results
    pub output_dir: PathBuf,
    /// Whether to run throughput tests
    pub run_throughput_tests: bool,
    /// Whether to run latency tests
    pub run_latency_tests: bool,
    /// Delay between different test types
    pub inter_test_delay: Duration,
    /// Maximum test duration before timeout
    pub max_test_duration: Duration,
    /// Test configurations
    pub test_configs: TestConfigs,
    /// Reporting configuration
    pub reporting: ReportingConfig,
    /// Network cleanup configuration
    pub cleanup: CleanupConfig,
}

/// Test configurations for different test types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfigs {
    /// Throughput test configurations
    pub throughput: Vec<BenchmarkConfig>,
    /// Latency test configurations
    pub latency: Vec<LatencyConfig>,
}

/// Reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    /// Generate JSON reports
    pub json: bool,
    /// Generate HTML reports
    pub html: bool,
    /// Generate CSV reports
    pub csv: bool,
    /// Generate comparison charts
    pub charts: bool,
    /// Email notifications configuration
    pub email: Option<EmailConfig>,
    /// Slack notifications configuration
    pub slack: Option<SlackConfig>,
}

/// Email notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP server
    pub smtp_server: String,
    /// SMTP port
    pub smtp_port: u16,
    /// Email username
    pub username: String,
    /// Email password
    pub password: String,
    /// Recipients
    pub recipients: Vec<String>,
}

/// Slack notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Webhook URL
    pub webhook_url: String,
    /// Channel name
    pub channel: String,
    /// Notification level (all, failures, summary)
    pub level: String,
}

/// Cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Clean up test networks after tests
    pub cleanup_networks: bool,
    /// Clean up old results
    pub cleanup_old_results: bool,
    /// Days to keep results
    pub keep_results_days: u32,
    /// Clean up temporary files
    pub cleanup_temp_files: bool,
}

/// Comprehensive test suite results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResults {
    /// Test suite configuration
    pub config: TestSuiteConfig,
    /// Test start time
    pub start_time: SystemTime,
    /// Test end time
    pub end_time: SystemTime,
    /// Total duration
    pub duration: Duration,
    /// Throughput test results
    pub throughput_results: Vec<BenchmarkResults>,
    /// Latency test results
    pub latency_results: Vec<LatencyTestResults>,
    /// Overall summary
    pub summary: TestSuiteSummary,
    /// Test execution log
    pub execution_log: Vec<TestExecutionEntry>,
}

/// Test suite summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteSummary {
    /// Total tests run
    pub total_tests: usize,
    /// Successful tests
    pub successful_tests: usize,
    /// Failed tests
    pub failed_tests: usize,
    /// Success rate
    pub success_rate: f64,
    /// Performance highlights
    pub performance_highlights: PerformanceHighlights,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Performance highlights from the test suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHighlights {
    /// Peak TPS achieved
    pub peak_tps: f64,
    /// Average TPS across all tests
    pub avg_tps: f64,
    /// Best latency (P95)
    pub best_latency_p95: f64,
    /// Average latency (P95)
    pub avg_latency_p95: f64,
    /// Most stable configuration
    pub most_stable_config: String,
    /// Best performing transaction type
    pub best_tx_type: TransactionType,
}

/// Test execution log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionEntry {
    /// Timestamp
    pub timestamp: SystemTime,
    /// Test name
    pub test_name: String,
    /// Test type
    pub test_type: String,
    /// Status
    pub status: String,
    /// Duration
    pub duration: Option<Duration>,
    /// Message
    pub message: String,
}

/// Automated test runner
pub struct AutomatedTestRunner {
    config: TestSuiteConfig,
    results: TestSuiteResults,
    execution_log: Vec<TestExecutionEntry>,
}

impl AutomatedTestRunner {
    /// Create a new automated test runner
    pub fn new(config: TestSuiteConfig) -> Self {
        let results = TestSuiteResults {
            config: config.clone(),
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
            duration: Duration::from_secs(0),
            throughput_results: Vec::new(),
            latency_results: Vec::new(),
            summary: TestSuiteSummary {
                total_tests: 0,
                successful_tests: 0,
                failed_tests: 0,
                success_rate: 0.0,
                performance_highlights: PerformanceHighlights {
                    peak_tps: 0.0,
                    avg_tps: 0.0,
                    best_latency_p95: 0.0,
                    avg_latency_p95: 0.0,
                    most_stable_config: String::new(),
                    best_tx_type: TransactionType::Transfer,
                },
                recommendations: Vec::new(),
            },
            execution_log: Vec::new(),
        };

        Self {
            config,
            results,
            execution_log: Vec::new(),
        }
    }

    /// Run the complete test suite
    pub async fn run_test_suite(&mut self) -> Result<TestSuiteResults, Box<dyn std::error::Error>> {
        println!("Starting automated test suite: {}", self.config.name);

        self.log_execution("test_suite", "starting", "Test suite started");

        // Create output directory
        std::fs::create_dir_all(&self.config.output_dir)?;

        // Run throughput tests
        if self.config.run_throughput_tests {
            self.run_throughput_tests().await?;
        }

        // Delay between test types
        if self.config.run_throughput_tests && self.config.run_latency_tests {
            println!("Waiting {} between test types...", format_duration(self.config.inter_test_delay));
            sleep(self.config.inter_test_delay).await;
        }

        // Run latency tests
        if self.config.run_latency_tests {
            self.run_latency_tests().await?;
        }

        // Finalize results
        self.results.end_time = SystemTime::now();
        self.results.duration = self.results.end_time
            .duration_since(self.results.start_time)
            .unwrap_or_default();

        self.results.execution_log = self.execution_log.clone();

        // Generate summary
        self.generate_summary();

        // Generate reports
        self.generate_reports().await?;

        // Send notifications
        self.send_notifications().await?;

        // Cleanup
        self.cleanup().await?;

        self.log_execution("test_suite", "completed", "Test suite completed successfully");

        println!("Test suite completed in {}", format_duration(self.results.duration));

        Ok(self.results.clone())
    }

    /// Run throughput tests
    async fn run_throughput_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running throughput tests...");

        for (i, config) in self.config.test_configs.throughput.iter().enumerate() {
            let test_name = format!("throughput_test_{}", i + 1);

            self.log_execution(&test_name, "starting", &format!("Starting throughput test: {}", config.name));

            let test_start = SystemTime::now();

            match BenchmarkRunner::new(config.clone()) {
                Ok(mut runner) => {
                    match tokio::time::timeout(self.config.max_test_duration, runner.run_benchmark()).await {
                        Ok(Ok(result)) => {
                            let duration = SystemTime::now().duration_since(test_start).unwrap_or_default();
                            self.results.throughput_results.push(result);
                            self.log_execution(&test_name, "completed", &format!("Throughput test completed in {}", format_duration(duration)));
                        }
                        Ok(Err(e)) => {
                            self.log_execution(&test_name, "failed", &format!("Throughput test failed: {}", e));
                        }
                        Err(_) => {
                            self.log_execution(&test_name, "timeout", "Throughput test timed out");
                        }
                    }
                }
                Err(e) => {
                    self.log_execution(&test_name, "failed", &format!("Failed to create throughput runner: {}", e));
                }
            }

            // Delay between tests
            if i < self.config.test_configs.throughput.len() - 1 {
                println!("Waiting {} before next test...", format_duration(self.config.inter_test_delay));
                sleep(self.config.inter_test_delay).await;
            }
        }

        Ok(())
    }

    /// Run latency tests
    async fn run_latency_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running latency tests...");

        for (i, config) in self.config.test_configs.latency.iter().enumerate() {
            let test_name = format!("latency_test_{}", i + 1);

            self.log_execution(&test_name, "starting", "Starting latency test");

            let test_start = SystemTime::now();

            match LatencyTestRunner::new(config.clone()) {
                Ok(mut runner) => {
                    match tokio::time::timeout(self.config.max_test_duration, runner.run_test()).await {
                        Ok(Ok(result)) => {
                            let duration = SystemTime::now().duration_since(test_start).unwrap_or_default();
                            self.results.latency_results.push(result);
                            self.log_execution(&test_name, "completed", &format!("Latency test completed in {}", format_duration(duration)));
                        }
                        Ok(Err(e)) => {
                            self.log_execution(&test_name, "failed", &format!("Latency test failed: {}", e));
                        }
                        Err(_) => {
                            self.log_execution(&test_name, "timeout", "Latency test timed out");
                        }
                    }
                }
                Err(e) => {
                    self.log_execution(&test_name, "failed", &format!("Failed to create latency runner: {}", e));
                }
            }

            // Delay between tests
            if i < self.config.test_configs.latency.len() - 1 {
                println!("Waiting {} before next test...", format_duration(self.config.inter_test_delay));
                sleep(self.config.inter_test_delay).await;
            }
        }

        Ok(())
    }

    /// Generate test suite summary
    fn generate_summary(&mut self) {
        let total_tests = self.results.throughput_results.len() + self.results.latency_results.len();
        let successful_tests = self.execution_log.iter()
            .filter(|entry| entry.status == "completed")
            .count();
        let failed_tests = total_tests - successful_tests;

        let success_rate = if total_tests > 0 {
            successful_tests as f64 / total_tests as f64
        } else {
            0.0
        };

        // Calculate performance highlights
        let mut peak_tps = 0.0;
        let mut avg_tps = 0.0;
        let mut tps_count = 0;

        for result in &self.results.throughput_results {
            if let Some(max_tps) = result.throughput_stats.tps_stats.iter()
                .map(|ts| ts.tps)
                .max_by(|a, b| a.partial_cmp(b).unwrap()) {
                peak_tps = peak_tps.max(max_tps);
            }

            let result_avg = result.throughput_stats.tps_stats.iter()
                .map(|ts| ts.tps)
                .sum::<f64>() / result.throughput_stats.tps_stats.len() as f64;

            avg_tps += result_avg;
            tps_count += 1;
        }

        if tps_count > 0 {
            avg_tps /= tps_count as f64;
        }

        let mut best_latency_p95 = f64::MAX;
        let mut avg_latency_p95 = 0.0;
        let mut latency_count = 0;

        for result in &self.results.latency_results {
            if let Some(p95) = result.stats.end_to_end.percentiles.get("P95") {
                best_latency_p95 = best_latency_p95.min(*p95);
                avg_latency_p95 += p95;
                latency_count += 1;
            }
        }

        if latency_count > 0 {
            avg_latency_p95 /= latency_count as f64;
        } else {
            best_latency_p95 = 0.0;
        }

        // Generate recommendations
        let mut recommendations = Vec::new();

        if success_rate < 0.8 {
            recommendations.push("Consider investigating test failures - success rate is below 80%".to_string());
        }

        if peak_tps < 1000.0 {
            recommendations.push("Peak TPS is below 1000 - consider optimizing network configuration".to_string());
        }

        if best_latency_p95 > 5000.0 {
            recommendations.push("P95 latency is above 5 seconds - investigate network congestion".to_string());
        }

        if avg_tps > 0.0 && peak_tps / avg_tps > 3.0 {
            recommendations.push("High TPS variance detected - consider load balancing improvements".to_string());
        }

        // Find most stable configuration (lowest coefficient of variation)
        let mut most_stable_config = String::new();
        let mut lowest_cv = f64::MAX;

        for result in &self.results.throughput_results {
            if !result.throughput_stats.tps_stats.is_empty() {
                let tps_values: Vec<f64> = result.throughput_stats.tps_stats.iter()
                    .map(|ts| ts.tps)
                    .collect();

                let mean = tps_values.iter().sum::<f64>() / tps_values.len() as f64;
                let variance = tps_values.iter()
                    .map(|&x| (x - mean).powi(2))
                    .sum::<f64>() / tps_values.len() as f64;
                let std_dev = variance.sqrt();
                let cv = std_dev / mean;

                if cv < lowest_cv {
                    lowest_cv = cv;
                    most_stable_config = result.config.name.clone();
                }
            }
        }

        self.results.summary = TestSuiteSummary {
            total_tests,
            successful_tests,
            failed_tests,
            success_rate,
            performance_highlights: PerformanceHighlights {
                peak_tps,
                avg_tps,
                best_latency_p95,
                avg_latency_p95,
                most_stable_config,
                best_tx_type: TransactionType::Transfer, // TODO: Calculate from results
            },
            recommendations,
        };
    }

    /// Generate reports
    async fn generate_reports(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Generating reports...");

        // JSON report
        if self.config.reporting.json {
            let json_path = self.config.output_dir.join("test_suite_results.json");
            let json_content = serde_json::to_string_pretty(&self.results)?;
            std::fs::write(&json_path, json_content)?;
            println!("JSON report saved to: {}", json_path.display());
        }

        // HTML report
        if self.config.reporting.html {
            let html_path = self.config.output_dir.join("test_suite_report.html");
            let html_content = self.generate_html_report();
            std::fs::write(&html_path, html_content)?;
            println!("HTML report saved to: {}", html_path.display());
        }

        // CSV report
        if self.config.reporting.csv {
            let csv_path = self.config.output_dir.join("test_suite_results.csv");
            let csv_content = self.generate_csv_report();
            std::fs::write(&csv_path, csv_content)?;
            println!("CSV report saved to: {}", csv_path.display());
        }

        Ok(())
    }

    /// Generate HTML report
    fn generate_html_report(&self) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html><head><title>IPC Benchmark Test Suite Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; margin: 20px 0; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("th { background-color: #f2f2f2; }\n");
        html.push_str(".summary { background-color: #f9f9f9; padding: 15px; margin: 20px 0; }\n");
        html.push_str(".highlight { background-color: #e6f3ff; padding: 10px; margin: 10px 0; }\n");
        html.push_str(".recommendation { background-color: #fff3cd; padding: 10px; margin: 5px 0; }\n");
        html.push_str("</style>\n");
        html.push_str("</head><body>\n");

        html.push_str(&format!("<h1>IPC Benchmark Test Suite Report</h1>\n"));
        html.push_str(&format!("<h2>{}</h2>\n", self.config.name));
        html.push_str(&format!("<p>{}</p>\n", self.config.description));

        // Summary section
        html.push_str("<div class=\"summary\">\n");
        html.push_str("<h3>Test Suite Summary</h3>\n");
        html.push_str(&format!("<p><strong>Duration:</strong> {}</p>\n", format_duration(self.results.duration)));
        html.push_str(&format!("<p><strong>Total Tests:</strong> {}</p>\n", self.results.summary.total_tests));
        html.push_str(&format!("<p><strong>Successful Tests:</strong> {}</p>\n", self.results.summary.successful_tests));
        html.push_str(&format!("<p><strong>Failed Tests:</strong> {}</p>\n", self.results.summary.failed_tests));
        html.push_str(&format!("<p><strong>Success Rate:</strong> {:.2}%</p>\n", self.results.summary.success_rate * 100.0));
        html.push_str("</div>\n");

        // Performance highlights
        html.push_str("<div class=\"highlight\">\n");
        html.push_str("<h3>Performance Highlights</h3>\n");
        html.push_str(&format!("<p><strong>Peak TPS:</strong> {:.2}</p>\n", self.results.summary.performance_highlights.peak_tps));
        html.push_str(&format!("<p><strong>Average TPS:</strong> {:.2}</p>\n", self.results.summary.performance_highlights.avg_tps));
        html.push_str(&format!("<p><strong>Best P95 Latency:</strong> {:.2}ms</p>\n", self.results.summary.performance_highlights.best_latency_p95));
        html.push_str(&format!("<p><strong>Average P95 Latency:</strong> {:.2}ms</p>\n", self.results.summary.performance_highlights.avg_latency_p95));
        html.push_str(&format!("<p><strong>Most Stable Config:</strong> {}</p>\n", self.results.summary.performance_highlights.most_stable_config));
        html.push_str("</div>\n");

        // Recommendations
        if !self.results.summary.recommendations.is_empty() {
            html.push_str("<h3>Recommendations</h3>\n");
            for recommendation in &self.results.summary.recommendations {
                html.push_str(&format!("<div class=\"recommendation\">• {}</div>\n", recommendation));
            }
        }

        // Throughput results table
        if !self.results.throughput_results.is_empty() {
            html.push_str("<h3>Throughput Test Results</h3>\n");
            html.push_str("<table>\n");
            html.push_str("<tr><th>Test Name</th><th>Peak TPS</th><th>Avg TPS</th><th>Total Transactions</th><th>Success Rate</th></tr>\n");

            for result in &self.results.throughput_results {
                let peak_tps = result.throughput_stats.tps_stats.iter()
                    .map(|ts| ts.tps)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);

                let avg_tps = if !result.throughput_stats.tps_stats.is_empty() {
                    result.throughput_stats.tps_stats.iter()
                        .map(|ts| ts.tps)
                        .sum::<f64>() / result.throughput_stats.tps_stats.len() as f64
                } else {
                    0.0
                };

                html.push_str(&format!(
                    "<tr><td>{}</td><td>{:.2}</td><td>{:.2}</td><td>{}</td><td>{:.2}%</td></tr>\n",
                    result.config.name,
                    peak_tps,
                    avg_tps,
                    result.total_transactions,
                    result.success_rate * 100.0
                ));
            }

            html.push_str("</table>\n");
        }

        // Latency results table
        if !self.results.latency_results.is_empty() {
            html.push_str("<h3>Latency Test Results</h3>\n");
            html.push_str("<table>\n");
            html.push_str("<tr><th>Test</th><th>Total Transactions</th><th>Success Rate</th><th>Avg Latency (ms)</th><th>P95 Latency (ms)</th><th>P99 Latency (ms)</th></tr>\n");

            for (i, result) in self.results.latency_results.iter().enumerate() {
                html.push_str(&format!(
                    "<tr><td>Latency Test {}</td><td>{}</td><td>{:.2}%</td><td>{:.2}</td><td>{:.2}</td><td>{:.2}</td></tr>\n",
                    i + 1,
                    result.total_transactions,
                    result.success_rate * 100.0,
                    result.stats.end_to_end.avg_ms,
                    result.stats.end_to_end.percentiles.get("P95").unwrap_or(&0.0),
                    result.stats.end_to_end.percentiles.get("P99").unwrap_or(&0.0)
                ));
            }

            html.push_str("</table>\n");
        }

        html.push_str("</body></html>\n");
        html
    }

    /// Generate CSV report
    fn generate_csv_report(&self) -> String {
        let mut csv = String::new();

        // Header
        csv.push_str("test_type,test_name,peak_tps,avg_tps,total_transactions,success_rate,avg_latency_ms,p95_latency_ms,p99_latency_ms\n");

        // Throughput results
        for result in &self.results.throughput_results {
            let peak_tps = result.throughput_stats.tps_stats.iter()
                .map(|ts| ts.tps)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);

            let avg_tps = if !result.throughput_stats.tps_stats.is_empty() {
                result.throughput_stats.tps_stats.iter()
                    .map(|ts| ts.tps)
                    .sum::<f64>() / result.throughput_stats.tps_stats.len() as f64
            } else {
                0.0
            };

            csv.push_str(&format!(
                "throughput,{},{:.2},{:.2},{},{:.4},,,\n",
                result.config.name,
                peak_tps,
                avg_tps,
                result.total_transactions,
                result.success_rate
            ));
        }

        // Latency results
        for (i, result) in self.results.latency_results.iter().enumerate() {
            csv.push_str(&format!(
                "latency,latency_test_{},,,{},{:.4},{:.2},{:.2},{:.2}\n",
                i + 1,
                result.total_transactions,
                result.success_rate,
                result.stats.end_to_end.avg_ms,
                result.stats.end_to_end.percentiles.get("P95").unwrap_or(&0.0),
                result.stats.end_to_end.percentiles.get("P99").unwrap_or(&0.0)
            ));
        }

        csv
    }

    /// Send notifications
    async fn send_notifications(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Email notifications
        if let Some(_email_config) = &self.config.reporting.email {
            // TODO: Implement email notifications
            println!("Email notifications would be sent here");
        }

        // Slack notifications
        if let Some(_slack_config) = &self.config.reporting.slack {
            // TODO: Implement Slack notifications
            println!("Slack notifications would be sent here");
        }

        Ok(())
    }

    /// Cleanup resources
    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.cleanup.cleanup_networks {
            println!("Cleaning up test networks...");
            // TODO: Implement network cleanup
        }

        if self.config.cleanup.cleanup_old_results {
            println!("Cleaning up old results...");
            // TODO: Implement old results cleanup
        }

        if self.config.cleanup.cleanup_temp_files {
            println!("Cleaning up temporary files...");
            // TODO: Implement temp files cleanup
        }

        Ok(())
    }

    /// Log execution entry
    fn log_execution(&mut self, test_name: &str, status: &str, message: &str) {
        let entry = TestExecutionEntry {
            timestamp: SystemTime::now(),
            test_name: test_name.to_string(),
            test_type: if test_name.contains("throughput") { "throughput" } else { "latency" }.to_string(),
            status: status.to_string(),
            duration: None,
            message: message.to_string(),
        };

        self.execution_log.push(entry);
        println!("[{}] {}: {}", test_name, status, message);
    }
}

/// Create a default test suite configuration
pub fn create_default_test_suite_config() -> TestSuiteConfig {
    TestSuiteConfig {
        name: "IPC Benchmark Test Suite".to_string(),
        description: "Comprehensive performance testing for IPC subnets".to_string(),
        output_dir: PathBuf::from("./benchmark_results"),
        run_throughput_tests: true,
        run_latency_tests: true,
        inter_test_delay: Duration::from_secs(60),
        max_test_duration: Duration::from_secs(3600), // 1 hour
        test_configs: TestConfigs {
            throughput: Vec::new(),
            latency: Vec::new(),
        },
        reporting: ReportingConfig {
            json: true,
            html: true,
            csv: true,
            charts: true,
            email: None,
            slack: None,
        },
        cleanup: CleanupConfig {
            cleanup_networks: true,
            cleanup_old_results: true,
            keep_results_days: 7,
            cleanup_temp_files: true,
        },
    }
}