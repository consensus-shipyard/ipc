//! Latency Testing Module
//!
//! This module provides comprehensive latency measurement capabilities for IPC subnets.
//! It measures various types of latency including:
//! - End-to-end transaction latency
//! - Block confirmation latency
//! - Network propagation latency
//! - Cross-subnet message latency

use crate::{
    BenchmarkResults, NetworkEndpoints, TransactionType,
    utils::{format_duration, calculate_percentile}
};

use ethers::{
    core::types::{TransactionRequest, H256},
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::LocalWallet,
    types::{Address, U256},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::time::{interval, timeout};

/// Latency measurement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConfig {
    /// Transaction types to test
    pub transaction_types: Vec<TransactionType>,
    /// Number of test transactions per type
    pub samples_per_type: usize,
    /// Interval between test transactions
    pub test_interval: Duration,
    /// Maximum timeout for transaction confirmation
    pub confirmation_timeout: Duration,
    /// Block confirmation depth to measure
    pub confirmation_depth: u64,
    /// Network endpoints configuration
    pub network: NetworkEndpoints,
    /// Test duration
    pub duration: Duration,
    /// Enable cross-subnet latency testing
    pub cross_subnet_enabled: bool,
}

/// Individual latency measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMeasurement {
    /// Transaction hash
    pub tx_hash: H256,
    /// Transaction type
    pub tx_type: TransactionType,
    /// Timestamp when transaction was submitted
    pub submitted_at: SystemTime,
    /// Timestamp when transaction was included in mempool
    pub mempool_at: Option<SystemTime>,
    /// Timestamp when transaction was included in block
    pub block_at: Option<SystemTime>,
    /// Timestamp when transaction reached confirmation depth
    pub confirmed_at: Option<SystemTime>,
    /// Block number where transaction was included
    pub block_number: Option<u64>,
    /// Gas used by transaction
    pub gas_used: Option<u64>,
    /// Gas price used
    pub gas_price: Option<U256>,
    /// Network endpoint used
    pub endpoint: String,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Comprehensive latency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    /// End-to-end latency (submission to confirmation)
    pub end_to_end: LatencyMetrics,
    /// Mempool latency (submission to mempool inclusion)
    pub mempool: LatencyMetrics,
    /// Block inclusion latency (mempool to block)
    pub block_inclusion: LatencyMetrics,
    /// Confirmation latency (block to confirmation depth)
    pub confirmation: LatencyMetrics,
    /// Network propagation latency
    pub network_propagation: LatencyMetrics,
    /// Statistics per transaction type
    pub by_transaction_type: HashMap<TransactionType, LatencyMetrics>,
    /// Statistics per network endpoint
    pub by_endpoint: HashMap<String, LatencyMetrics>,
}

/// Latency metrics for a specific measurement type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    /// Number of samples
    pub samples: usize,
    /// Average latency in milliseconds
    pub avg_ms: f64,
    /// Minimum latency in milliseconds
    pub min_ms: f64,
    /// Maximum latency in milliseconds
    pub max_ms: f64,
    /// Standard deviation
    pub std_dev_ms: f64,
    /// Percentile measurements
    pub percentiles: HashMap<String, f64>,
}

/// Latency test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyTestResults {
    /// Test configuration
    pub config: LatencyConfig,
    /// Test start time
    pub start_time: SystemTime,
    /// Test duration
    pub duration: Duration,
    /// All individual measurements
    pub measurements: Vec<LatencyMeasurement>,
    /// Comprehensive statistics
    pub stats: LatencyStats,
    /// Total transactions tested
    pub total_transactions: usize,
    /// Successful transactions
    pub successful_transactions: usize,
    /// Failed transactions
    pub failed_transactions: usize,
    /// Success rate
    pub success_rate: f64,
    /// Network health during test
    pub network_health: NetworkHealthMetrics,
}

/// Network health metrics during latency testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealthMetrics {
    /// Average block time
    pub avg_block_time_ms: f64,
    /// Block time standard deviation
    pub block_time_std_dev_ms: f64,
    /// Network congestion indicators
    pub congestion_indicators: CongestionMetrics,
    /// Validator performance metrics
    pub validator_metrics: Vec<ValidatorMetrics>,
}

/// Network congestion metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CongestionMetrics {
    /// Average gas price during test
    pub avg_gas_price: U256,
    /// Gas price volatility
    pub gas_price_volatility: f64,
    /// Mempool size fluctuation
    pub mempool_size_stats: LatencyMetrics,
    /// Transaction queue depth
    pub queue_depth_stats: LatencyMetrics,
}

/// Individual validator performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorMetrics {
    /// Validator address/identifier
    pub validator_id: String,
    /// Endpoint URL
    pub endpoint: String,
    /// Response time statistics
    pub response_time: LatencyMetrics,
    /// Block proposal frequency
    pub proposal_frequency: f64,
    /// Missed proposals
    pub missed_proposals: u64,
    /// Sync status
    pub sync_status: bool,
}

/// Latency test runner
pub struct LatencyTestRunner {
    config: LatencyConfig,
    providers: Vec<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    measurements: Vec<LatencyMeasurement>,
    start_time: SystemTime,
}

impl LatencyTestRunner {
    /// Create a new latency test runner
    pub fn new(config: LatencyConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let providers = Self::setup_providers(&config.network)?;

        Ok(Self {
            config,
            providers,
            measurements: Vec::new(),
            start_time: SystemTime::now(),
        })
    }

    /// Setup providers for all network endpoints
    fn setup_providers(
        network: &NetworkEndpoints,
    ) -> Result<Vec<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>, Box<dyn std::error::Error>> {
        let mut providers = Vec::new();

        for endpoint in &network.endpoints {
            let provider = Provider::<Http>::try_from(endpoint.as_str())?;

            // Create a test wallet for signing transactions
            let wallet = LocalWallet::new(&mut rand::thread_rng());
            let signer = SignerMiddleware::new(provider, wallet);

            providers.push(Arc::new(signer));
        }

        Ok(providers)
    }

    /// Run comprehensive latency tests
    pub async fn run_test(&mut self) -> Result<LatencyTestResults, Box<dyn std::error::Error>> {
        println!("Starting latency test...");

        self.start_time = SystemTime::now();
        let mut test_interval = interval(self.config.test_interval);
        let test_start = Instant::now();

        let mut transaction_counter = 0;

        while test_start.elapsed() < self.config.duration {
            test_interval.tick().await;

            // Test each transaction type
            for &tx_type in &self.config.transaction_types {
                for _ in 0..self.config.samples_per_type {
                    if let Err(e) = self.test_transaction_latency(tx_type, transaction_counter).await {
                        eprintln!("Error testing transaction latency: {}", e);
                    }
                    transaction_counter += 1;
                }
            }

            // Test network propagation if multiple endpoints
            if self.providers.len() > 1 {
                if let Err(e) = self.test_network_propagation().await {
                    eprintln!("Error testing network propagation: {}", e);
                }
            }
        }

        // Wait for pending transactions to complete
        println!("Waiting for pending transactions to complete...");
        tokio::time::sleep(self.config.confirmation_timeout).await;

        // Generate comprehensive results
        let results = self.generate_results().await?;

        println!("Latency test completed. Total measurements: {}", results.measurements.len());

        Ok(results)
    }

    /// Test latency for a specific transaction type
    async fn test_transaction_latency(
        &mut self,
        tx_type: TransactionType,
        nonce: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let provider_idx = nonce as usize % self.providers.len();
        let provider = self.providers[provider_idx].clone();
        let endpoint = self.config.network.endpoints[provider_idx].clone();

        let mut measurement = LatencyMeasurement {
            tx_hash: H256::zero(),
            tx_type,
            submitted_at: SystemTime::now(),
            mempool_at: None,
            block_at: None,
            confirmed_at: None,
            block_number: None,
            gas_used: None,
            gas_price: None,
            endpoint,
            success: false,
            error: None,
        };

        // Create transaction based on type
        let tx_request = self.create_transaction_request(tx_type, nonce).await?;

        // Submit transaction and measure latency
        match provider.send_transaction(tx_request, None).await {
            Ok(pending_tx) => {
                measurement.tx_hash = pending_tx.tx_hash();
                measurement.mempool_at = Some(SystemTime::now());

                // Wait for confirmation with timeout
                match timeout(self.config.confirmation_timeout, pending_tx.confirmations(self.config.confirmation_depth)).await {
                    Ok(Ok(receipt)) => {
                        measurement.block_at = Some(SystemTime::now());
                        measurement.confirmed_at = Some(SystemTime::now());
                        measurement.block_number = receipt.block_number.map(|n| n.as_u64());
                        measurement.gas_used = receipt.gas_used.map(|g| g.as_u64());
                        measurement.success = true;
                    }
                    Ok(Err(e)) => {
                        measurement.error = Some(e.to_string());
                    }
                    Err(_) => {
                        measurement.error = Some("Transaction confirmation timeout".to_string());
                    }
                }
            }
            Err(e) => {
                measurement.error = Some(e.to_string());
            }
        }

        self.measurements.push(measurement);
        Ok(())
    }

    /// Test network propagation latency across multiple endpoints
    async fn test_network_propagation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let tx_request = self.create_transaction_request(TransactionType::Transfer, 0).await?;

        let submit_time = SystemTime::now();

        // Submit to first endpoint
        let first_provider = self.providers[0].clone();
        let pending_tx = first_provider.send_transaction(tx_request, None).await?;
        let tx_hash = pending_tx.tx_hash();

        // Check propagation to other endpoints
        for (i, provider) in self.providers.iter().enumerate().skip(1) {
            let start_check = Instant::now();

            // Poll for transaction existence
            while start_check.elapsed() < Duration::from_secs(30) {
                if let Ok(Some(_)) = provider.get_transaction(tx_hash).await {
                    // Found transaction, record propagation time
                    let propagation_time = SystemTime::now()
                        .duration_since(submit_time)
                        .unwrap_or_default();

                    // Create measurement for propagation
                    let measurement = LatencyMeasurement {
                        tx_hash,
                        tx_type: TransactionType::Transfer,
                        submitted_at: submit_time,
                        mempool_at: Some(SystemTime::now()),
                        block_at: None,
                        confirmed_at: None,
                        block_number: None,
                        gas_used: None,
                        gas_price: None,
                        endpoint: self.config.network.endpoints[i].clone(),
                        success: true,
                        error: None,
                    };

                    self.measurements.push(measurement);
                    break;
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        Ok(())
    }

    /// Create transaction request based on type
    async fn create_transaction_request(
        &self,
        tx_type: TransactionType,
        nonce: u64,
    ) -> Result<TransactionRequest, Box<dyn std::error::Error>> {
        let provider = &self.providers[0];
        let from_address = provider.address();

        let tx_request = match tx_type {
            TransactionType::Transfer => {
                TransactionRequest::new()
                    .from(from_address)
                    .to(Address::random())
                    .value(U256::from(1000000000000000000u64)) // 1 ETH
                    .gas(21000)
                    .gas_price(U256::from(1000000000u64))
            }
            TransactionType::Erc20 => {
                // ERC-20 transfer call
                TransactionRequest::new()
                    .from(from_address)
                    .to(Address::random()) // Contract address
                    .value(U256::zero())
                    .gas(65000)
                    .gas_price(U256::from(1000000000u64))
                    .data(vec![0; 68]) // ERC-20 transfer data
            }
            TransactionType::Deploy => {
                // Simple contract deployment
                TransactionRequest::new()
                    .from(from_address)
                    .value(U256::zero())
                    .gas(2000000)
                    .gas_price(U256::from(1000000000u64))
                    .data(vec![0x60, 0x80, 0x60, 0x40, 0x52]) // Minimal contract bytecode
            }
            TransactionType::ContractCall => {
                // Contract function call
                TransactionRequest::new()
                    .from(from_address)
                    .to(Address::random()) // Contract address
                    .value(U256::zero())
                    .gas(100000)
                    .gas_price(U256::from(1000000000u64))
                    .data(vec![0; 4]) // Function selector
            }
            TransactionType::CrossSubnet => {
                // Cross-subnet message
                TransactionRequest::new()
                    .from(from_address)
                    .to(Address::random()) // Gateway address
                    .value(U256::from(1000000000000000000u64))
                    .gas(200000)
                    .gas_price(U256::from(1000000000u64))
                    .data(vec![0; 128]) // Cross-subnet message data
            }
        };

        Ok(tx_request)
    }

    /// Generate comprehensive latency test results
    async fn generate_results(&self) -> Result<LatencyTestResults, Box<dyn std::error::Error>> {
        let total_transactions = self.measurements.len();
        let successful_transactions = self.measurements.iter().filter(|m| m.success).count();
        let failed_transactions = total_transactions - successful_transactions;
        let success_rate = if total_transactions > 0 {
            successful_transactions as f64 / total_transactions as f64
        } else {
            0.0
        };

        let stats = self.calculate_latency_stats();
        let network_health = self.calculate_network_health().await?;

        let duration = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or_default();

        Ok(LatencyTestResults {
            config: self.config.clone(),
            start_time: self.start_time,
            duration,
            measurements: self.measurements.clone(),
            stats,
            total_transactions,
            successful_transactions,
            failed_transactions,
            success_rate,
            network_health,
        })
    }

    /// Calculate comprehensive latency statistics
    fn calculate_latency_stats(&self) -> LatencyStats {
        let successful_measurements: Vec<&LatencyMeasurement> = self.measurements
            .iter()
            .filter(|m| m.success)
            .collect();

        let end_to_end = self.calculate_end_to_end_latency(&successful_measurements);
        let mempool = self.calculate_mempool_latency(&successful_measurements);
        let block_inclusion = self.calculate_block_inclusion_latency(&successful_measurements);
        let confirmation = self.calculate_confirmation_latency(&successful_measurements);
        let network_propagation = self.calculate_network_propagation_latency(&successful_measurements);

        let by_transaction_type = self.calculate_latency_by_transaction_type(&successful_measurements);
        let by_endpoint = self.calculate_latency_by_endpoint(&successful_measurements);

        LatencyStats {
            end_to_end,
            mempool,
            block_inclusion,
            confirmation,
            network_propagation,
            by_transaction_type,
            by_endpoint,
        }
    }

    /// Calculate end-to-end latency metrics
    fn calculate_end_to_end_latency(&self, measurements: &[&LatencyMeasurement]) -> LatencyMetrics {
        let latencies: Vec<f64> = measurements
            .iter()
            .filter_map(|m| {
                if let (Some(confirmed_at), submitted_at) = (m.confirmed_at, m.submitted_at) {
                    confirmed_at.duration_since(submitted_at).ok()
                        .map(|d| d.as_secs_f64() * 1000.0)
                } else {
                    None
                }
            })
            .collect();

        self.calculate_latency_metrics(&latencies)
    }

    /// Calculate mempool latency metrics
    fn calculate_mempool_latency(&self, measurements: &[&LatencyMeasurement]) -> LatencyMetrics {
        let latencies: Vec<f64> = measurements
            .iter()
            .filter_map(|m| {
                if let (Some(mempool_at), submitted_at) = (m.mempool_at, m.submitted_at) {
                    mempool_at.duration_since(submitted_at).ok()
                        .map(|d| d.as_secs_f64() * 1000.0)
                } else {
                    None
                }
            })
            .collect();

        self.calculate_latency_metrics(&latencies)
    }

    /// Calculate block inclusion latency metrics
    fn calculate_block_inclusion_latency(&self, measurements: &[&LatencyMeasurement]) -> LatencyMetrics {
        let latencies: Vec<f64> = measurements
            .iter()
            .filter_map(|m| {
                if let (Some(block_at), Some(mempool_at)) = (m.block_at, m.mempool_at) {
                    block_at.duration_since(mempool_at).ok()
                        .map(|d| d.as_secs_f64() * 1000.0)
                } else {
                    None
                }
            })
            .collect();

        self.calculate_latency_metrics(&latencies)
    }

    /// Calculate confirmation latency metrics
    fn calculate_confirmation_latency(&self, measurements: &[&LatencyMeasurement]) -> LatencyMetrics {
        let latencies: Vec<f64> = measurements
            .iter()
            .filter_map(|m| {
                if let (Some(confirmed_at), Some(block_at)) = (m.confirmed_at, m.block_at) {
                    confirmed_at.duration_since(block_at).ok()
                        .map(|d| d.as_secs_f64() * 1000.0)
                } else {
                    None
                }
            })
            .collect();

        self.calculate_latency_metrics(&latencies)
    }

    /// Calculate network propagation latency metrics
    fn calculate_network_propagation_latency(&self, measurements: &[&LatencyMeasurement]) -> LatencyMetrics {
        // For network propagation, we look at the time difference between
        // submission and appearance in different endpoints
        let latencies: Vec<f64> = measurements
            .iter()
            .filter_map(|m| {
                if let (Some(mempool_at), submitted_at) = (m.mempool_at, m.submitted_at) {
                    mempool_at.duration_since(submitted_at).ok()
                        .map(|d| d.as_secs_f64() * 1000.0)
                } else {
                    None
                }
            })
            .collect();

        self.calculate_latency_metrics(&latencies)
    }

    /// Calculate latency by transaction type
    fn calculate_latency_by_transaction_type(&self, measurements: &[&LatencyMeasurement]) -> HashMap<TransactionType, LatencyMetrics> {
        let mut by_type = HashMap::new();

        for &tx_type in &self.config.transaction_types {
            let type_measurements: Vec<&LatencyMeasurement> = measurements
                .iter()
                .filter(|m| m.tx_type == tx_type)
                .cloned()
                .collect();

            let latencies: Vec<f64> = type_measurements
                .iter()
                .filter_map(|m| {
                    if let (Some(confirmed_at), submitted_at) = (m.confirmed_at, m.submitted_at) {
                        confirmed_at.duration_since(submitted_at).ok()
                            .map(|d| d.as_secs_f64() * 1000.0)
                    } else {
                        None
                    }
                })
                .collect();

            by_type.insert(tx_type, self.calculate_latency_metrics(&latencies));
        }

        by_type
    }

    /// Calculate latency by endpoint
    fn calculate_latency_by_endpoint(&self, measurements: &[&LatencyMeasurement]) -> HashMap<String, LatencyMetrics> {
        let mut by_endpoint = HashMap::new();

        for endpoint in &self.config.network.endpoints {
            let endpoint_measurements: Vec<&LatencyMeasurement> = measurements
                .iter()
                .filter(|m| m.endpoint == *endpoint)
                .cloned()
                .collect();

            let latencies: Vec<f64> = endpoint_measurements
                .iter()
                .filter_map(|m| {
                    if let (Some(confirmed_at), submitted_at) = (m.confirmed_at, m.submitted_at) {
                        confirmed_at.duration_since(submitted_at).ok()
                            .map(|d| d.as_secs_f64() * 1000.0)
                    } else {
                        None
                    }
                })
                .collect();

            by_endpoint.insert(endpoint.clone(), self.calculate_latency_metrics(&latencies));
        }

        by_endpoint
    }

    /// Calculate latency metrics from raw latency values
    fn calculate_latency_metrics(&self, latencies: &[f64]) -> LatencyMetrics {
        if latencies.is_empty() {
            return LatencyMetrics {
                samples: 0,
                avg_ms: 0.0,
                min_ms: 0.0,
                max_ms: 0.0,
                std_dev_ms: 0.0,
                percentiles: HashMap::new(),
            };
        }

        let samples = latencies.len();
        let avg_ms = latencies.iter().sum::<f64>() / samples as f64;
        let min_ms = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_ms = latencies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let variance = latencies.iter()
            .map(|&x| (x - avg_ms).powi(2))
            .sum::<f64>() / samples as f64;
        let std_dev_ms = variance.sqrt();

        let percentiles = vec![
            ("P50".to_string(), calculate_percentile(latencies, 50.0)),
            ("P90".to_string(), calculate_percentile(latencies, 90.0)),
            ("P95".to_string(), calculate_percentile(latencies, 95.0)),
            ("P99".to_string(), calculate_percentile(latencies, 99.0)),
            ("P99.9".to_string(), calculate_percentile(latencies, 99.9)),
        ].into_iter().collect();

        LatencyMetrics {
            samples,
            avg_ms,
            min_ms,
            max_ms,
            std_dev_ms,
            percentiles,
        }
    }

    /// Calculate network health metrics
    async fn calculate_network_health(&self) -> Result<NetworkHealthMetrics, Box<dyn std::error::Error>> {
        // This is a placeholder for network health calculation
        // In a real implementation, this would query blockchain metrics

        let avg_block_time_ms = 1000.0; // 1 second average block time
        let block_time_std_dev_ms = 100.0; // 100ms std deviation

        let congestion_indicators = CongestionMetrics {
            avg_gas_price: U256::from(1000000000u64),
            gas_price_volatility: 0.1,
            mempool_size_stats: LatencyMetrics {
                samples: 100,
                avg_ms: 50.0,
                min_ms: 10.0,
                max_ms: 200.0,
                std_dev_ms: 25.0,
                percentiles: HashMap::new(),
            },
            queue_depth_stats: LatencyMetrics {
                samples: 100,
                avg_ms: 25.0,
                min_ms: 5.0,
                max_ms: 100.0,
                std_dev_ms: 15.0,
                percentiles: HashMap::new(),
            },
        };

        let validator_metrics = self.config.network.endpoints
            .iter()
            .enumerate()
            .map(|(i, endpoint)| ValidatorMetrics {
                validator_id: format!("validator_{}", i),
                endpoint: endpoint.clone(),
                response_time: LatencyMetrics {
                    samples: 100,
                    avg_ms: 10.0,
                    min_ms: 5.0,
                    max_ms: 50.0,
                    std_dev_ms: 8.0,
                    percentiles: HashMap::new(),
                },
                proposal_frequency: 0.25, // 25% of blocks
                missed_proposals: 0,
                sync_status: true,
            })
            .collect();

        Ok(NetworkHealthMetrics {
            avg_block_time_ms,
            block_time_std_dev_ms,
            congestion_indicators,
            validator_metrics,
        })
    }
}