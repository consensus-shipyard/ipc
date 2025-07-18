// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Metrics collection for benchmarking

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};
use serde::{Deserialize, Serialize};
use statrs::statistics::{OrderStatistics, Statistics};
use tokio::time::interval;
use tracing::{debug, info, warn};

use crate::{BenchmarkResults, ResourceMetrics, TimeSeriesPoint, TransactionResult};

/// Real-time metrics collector
pub struct MetricsCollector {
    start_time: Instant,
    transaction_results: Vec<TransactionResult>,
    resource_samples: Vec<ResourceSample>,
    collection_interval: Duration,
}

/// Resource usage sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSample {
    pub timestamp: SystemTime,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub disk_reads: u64,
    pub disk_writes: u64,
}

/// Transaction latency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub min: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub median: Duration,
    pub p90: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub std_dev: Duration,
}

/// TPS statistics over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpsStats {
    pub average: f64,
    pub peak: f64,
    pub minimum: f64,
    pub time_series: Vec<TpsPoint>,
}

/// TPS measurement point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpsPoint {
    pub timestamp: f64,
    pub tps: f64,
    pub window_size: Duration,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(collection_interval: Duration) -> Self {
        Self {
            start_time: Instant::now(),
            transaction_results: Vec::new(),
            resource_samples: Vec::new(),
            collection_interval,
        }
    }

    /// Start collecting metrics
    pub async fn start_collection(&mut self) {
        let mut interval = interval(self.collection_interval);

        info!("Starting metrics collection with interval: {:?}", self.collection_interval);

        loop {
            interval.tick().await;
            self.collect_resource_sample().await;
        }
    }

    /// Add a transaction result to the collection
    pub fn add_transaction_result(&mut self, result: TransactionResult) {
        self.transaction_results.push(result);
    }

    /// Add multiple transaction results
    pub fn add_transaction_results(&mut self, results: Vec<TransactionResult>) {
        self.transaction_results.extend(results);
    }

    /// Collect a resource usage sample
    async fn collect_resource_sample(&mut self) {
        let sample = ResourceSample {
            timestamp: Instant::now(),
            cpu_usage: self.get_cpu_usage().await,
            memory_usage: self.get_memory_usage().await,
            network_rx: self.get_network_rx().await,
            network_tx: self.get_network_tx().await,
            disk_reads: self.get_disk_reads().await,
            disk_writes: self.get_disk_writes().await,
        };

        self.resource_samples.push(sample);
    }

    /// Calculate latency statistics from transaction results
    pub fn calculate_latency_stats(&self) -> LatencyStats {
        let successful_results: Vec<_> = self.transaction_results
            .iter()
            .filter(|r| r.success)
            .collect();

        if successful_results.is_empty() {
            return LatencyStats {
                min: Duration::from_secs(0),
                max: Duration::from_secs(0),
                mean: Duration::from_secs(0),
                median: Duration::from_secs(0),
                p90: Duration::from_secs(0),
                p95: Duration::from_secs(0),
                p99: Duration::from_secs(0),
                std_dev: Duration::from_secs(0),
            };
        }

        let mut latencies: Vec<f64> = successful_results
            .iter()
            .map(|r| r.latency.as_secs_f64() * 1000.0) // Convert to milliseconds
            .collect();

        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = Duration::from_secs_f64(latencies[0] / 1000.0);
        let max = Duration::from_secs_f64(latencies[latencies.len() - 1] / 1000.0);
        let mean = Duration::from_secs_f64(latencies.mean() / 1000.0);
        let median = Duration::from_secs_f64(crate::utils::calculate_percentile(&latencies, 50.0) / 1000.0);
        let p90 = Duration::from_secs_f64(crate::utils::calculate_percentile(&latencies, 90.0) / 1000.0);
        let p95 = Duration::from_secs_f64(crate::utils::calculate_percentile(&latencies, 95.0) / 1000.0);
        let p99 = Duration::from_secs_f64(crate::utils::calculate_percentile(&latencies, 99.0) / 1000.0);
        let std_dev = Duration::from_secs_f64(latencies.std_dev() / 1000.0);

        LatencyStats {
            min,
            max,
            mean,
            median,
            p90,
            p95,
            p99,
            std_dev,
        }
    }

    /// Calculate TPS statistics
    pub fn calculate_tps_stats(&self, window_size: Duration) -> TpsStats {
        let mut tps_points = Vec::new();
        let total_duration = self.start_time.elapsed();

        // Calculate TPS in sliding windows
        let mut current_time = Duration::from_secs(0);
        while current_time < total_duration {
            let window_start = current_time;
            let window_end = current_time + window_size;

            let transactions_in_window = self.transaction_results
                .iter()
                .filter(|r| {
                    let tx_time = r.timestamp.duration_since(self.start_time);
                    tx_time >= window_start && tx_time < window_end && r.success
                })
                .count();

            let tps = transactions_in_window as f64 / window_size.as_secs_f64();

            tps_points.push(TpsPoint {
                timestamp: current_time.as_secs_f64(),
                tps,
                window_size,
            });

            current_time += window_size;
        }

        let tps_values: Vec<f64> = tps_points.iter().map(|p| p.tps).collect();
        let average = if !tps_values.is_empty() { tps_values.iter().sum::<f64>() / tps_values.len() as f64 } else { 0.0 };
        let peak = tps_values.iter().fold(0.0f64, |acc, &x| acc.max(x));
        let minimum = tps_values.iter().fold(f64::INFINITY, |acc, &x| acc.min(x));

        TpsStats {
            average,
            peak,
            minimum: if minimum == f64::INFINITY { 0.0 } else { minimum },
            time_series: tps_points,
        }
    }

    /// Calculate resource metrics
    pub fn calculate_resource_metrics(&self) -> ResourceMetrics {
        if self.resource_samples.is_empty() {
            return ResourceMetrics {
                avg_cpu_usage: 0.0,
                peak_cpu_usage: 0.0,
                avg_memory_usage: 0.0,
                peak_memory_usage: 0.0,
                network_bandwidth: 0.0,
                disk_iops: 0.0,
            };
        }

        let cpu_values: Vec<f64> = self.resource_samples.iter().map(|s| s.cpu_usage).collect();
        let memory_values: Vec<f64> = self.resource_samples.iter().map(|s| s.memory_usage).collect();

        let avg_cpu_usage = cpu_values.iter().sum::<f64>() / cpu_values.len() as f64;
        let peak_cpu_usage = cpu_values.iter().fold(0.0f64, |acc, &x| acc.max(x));
        let avg_memory_usage = memory_values.iter().sum::<f64>() / memory_values.len() as f64;
        let peak_memory_usage = memory_values.iter().fold(0.0f64, |acc, &x| acc.max(x));

        // Calculate network bandwidth (simplified)
        let network_bandwidth = self.calculate_network_bandwidth();
        let disk_iops = self.calculate_disk_iops();

        ResourceMetrics {
            avg_cpu_usage,
            peak_cpu_usage,
            avg_memory_usage,
            peak_memory_usage,
            network_bandwidth,
            disk_iops,
        }
    }

    /// Generate time series data for detailed analysis
    pub fn generate_time_series(&self, interval: Duration) -> Vec<TimeSeriesPoint> {
        let mut time_series = Vec::new();
        let total_duration = self.start_time.elapsed();

        let mut current_time = Duration::from_secs(0);
        while current_time < total_duration {
            let window_start = current_time;
            let window_end = current_time + interval;

            // Calculate TPS for this window
            let transactions_in_window = self.transaction_results
                .iter()
                .filter(|r| {
                    let tx_time = r.timestamp.duration_since(self.start_time);
                    tx_time >= window_start && tx_time < window_end && r.success
                })
                .count();

            let tps = transactions_in_window as f64 / interval.as_secs_f64();

            // Calculate average latency for this window
            let latencies_in_window: Vec<Duration> = self.transaction_results
                .iter()
                .filter_map(|r| {
                    let tx_time = r.timestamp.duration_since(self.start_time);
                    if tx_time >= window_start && tx_time < window_end && r.success {
                        Some(r.latency)
                    } else {
                        None
                    }
                })
                .collect();

            let latency = if !latencies_in_window.is_empty() {
                latencies_in_window.iter().sum::<Duration>().as_secs_f64() * 1000.0 / latencies_in_window.len() as f64
            } else {
                0.0
            };

            // Find resource sample closest to this time
            let resource_sample = self.resource_samples
                .iter()
                .min_by_key(|s| {
                    let sample_time = s.timestamp.duration_since(self.start_time);
                    if sample_time >= window_start && sample_time < window_end {
                        Duration::from_secs(0)
                    } else {
                        Duration::from_secs(u64::MAX)
                    }
                });

            let (cpu_usage, memory_usage) = if let Some(sample) = resource_sample {
                (sample.cpu_usage, sample.memory_usage)
            } else {
                (0.0, 0.0)
            };

            // Count errors in this window
            let errors = self.transaction_results
                .iter()
                .filter(|r| {
                    let tx_time = r.timestamp.duration_since(self.start_time);
                    tx_time >= window_start && tx_time < window_end && !r.success
                })
                .count() as u64;

            time_series.push(TimeSeriesPoint {
                timestamp: current_time.as_secs_f64(),
                tps,
                latency,
                cpu_usage,
                memory_usage,
                errors,
            });

            current_time += interval;
        }

        time_series
    }

    /// Calculate error breakdown
    pub fn calculate_error_breakdown(&self) -> HashMap<String, u64> {
        let mut error_counts = HashMap::new();

        for result in &self.transaction_results {
            if !result.success {
                if let Some(error) = &result.error {
                    *error_counts.entry(error.clone()).or_insert(0) += 1;
                } else {
                    *error_counts.entry("Unknown error".to_string()).or_insert(0) += 1;
                }
            }
        }

        error_counts
    }

    // Platform-specific resource monitoring methods
    // These would be implemented with actual system calls in a real implementation

    async fn get_cpu_usage(&self) -> f64 {
        // Mock implementation - in reality would use system calls
        rand::random::<f64>() * 100.0
    }

    async fn get_memory_usage(&self) -> f64 {
        // Mock implementation - in reality would use system calls
        rand::random::<f64>() * 1000.0 // MB
    }

    async fn get_network_rx(&self) -> u64 {
        // Mock implementation
        rand::random::<u64>() % 1000000
    }

    async fn get_network_tx(&self) -> u64 {
        // Mock implementation
        rand::random::<u64>() % 1000000
    }

    async fn get_disk_reads(&self) -> u64 {
        // Mock implementation
        rand::random::<u64>() % 10000
    }

    async fn get_disk_writes(&self) -> u64 {
        // Mock implementation
        rand::random::<u64>() % 10000
    }

    fn calculate_network_bandwidth(&self) -> f64 {
        // Mock implementation
        rand::random::<f64>() * 100.0 // MB/s
    }

    fn calculate_disk_iops(&self) -> f64 {
        // Mock implementation
        rand::random::<f64>() * 1000.0 // IOPS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransactionType;

    #[test]
    fn test_latency_stats_calculation() {
        let mut collector = MetricsCollector::new(Duration::from_secs(1));

        let results = vec![
            TransactionResult {
                tx_hash: Some(ethers::types::H256::random()),
                success: true,
                latency: Duration::from_millis(100),
                error: None,
                tx_type: TransactionType::Transfer,
                timestamp: Instant::now(),
            },
            TransactionResult {
                tx_hash: Some(ethers::types::H256::random()),
                success: true,
                latency: Duration::from_millis(200),
                error: None,
                tx_type: TransactionType::Transfer,
                timestamp: Instant::now(),
            },
            TransactionResult {
                tx_hash: None,
                success: false,
                latency: Duration::from_millis(300),
                error: Some("Test error".to_string()),
                tx_type: TransactionType::Transfer,
                timestamp: Instant::now(),
            },
        ];

        collector.add_transaction_results(results);
        let stats = collector.calculate_latency_stats();

        assert_eq!(stats.min, Duration::from_millis(100));
        assert_eq!(stats.max, Duration::from_millis(200));
        assert_eq!(stats.mean, Duration::from_millis(150));
    }

    #[test]
    fn test_error_breakdown() {
        let mut collector = MetricsCollector::new(Duration::from_secs(1));

        let results = vec![
            TransactionResult {
                tx_hash: None,
                success: false,
                latency: Duration::from_millis(100),
                error: Some("Network error".to_string()),
                tx_type: TransactionType::Transfer,
                timestamp: Instant::now(),
            },
            TransactionResult {
                tx_hash: None,
                success: false,
                latency: Duration::from_millis(200),
                error: Some("Network error".to_string()),
                tx_type: TransactionType::Transfer,
                timestamp: Instant::now(),
            },
            TransactionResult {
                tx_hash: None,
                success: false,
                latency: Duration::from_millis(300),
                error: Some("Gas error".to_string()),
                tx_type: TransactionType::Transfer,
                timestamp: Instant::now(),
            },
        ];

        collector.add_transaction_results(results);
        let breakdown = collector.calculate_error_breakdown();

        assert_eq!(breakdown.get("Network error"), Some(&2));
        assert_eq!(breakdown.get("Gas error"), Some(&1));
    }
}