// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Configuration management for throughput benchmarking

use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::{
    BenchmarkError, BenchmarkResult, LoadPattern, TransactionType,
    DEFAULT_BLOCK_TIME, DEFAULT_CONCURRENT_USERS, DEFAULT_DURATION,
    DEFAULT_RAMP_UP_DURATION, DEFAULT_TARGET_TPS, DEFAULT_VALIDATORS
};

/// Configuration for benchmark runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of validators in the test network
    pub validators: usize,
    /// Type of transactions to generate
    pub transaction_type: TransactionType,
    /// Duration of the benchmark test
    pub duration: Duration,
    /// Number of concurrent users/connections
    pub concurrent_users: usize,
    /// Target transactions per second to achieve
    pub target_tps: u64,
    /// Load pattern to use during testing
    pub load_pattern: LoadPattern,
    /// Duration for ramp-up phase (if applicable)
    pub ramp_up_duration: Duration,
    /// Block time for the test network
    pub block_time: Duration,
    /// Enable detailed metrics collection
    pub detailed_metrics: bool,
    /// Warmup duration before starting measurements
    pub warmup_duration: Duration,
    /// Timeout for individual transactions
    pub transaction_timeout: Duration,
    /// Enable resource monitoring
    pub resource_monitoring: bool,
    /// Network endpoints for the test
    pub network_endpoints: NetworkEndpoints,
    /// Transaction generator configuration
    pub transaction_config: TransactionConfig,
}

/// Network endpoints configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEndpoints {
    /// Ethereum JSON-RPC endpoint
    pub eth_rpc_url: String,
    /// CometBFT RPC endpoint
    pub cometbft_rpc_url: String,
    /// Prometheus metrics endpoint (if available)
    pub metrics_url: Option<String>,
}

/// Transaction generator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionConfig {
    /// Gas limit for transactions
    pub gas_limit: u64,
    /// Gas price for transactions
    pub gas_price: u64,
    /// Value to transfer (for transfer transactions)
    pub transfer_value: u64,
    /// Contract address (for contract calls)
    pub contract_address: Option<String>,
    /// Contract ABI (for contract calls)
    pub contract_abi: Option<String>,
    /// Number of accounts to use for testing
    pub num_accounts: usize,
    /// Initial balance for test accounts
    pub initial_balance: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            validators: DEFAULT_VALIDATORS,
            transaction_type: TransactionType::Transfer,
            duration: DEFAULT_DURATION,
            concurrent_users: DEFAULT_CONCURRENT_USERS,
            target_tps: DEFAULT_TARGET_TPS,
            load_pattern: LoadPattern::Constant,
            ramp_up_duration: DEFAULT_RAMP_UP_DURATION,
            block_time: DEFAULT_BLOCK_TIME,
            detailed_metrics: true,
            warmup_duration: Duration::from_secs(30),
            transaction_timeout: Duration::from_secs(10),
            resource_monitoring: true,
            network_endpoints: NetworkEndpoints::default(),
            transaction_config: TransactionConfig::default(),
        }
    }
}

impl Default for NetworkEndpoints {
    fn default() -> Self {
        Self {
            eth_rpc_url: "http://localhost:8545".to_string(),
            cometbft_rpc_url: "http://localhost:26657".to_string(),
            metrics_url: Some("http://localhost:9184/metrics".to_string()),
        }
    }
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            gas_limit: 21000,
            gas_price: 1000000000, // 1 gwei
            transfer_value: 1000000000000000000, // 1 ETH in wei
            contract_address: None,
            contract_abi: None,
            num_accounts: 1000,
            initial_balance: 1000000000000000000000, // 1000 ETH in wei
        }
    }
}

/// Builder for BenchmarkConfig
pub struct BenchmarkConfigBuilder {
    config: BenchmarkConfig,
}

impl BenchmarkConfigBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            config: BenchmarkConfig::default(),
        }
    }

    /// Set the number of validators
    pub fn validators(mut self, validators: usize) -> Self {
        self.config.validators = validators;
        self
    }

    /// Set the transaction type
    pub fn transaction_type(mut self, transaction_type: TransactionType) -> Self {
        self.config.transaction_type = transaction_type;
        self
    }

    /// Set the test duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.config.duration = duration;
        self
    }

    /// Set the number of concurrent users
    pub fn concurrent_users(mut self, concurrent_users: usize) -> Self {
        self.config.concurrent_users = concurrent_users;
        self
    }

    /// Set the target TPS
    pub fn target_tps(mut self, target_tps: u64) -> Self {
        self.config.target_tps = target_tps;
        self
    }

    /// Set the load pattern
    pub fn load_pattern(mut self, load_pattern: LoadPattern) -> Self {
        self.config.load_pattern = load_pattern;
        self
    }

    /// Set the ramp-up duration
    pub fn ramp_up_duration(mut self, duration: Duration) -> Self {
        self.config.ramp_up_duration = duration;
        self
    }

    /// Set the block time
    pub fn block_time(mut self, block_time: Duration) -> Self {
        self.config.block_time = block_time;
        self
    }

    /// Enable or disable detailed metrics
    pub fn detailed_metrics(mut self, enabled: bool) -> Self {
        self.config.detailed_metrics = enabled;
        self
    }

    /// Set the warmup duration
    pub fn warmup_duration(mut self, duration: Duration) -> Self {
        self.config.warmup_duration = duration;
        self
    }

    /// Set the transaction timeout
    pub fn transaction_timeout(mut self, timeout: Duration) -> Self {
        self.config.transaction_timeout = timeout;
        self
    }

    /// Enable or disable resource monitoring
    pub fn resource_monitoring(mut self, enabled: bool) -> Self {
        self.config.resource_monitoring = enabled;
        self
    }

    /// Set the network endpoints
    pub fn network_endpoints(mut self, endpoints: NetworkEndpoints) -> Self {
        self.config.network_endpoints = endpoints;
        self
    }

    /// Set the transaction configuration
    pub fn transaction_config(mut self, config: TransactionConfig) -> Self {
        self.config.transaction_config = config;
        self
    }

    /// Build the configuration, validating it
    pub fn build(self) -> BenchmarkResult<BenchmarkConfig> {
        self.validate()?;
        Ok(self.config)
    }

    /// Validate the configuration
    fn validate(&self) -> BenchmarkResult<()> {
        if self.config.validators == 0 {
            return Err(BenchmarkError::Configuration(
                "Number of validators must be greater than 0".to_string(),
            ));
        }

        if self.config.validators > 100 {
            return Err(BenchmarkError::Configuration(
                "Number of validators should not exceed 100 for realistic testing".to_string(),
            ));
        }

        if self.config.duration.as_secs() == 0 {
            return Err(BenchmarkError::Configuration(
                "Test duration must be greater than 0".to_string(),
            ));
        }

        if self.config.concurrent_users == 0 {
            return Err(BenchmarkError::Configuration(
                "Number of concurrent users must be greater than 0".to_string(),
            ));
        }

        if self.config.target_tps == 0 {
            return Err(BenchmarkError::Configuration(
                "Target TPS must be greater than 0".to_string(),
            ));
        }

        if self.config.transaction_config.num_accounts == 0 {
            return Err(BenchmarkError::Configuration(
                "Number of accounts must be greater than 0".to_string(),
            ));
        }

        if self.config.transaction_config.gas_limit == 0 {
            return Err(BenchmarkError::Configuration(
                "Gas limit must be greater than 0".to_string(),
            ));
        }

        // Validate URL formats
        if let Err(e) = url::Url::parse(&self.config.network_endpoints.eth_rpc_url) {
            return Err(BenchmarkError::Configuration(
                format!("Invalid ETH RPC URL: {}", e),
            ));
        }

        if let Err(e) = url::Url::parse(&self.config.network_endpoints.cometbft_rpc_url) {
            return Err(BenchmarkError::Configuration(
                format!("Invalid CometBFT RPC URL: {}", e),
            ));
        }

        Ok(())
    }
}

impl BenchmarkConfig {
    /// Create a new config builder
    pub fn builder() -> BenchmarkConfigBuilder {
        BenchmarkConfigBuilder::new()
    }

    /// Load configuration from a YAML file
    pub fn from_yaml_file(path: &str) -> BenchmarkResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| BenchmarkError::Configuration(format!("Failed to read config file: {}", e)))?;

        let config: BenchmarkConfig = serde_yaml::from_str(&content)
            .map_err(|e| BenchmarkError::Configuration(format!("Failed to parse config: {}", e)))?;

        // Validate the loaded configuration
        let builder = BenchmarkConfigBuilder { config };
        builder.validate()?;
        Ok(builder.config)
    }

    /// Save configuration to a YAML file
    pub fn to_yaml_file(&self, path: &str) -> BenchmarkResult<()> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| BenchmarkError::Configuration(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)
            .map_err(|e| BenchmarkError::Configuration(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Get the expected transaction rate based on load pattern and time
    pub fn get_expected_tps(&self, elapsed_time: Duration) -> u64 {
        match self.load_pattern {
            LoadPattern::Constant => self.target_tps,
            LoadPattern::RampUp => {
                if elapsed_time < self.ramp_up_duration {
                    let progress = elapsed_time.as_secs_f64() / self.ramp_up_duration.as_secs_f64();
                    (self.target_tps as f64 * progress) as u64
                } else {
                    self.target_tps
                }
            }
            LoadPattern::Burst => {
                // Simple burst pattern: 50% of time at target TPS, 50% at 150% of target TPS
                let cycle_duration = 60.0; // 60 second cycles
                let cycle_position = (elapsed_time.as_secs_f64() % cycle_duration) / cycle_duration;
                if cycle_position < 0.5 {
                    self.target_tps
                } else {
                    (self.target_tps as f64 * 1.5) as u64
                }
            }
            LoadPattern::Periodic => {
                // Sine wave pattern
                let cycle_duration = 120.0; // 2 minute cycles
                let amplitude = self.target_tps as f64 * 0.5;
                let base = self.target_tps as f64 * 0.5;
                let angle = 2.0 * std::f64::consts::PI * elapsed_time.as_secs_f64() / cycle_duration;
                (base + amplitude * angle.sin()) as u64
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.validators, DEFAULT_VALIDATORS);
        assert_eq!(config.duration, DEFAULT_DURATION);
        assert_eq!(config.concurrent_users, DEFAULT_CONCURRENT_USERS);
        assert_eq!(config.target_tps, DEFAULT_TARGET_TPS);
    }

    #[test]
    fn test_config_builder() {
        let config = BenchmarkConfig::builder()
            .validators(7)
            .transaction_type(TransactionType::Erc20)
            .duration(Duration::from_secs(600))
            .concurrent_users(200)
            .target_tps(2000)
            .build()
            .unwrap();

        assert_eq!(config.validators, 7);
        assert_eq!(config.transaction_type, TransactionType::Erc20);
        assert_eq!(config.duration, Duration::from_secs(600));
        assert_eq!(config.concurrent_users, 200);
        assert_eq!(config.target_tps, 2000);
    }

    #[test]
    fn test_config_validation() {
        // Test invalid validator count
        let result = BenchmarkConfig::builder()
            .validators(0)
            .build();
        assert!(result.is_err());

        // Test invalid duration
        let result = BenchmarkConfig::builder()
            .duration(Duration::from_secs(0))
            .build();
        assert!(result.is_err());

        // Test invalid concurrent users
        let result = BenchmarkConfig::builder()
            .concurrent_users(0)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_patterns() {
        let config = BenchmarkConfig::builder()
            .target_tps(1000)
            .load_pattern(LoadPattern::Constant)
            .build()
            .unwrap();

        assert_eq!(config.get_expected_tps(Duration::from_secs(0)), 1000);
        assert_eq!(config.get_expected_tps(Duration::from_secs(100)), 1000);

        let config = BenchmarkConfig::builder()
            .target_tps(1000)
            .load_pattern(LoadPattern::RampUp)
            .ramp_up_duration(Duration::from_secs(60))
            .build()
            .unwrap();

        assert_eq!(config.get_expected_tps(Duration::from_secs(0)), 0);
        assert_eq!(config.get_expected_tps(Duration::from_secs(30)), 500);
        assert_eq!(config.get_expected_tps(Duration::from_secs(60)), 1000);
        assert_eq!(config.get_expected_tps(Duration::from_secs(120)), 1000);
    }
}