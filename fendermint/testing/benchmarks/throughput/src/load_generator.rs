// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Load generator for benchmarking transactions

use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use ethers::prelude::*;
use futures::future::join_all;
use rand::Rng;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{debug, info, warn};

use crate::{BenchmarkConfig, BenchmarkError, TransactionType};

/// Transaction generator that creates different types of transactions
pub struct LoadGenerator {
    config: BenchmarkConfig,
    provider: Arc<Provider<Http>>,
    accounts: Vec<LocalWallet>,
    semaphore: Arc<Semaphore>,
}

/// Result of a transaction attempt
#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub tx_hash: Option<H256>,
    pub success: bool,
    pub latency: Duration,
    pub error: Option<String>,
    pub tx_type: TransactionType,
    pub timestamp: Instant,
}

/// Load generation statistics
#[derive(Debug, Clone)]
pub struct LoadStats {
    pub total_sent: u64,
    pub total_successful: u64,
    pub total_failed: u64,
    pub current_tps: f64,
    pub average_latency: Duration,
    pub errors: Vec<String>,
}

impl LoadGenerator {
    /// Create a new load generator
    pub async fn new(config: BenchmarkConfig) -> Result<Self> {
        let provider = Provider::<Http>::try_from(&config.network_endpoints.eth_rpc_url)?;
        let provider = Arc::new(provider);

        // Generate test accounts
        let mut accounts = Vec::new();
        let mut rng = rand::thread_rng();

        for _ in 0..config.transaction_config.num_accounts {
            let wallet = LocalWallet::new(&mut rng);
            accounts.push(wallet);
        }

        info!(
            "Created {} test accounts for load generation",
            accounts.len()
        );

        let semaphore = Arc::new(Semaphore::new(config.concurrent_users));

        Ok(Self {
            config,
            provider,
            accounts,
            semaphore,
        })
    }

    /// Generate load for the specified duration
    pub async fn generate_load(&self, duration: Duration) -> Result<Vec<TransactionResult>> {
        let start_time = Instant::now();
        let mut results = Vec::new();
        let mut tasks = Vec::new();

        info!("Starting load generation for {:?}", duration);

        while start_time.elapsed() < duration {
            let elapsed = start_time.elapsed();
            let target_tps = self.config.get_expected_tps(elapsed);
            let interval = Duration::from_secs_f64(1.0 / target_tps as f64);

            // Create transaction task
            let task = self.create_transaction_task().await?;
            tasks.push(task);

            // Sleep to maintain target TPS
            sleep(interval).await;
        }

        // Wait for all tasks to complete
        let task_results = join_all(tasks).await;

        for task_result in task_results {
            match task_result {
                Ok(tx_result) => results.push(tx_result),
                Err(e) => warn!("Transaction task failed: {}", e),
            }
        }

        info!(
            "Load generation completed. Sent {} transactions in {:?}",
            results.len(),
            duration
        );

        Ok(results)
    }

    /// Create a transaction task based on the configured transaction type
    async fn create_transaction_task(&self) -> Result<tokio::task::JoinHandle<Result<TransactionResult>>> {
        let permit = self.semaphore.clone().acquire_owned().await?;
        let provider = self.provider.clone();
        let config = self.config.clone();
        let sender = self.get_random_account();
        let recipient = self.get_random_account();

        let task = tokio::spawn(async move {
            let _permit = permit; // Keep permit alive
            let start_time = Instant::now();

            let result = match config.transaction_type {
                TransactionType::Transfer => {
                    Self::send_transfer(provider, sender, recipient, config.transaction_config.transfer_value).await
                }
                TransactionType::Erc20 => {
                    Self::send_erc20_transfer(provider, sender, recipient, config.transaction_config.transfer_value).await
                }
                TransactionType::Deploy => {
                    Self::deploy_contract(provider, sender).await
                }
                TransactionType::ContractCall => {
                    Self::call_contract(provider, sender, config.transaction_config.contract_address.clone()).await
                }
                TransactionType::CrossSubnet => {
                    Self::send_cross_subnet_message(provider, sender, recipient).await
                }
            };

            let latency = start_time.elapsed();

            match result {
                Ok(tx_hash) => TransactionResult {
                    tx_hash: Some(tx_hash),
                    success: true,
                    latency,
                    error: None,
                    tx_type: config.transaction_type,
                    timestamp: start_time,
                },
                Err(e) => TransactionResult {
                    tx_hash: None,
                    success: false,
                    latency,
                    error: Some(e.to_string()),
                    tx_type: config.transaction_type,
                    timestamp: start_time,
                },
            }
        });

        Ok(task)
    }

    /// Send a simple transfer transaction
    async fn send_transfer(
        provider: Arc<Provider<Http>>,
        sender: LocalWallet,
        recipient: LocalWallet,
        value: u64,
    ) -> Result<H256> {
        let client = SignerMiddleware::new(provider, sender);

        let tx = TransactionRequest::new()
            .to(recipient.address())
            .value(U256::from(value))
            .gas(21000);

        let pending_tx = client.send_transaction(tx, None).await?;
        Ok(pending_tx.tx_hash())
    }

    /// Send an ERC-20 transfer transaction
    async fn send_erc20_transfer(
        provider: Arc<Provider<Http>>,
        sender: LocalWallet,
        recipient: LocalWallet,
        value: u64,
    ) -> Result<H256> {
        // For now, just send a regular transfer
        // In a real implementation, this would interact with an ERC-20 contract
        Self::send_transfer(provider, sender, recipient, value).await
    }

    /// Deploy a simple contract
    async fn deploy_contract(
        provider: Arc<Provider<Http>>,
        sender: LocalWallet,
    ) -> Result<H256> {
        let client = SignerMiddleware::new(provider, sender);

        // Simple contract bytecode (empty contract)
        let bytecode = "0x6080604052348015600f57600080fd5b50603f80601d6000396000f3fe6080604052600080fdfea264697066735822122000000000000000000000000000000000000000000000000000000000000000064736f6c634300080a0033";

        let tx = TransactionRequest::new()
            .data(hex::decode(bytecode.trim_start_matches("0x")).unwrap())
            .gas(100000);

        let pending_tx = client.send_transaction(tx, None).await?;
        Ok(pending_tx.tx_hash())
    }

    /// Call a contract function
    async fn call_contract(
        provider: Arc<Provider<Http>>,
        sender: LocalWallet,
        contract_address: Option<String>,
    ) -> Result<H256> {
        let client = SignerMiddleware::new(provider, sender);

        let to = contract_address
            .and_then(|addr| addr.parse().ok())
            .unwrap_or_else(|| Address::random());

        let tx = TransactionRequest::new()
            .to(to)
            .data(vec![]) // Empty call data
            .gas(50000);

        let pending_tx = client.send_transaction(tx, None).await?;
        Ok(pending_tx.tx_hash())
    }

    /// Send a cross-subnet message
    async fn send_cross_subnet_message(
        provider: Arc<Provider<Http>>,
        sender: LocalWallet,
        recipient: LocalWallet,
    ) -> Result<H256> {
        // For now, just send a regular transfer
        // In a real implementation, this would interact with IPC gateway
        Self::send_transfer(provider, sender, recipient, 1000000000000000000).await
    }

    /// Get a random account from the available accounts
    fn get_random_account(&self) -> LocalWallet {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.accounts.len());
        self.accounts[index].clone()
    }

    /// Calculate load statistics from transaction results
    pub fn calculate_stats(&self, results: &[TransactionResult], duration: Duration) -> LoadStats {
        let total_sent = results.len() as u64;
        let total_successful = results.iter().filter(|r| r.success).count() as u64;
        let total_failed = total_sent - total_successful;

        let current_tps = total_sent as f64 / duration.as_secs_f64();

        let average_latency = if !results.is_empty() {
            let total_latency: Duration = results.iter().map(|r| r.latency).sum();
            total_latency / results.len() as u32
        } else {
            Duration::from_secs(0)
        };

        let errors: Vec<String> = results
            .iter()
            .filter_map(|r| r.error.clone())
            .collect();

        LoadStats {
            total_sent,
            total_successful,
            total_failed,
            current_tps,
            average_latency,
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BenchmarkConfig;

    #[tokio::test]
    async fn test_load_generator_creation() {
        let config = BenchmarkConfig::default();

        // Note: This test would fail without a real network
        // In a real test environment, you'd use a test network
        let result = LoadGenerator::new(config).await;

        // We expect this to fail in the test environment
        assert!(result.is_err());
    }

    #[test]
    fn test_stats_calculation() {
        let config = BenchmarkConfig::default();
        let provider = Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap());
        let generator = LoadGenerator {
            config,
            provider,
            accounts: vec![],
            semaphore: Arc::new(Semaphore::new(1)),
        };

        let results = vec![
            TransactionResult {
                tx_hash: Some(H256::random()),
                success: true,
                latency: Duration::from_millis(100),
                error: None,
                tx_type: TransactionType::Transfer,
                timestamp: Instant::now(),
            },
            TransactionResult {
                tx_hash: None,
                success: false,
                latency: Duration::from_millis(200),
                error: Some("Test error".to_string()),
                tx_type: TransactionType::Transfer,
                timestamp: Instant::now(),
            },
        ];

        let stats = generator.calculate_stats(&results, Duration::from_secs(2));

        assert_eq!(stats.total_sent, 2);
        assert_eq!(stats.total_successful, 1);
        assert_eq!(stats.total_failed, 1);
        assert_eq!(stats.current_tps, 1.0);
        assert_eq!(stats.average_latency, Duration::from_millis(150));
        assert_eq!(stats.errors.len(), 1);
    }
}