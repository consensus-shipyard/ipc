// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Benchmark runner that orchestrates the entire benchmarking process

use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use tokio::time::sleep;
use tracing::{info, warn, error};

use crate::{
    BenchmarkConfig, BenchmarkResults, BenchmarkError, BenchmarkResult,
    LoadGenerator, MetricsCollector, TransactionResult, ResourceMetrics,
    TimeSeriesPoint,
};

/// Main benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    load_generator: LoadGenerator,
    metrics_collector: MetricsCollector,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub async fn new(config: BenchmarkConfig) -> BenchmarkResult<Self> {
        info!("Initializing benchmark runner with config: {:?}", config);

        let load_generator = LoadGenerator::new(config.clone()).await
            .map_err(|e| BenchmarkError::Configuration(format!("Failed to create load generator: {}", e)))?;

        let metrics_collector = MetricsCollector::new(Duration::from_secs(1));

        Ok(Self {
            config,
            load_generator,
            metrics_collector,
        })
    }

    /// Run the benchmark
    pub async fn run(&mut self) -> BenchmarkResult<BenchmarkResults> {
        info!("Starting benchmark run");
        let start_time = Instant::now();

        // Phase 1: Warmup
        if self.config.warmup_duration > Duration::from_secs(0) {
            info!("Starting warmup phase for {:?}", self.config.warmup_duration);
            self.run_warmup().await?;
        }

        // Phase 2: Start metrics collection
        let metrics_handle = {
            let mut collector = self.metrics_collector.clone();
            tokio::spawn(async move {
                collector.start_collection().await;
            })
        };

        // Phase 3: Main benchmark
        info!("Starting main benchmark phase for {:?}", self.config.duration);
        let transaction_results = self.run_main_benchmark().await?;

        // Phase 4: Stop metrics collection
        metrics_handle.abort();

        // Phase 5: Process results
        let results = self.process_results(transaction_results, start_time.elapsed()).await?;

        info!("Benchmark completed successfully");
        Ok(results)
    }

    /// Run warmup phase
    async fn run_warmup(&mut self) -> BenchmarkResult<()> {
        let warmup_config = BenchmarkConfig {
            duration: self.config.warmup_duration,
            target_tps: self.config.target_tps / 2, // Half TPS for warmup
            ..self.config.clone()
        };

        let warmup_generator = LoadGenerator::new(warmup_config).await
            .map_err(|e| BenchmarkError::Configuration(format!("Failed to create warmup generator: {}", e)))?;

        let _warmup_results = warmup_generator.generate_load(self.config.warmup_duration).await
            .map_err(|e| BenchmarkError::Network(e))?;

        info!("Warmup phase completed");
        Ok(())
    }

    /// Run the main benchmark
    async fn run_main_benchmark(&mut self) -> BenchmarkResult<Vec<TransactionResult>> {
        let results = self.load_generator.generate_load(self.config.duration).await
            .map_err(|e| BenchmarkError::Network(e))?;

        // Add results to metrics collector
        self.metrics_collector.add_transaction_results(results.clone());

        Ok(results)
    }

    /// Process benchmark results
    async fn process_results(
        &mut self,
        transaction_results: Vec<TransactionResult>,
        total_duration: Duration,
    ) -> BenchmarkResult<BenchmarkResults> {
        info!("Processing benchmark results");

        let total_transactions = transaction_results.len() as u64;
        let successful_transactions = transaction_results.iter().filter(|r| r.success).count() as u64;
        let failed_transactions = total_transactions - successful_transactions;

        let average_tps = successful_transactions as f64 / self.config.duration.as_secs_f64();

        // Calculate TPS statistics
        let tps_stats = self.metrics_collector.calculate_tps_stats(Duration::from_secs(10));
        let peak_tps = tps_stats.peak;
        let min_tps = tps_stats.minimum;

        // Calculate latency statistics
        let latency_stats = self.metrics_collector.calculate_latency_stats();

        // Calculate resource metrics
        let resource_metrics = self.metrics_collector.calculate_resource_metrics();

        // Generate time series data
        let time_series = self.metrics_collector.generate_time_series(Duration::from_secs(5));

        // Calculate error breakdown
        let error_breakdown = self.metrics_collector.calculate_error_breakdown();

        let results = BenchmarkResults {
            config: self.config.clone(),
            duration: total_duration,
            total_transactions,
            successful_transactions,
            failed_transactions,
            average_tps,
            peak_tps,
            min_tps,
            latency_p50: latency_stats.median.as_secs_f64() * 1000.0,
            latency_p90: latency_stats.p90.as_secs_f64() * 1000.0,
            latency_p95: latency_stats.p95.as_secs_f64() * 1000.0,
            latency_p99: latency_stats.p99.as_secs_f64() * 1000.0,
            resource_metrics,
            time_series,
            error_breakdown,
        };

        self.log_results_summary(&results);

        Ok(results)
    }

    /// Log a summary of the benchmark results
    fn log_results_summary(&self, results: &BenchmarkResults) {
        info!("=== BENCHMARK RESULTS SUMMARY ===");
        info!("Configuration: {} validators, {} concurrent users",
              results.config.validators, results.config.concurrent_users);
        info!("Transaction type: {}", results.config.transaction_type);
        info!("Duration: {:?}", results.duration);
        info!("Total transactions: {}", results.total_transactions);
        info!("Successful transactions: {}", results.successful_transactions);
        info!("Failed transactions: {}", results.failed_transactions);
        info!("Success rate: {:.2}%",
              (results.successful_transactions as f64 / results.total_transactions as f64) * 100.0);
        info!("Average TPS: {:.2}", results.average_tps);
        info!("Peak TPS: {:.2}", results.peak_tps);
        info!("Latency P50: {:.2}ms", results.latency_p50);
        info!("Latency P95: {:.2}ms", results.latency_p95);
        info!("Latency P99: {:.2}ms", results.latency_p99);
        info!("Average CPU usage: {:.2}%", results.resource_metrics.avg_cpu_usage);
        info!("Peak CPU usage: {:.2}%", results.resource_metrics.peak_cpu_usage);
        info!("Average memory usage: {:.2}MB", results.resource_metrics.avg_memory_usage);

        if !results.error_breakdown.is_empty() {
            info!("Error breakdown:");
            for (error, count) in &results.error_breakdown {
                info!("  {}: {}", error, count);
            }
        }

        info!("=== END SUMMARY ===");
    }

    /// Save results to file
    pub fn save_results(&self, results: &BenchmarkResults, path: &str) -> BenchmarkResult<()> {
        let json = serde_json::to_string_pretty(results)
            .map_err(|e| BenchmarkError::Configuration(format!("Failed to serialize results: {}", e)))?;

        std::fs::write(path, json)
            .map_err(|e| BenchmarkError::Configuration(format!("Failed to write results file: {}", e)))?;

        info!("Results saved to: {}", path);
        Ok(())
    }

    /// Load results from file
    pub fn load_results(path: &str) -> BenchmarkResult<BenchmarkResults> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| BenchmarkError::Configuration(format!("Failed to read results file: {}", e)))?;

        let results: BenchmarkResults = serde_json::from_str(&content)
            .map_err(|e| BenchmarkError::Configuration(format!("Failed to parse results: {}", e)))?;

        Ok(results)
    }
}

/// Stress test runner for finding performance limits
pub struct StressTestRunner {
    base_config: BenchmarkConfig,
    max_tps: u64,
    step_size: u64,
    step_duration: Duration,
}

impl StressTestRunner {
    /// Create a new stress test runner
    pub fn new(base_config: BenchmarkConfig, max_tps: u64, step_size: u64) -> Self {
        Self {
            base_config,
            max_tps,
            step_size,
            step_duration: Duration::from_secs(60), // 1 minute per step
        }
    }

    /// Run stress test to find maximum sustainable TPS
    pub async fn find_max_tps(&self) -> BenchmarkResult<BenchmarkResults> {
        info!("Starting stress test to find maximum TPS");
        let mut current_tps = self.step_size;
        let mut best_results = None;
        let mut last_successful_tps = 0;

        while current_tps <= self.max_tps {
            info!("Testing TPS: {}", current_tps);

            let mut config = self.base_config.clone();
            config.target_tps = current_tps;
            config.duration = self.step_duration;

            let mut runner = BenchmarkRunner::new(config).await?;
            let results = runner.run().await?;

            // Check if this TPS level is sustainable
            let success_rate = results.successful_transactions as f64 / results.total_transactions as f64;
            let achieved_tps = results.average_tps;

            info!("Results for {} TPS: {:.2} achieved TPS, {:.2}% success rate",
                  current_tps, achieved_tps, success_rate * 100.0);

            if success_rate >= 0.95 && achieved_tps >= current_tps as f64 * 0.9 {
                // This TPS level is sustainable
                last_successful_tps = current_tps;
                best_results = Some(results);
                current_tps += self.step_size;
            } else {
                // This TPS level is not sustainable
                info!("TPS {} is not sustainable, stopping stress test", current_tps);
                break;
            }
        }

        if let Some(results) = best_results {
            info!("Maximum sustainable TPS found: {}", last_successful_tps);
            Ok(results)
        } else {
            Err(BenchmarkError::Configuration(
                "No sustainable TPS level found".to_string(),
            ))
        }
    }
}

/// Benchmark suite runner for comprehensive testing
pub struct BenchmarkSuite {
    base_config: BenchmarkConfig,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(base_config: BenchmarkConfig) -> Self {
        Self { base_config }
    }

    /// Run the complete benchmark suite
    pub async fn run_complete_suite(&self) -> BenchmarkResult<Vec<BenchmarkResults>> {
        info!("Starting complete benchmark suite");
        let mut all_results = Vec::new();

        // Test different validator counts
        for validators in [1, 4, 7, 10] {
            let mut config = self.base_config.clone();
            config.validators = validators;

            info!("Running test with {} validators", validators);
            let mut runner = BenchmarkRunner::new(config).await?;
            let results = runner.run().await?;
            all_results.push(results);
        }

        // Test different transaction types
        use crate::TransactionType;
        for tx_type in [TransactionType::Transfer, TransactionType::Erc20, TransactionType::Deploy] {
            let mut config = self.base_config.clone();
            config.transaction_type = tx_type;

            info!("Running test with transaction type: {}", tx_type);
            let mut runner = BenchmarkRunner::new(config).await?;
            let results = runner.run().await?;
            all_results.push(results);
        }

        info!("Benchmark suite completed with {} test runs", all_results.len());
        Ok(all_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BenchmarkConfig;

    #[tokio::test]
    async fn test_benchmark_runner_creation() {
        let config = BenchmarkConfig::default();

        // This will fail without a real network, but tests the structure
        let result = BenchmarkRunner::new(config).await;
        assert!(result.is_err()); // Expected to fail in test environment
    }

    #[test]
    fn test_stress_test_runner_creation() {
        let config = BenchmarkConfig::default();
        let stress_runner = StressTestRunner::new(config, 10000, 500);

        assert_eq!(stress_runner.max_tps, 10000);
        assert_eq!(stress_runner.step_size, 500);
        assert_eq!(stress_runner.step_duration, Duration::from_secs(60));
    }

    #[test]
    fn test_benchmark_suite_creation() {
        let config = BenchmarkConfig::default();
        let suite = BenchmarkSuite::new(config);

        assert_eq!(suite.base_config.validators, 4);
    }
}