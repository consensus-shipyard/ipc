// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Throughput benchmarking tools for IPC subnets
//!
//! This crate provides comprehensive benchmarking capabilities for measuring
//! transaction throughput, latency, and resource utilization in IPC subnets.
//!
//! # Features
//!
//! - **Load Generation**: Configurable transaction generators for different workloads
//! - **Metrics Collection**: Real-time performance monitoring and analysis
//! - **Multi-Validator Testing**: Support for testing with various validator configurations
//! - **Transaction Types**: Support for transfers, contract calls, and cross-subnet messages
//! - **Stress Testing**: Automated stress testing to find performance limits
//!
//! # Usage
//!
//! ```rust
//! use fendermint_benchmarks_throughput::{BenchmarkConfig, BenchmarkRunner};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = BenchmarkConfig::builder()
//!         .validators(4)
//!         .transaction_type(TransactionType::Transfer)
//!         .duration(std::time::Duration::from_secs(300))
//!         .concurrent_users(100)
//!         .target_tps(1000)
//!         .build();
//!
//!     let runner = BenchmarkRunner::new(config).await?;
//!     let results = runner.run().await?;
//!
//!     println!("Average TPS: {:.2}", results.average_tps);
//!     println!("P95 Latency: {:.2}ms", results.latency_p95);
//!
//!     Ok(())
//! }
//! ```

use std::time::Duration;
use serde::{Deserialize, Serialize};
use anyhow::Result;

pub mod config;
pub mod latency;
pub mod load_generator;
pub mod metrics;
pub mod runner;
pub mod simple_benchmark;
pub mod test_runner;
pub mod transaction_types;
pub mod utils;

pub use config::*;
pub use latency::*;
pub use load_generator::*;
pub use metrics::*;
pub use runner::*;
pub use simple_benchmark::*;
pub use test_runner::*;
pub use transaction_types::*;
pub use utils::*;

/// Transaction types supported by the benchmark
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TransactionType {
    /// Simple FIL transfers between accounts
    Transfer,
    /// ERC-20 token transfers
    Erc20,
    /// Contract deployments
    Deploy,
    /// Contract function calls
    ContractCall,
    /// Cross-subnet messages
    CrossSubnet,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Transfer => write!(f, "transfer"),
            TransactionType::Erc20 => write!(f, "erc20"),
            TransactionType::Deploy => write!(f, "deploy"),
            TransactionType::ContractCall => write!(f, "contract_call"),
            TransactionType::CrossSubnet => write!(f, "cross_subnet"),
        }
    }
}

/// Load pattern for transaction generation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoadPattern {
    /// Constant load throughout the test
    Constant,
    /// Gradually increase load over time
    RampUp,
    /// Burst patterns with peaks and valleys
    Burst,
    /// Periodic load patterns
    Periodic,
}

/// Performance metrics collected during benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    /// Test configuration that produced these results
    pub config: BenchmarkConfig,
    /// Duration of the actual test run
    pub duration: Duration,
    /// Total number of transactions sent
    pub total_transactions: u64,
    /// Total number of successful transactions
    pub successful_transactions: u64,
    /// Total number of failed transactions
    pub failed_transactions: u64,
    /// Average transactions per second
    pub average_tps: f64,
    /// Peak transactions per second
    pub peak_tps: f64,
    /// Minimum transactions per second
    pub min_tps: f64,
    /// Transaction latency percentiles (in milliseconds)
    pub latency_p50: f64,
    pub latency_p90: f64,
    pub latency_p95: f64,
    pub latency_p99: f64,
    /// Resource utilization metrics
    pub resource_metrics: ResourceMetrics,
    /// Time series data for detailed analysis
    pub time_series: Vec<TimeSeriesPoint>,
    /// Error breakdown
    pub error_breakdown: std::collections::HashMap<String, u64>,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Average CPU usage percentage per validator
    pub avg_cpu_usage: f64,
    /// Peak CPU usage percentage
    pub peak_cpu_usage: f64,
    /// Average memory usage in MB per validator
    pub avg_memory_usage: f64,
    /// Peak memory usage in MB
    pub peak_memory_usage: f64,
    /// Network bandwidth usage in MB/s
    pub network_bandwidth: f64,
    /// Disk I/O operations per second
    pub disk_iops: f64,
}

/// Time series data point for detailed analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// Timestamp since test start (in seconds)
    pub timestamp: f64,
    /// Transactions per second at this point
    pub tps: f64,
    /// Average latency at this point (in milliseconds)
    pub latency: f64,
    /// CPU usage at this point
    pub cpu_usage: f64,
    /// Memory usage at this point
    pub memory_usage: f64,
    /// Number of errors at this point
    pub errors: u64,
}

/// Default configuration values
pub const DEFAULT_VALIDATORS: usize = 4;
pub const DEFAULT_DURATION: Duration = Duration::from_secs(300);
pub const DEFAULT_CONCURRENT_USERS: usize = 100;
pub const DEFAULT_TARGET_TPS: u64 = 1000;
pub const DEFAULT_RAMP_UP_DURATION: Duration = Duration::from_secs(60);
pub const DEFAULT_BLOCK_TIME: Duration = Duration::from_secs(1);

/// Common error types for benchmarking
#[derive(Debug, thiserror::Error)]
pub enum BenchmarkError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Network error: {0}")]
    Network(#[from] anyhow::Error),
    #[error("Transaction error: {0}")]
    Transaction(String),
    #[error("Timeout error: {0}")]
    Timeout(String),
    #[error("Resource error: {0}")]
    Resource(String),
}

/// Result type for benchmarking operations
pub type BenchmarkResult<T> = Result<T, BenchmarkError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_type_display() {
        assert_eq!(TransactionType::Transfer.to_string(), "transfer");
        assert_eq!(TransactionType::Erc20.to_string(), "erc20");
        assert_eq!(TransactionType::Deploy.to_string(), "deploy");
        assert_eq!(TransactionType::ContractCall.to_string(), "contract_call");
        assert_eq!(TransactionType::CrossSubnet.to_string(), "cross_subnet");
    }

    #[test]
    fn test_default_values() {
        assert_eq!(DEFAULT_VALIDATORS, 4);
        assert_eq!(DEFAULT_DURATION, Duration::from_secs(300));
        assert_eq!(DEFAULT_CONCURRENT_USERS, 100);
        assert_eq!(DEFAULT_TARGET_TPS, 1000);
    }
}