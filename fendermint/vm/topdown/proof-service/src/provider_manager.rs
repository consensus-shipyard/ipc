// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Multi-provider management with failover and rotation

use anyhow::{Context, Result};
use ipc_api::subnet_id::SubnetID;
use ipc_provider::jsonrpc::JsonRpcClientImpl;
use ipc_provider::lotus::client::{DefaultLotusJsonRPCClient, LotusJsonRPCClient};
use ipc_provider::lotus::message::f3::F3CertificateResponse;
use ipc_provider::lotus::LotusClient;
use parking_lot::RwLock;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use url::Url;

/// Provider health status
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub url: String,
    pub is_healthy: bool,
    pub last_success: Option<Instant>,
    pub last_failure: Option<Instant>,
    pub failure_count: usize,
    pub success_count: usize,
    pub average_latency_ms: Option<u64>,
}

/// Single RPC provider
struct Provider {
    url: String,
    client: Arc<DefaultLotusJsonRPCClient>,
    health: RwLock<ProviderHealth>,
}

impl Provider {
    fn new(url: String, subnet_id: &SubnetID) -> Result<Self> {
        let parsed_url = Url::parse(&url).context("Failed to parse RPC URL")?;
        let rpc_client = JsonRpcClientImpl::new(parsed_url, None);
        let lotus_client = Arc::new(LotusJsonRPCClient::new(rpc_client, subnet_id.clone()));

        let health = RwLock::new(ProviderHealth {
            url: url.clone(),
            is_healthy: true,
            last_success: None,
            last_failure: None,
            failure_count: 0,
            success_count: 0,
            average_latency_ms: None,
        });

        Ok(Self {
            url,
            client: lotus_client,
            health,
        })
    }

    fn mark_success(&self, latency: Duration) {
        let mut health = self.health.write();
        health.is_healthy = true;
        health.last_success = Some(Instant::now());
        health.success_count += 1;
        health.failure_count = 0; // Reset failure count on success

        // Update average latency (simple moving average)
        let new_latency = latency.as_millis() as u64;
        health.average_latency_ms = match health.average_latency_ms {
            Some(avg) => Some((avg * 9 + new_latency) / 10), // Weight recent more
            None => Some(new_latency),
        };
    }

    fn mark_failure(&self) {
        let mut health = self.health.write();
        health.last_failure = Some(Instant::now());
        health.failure_count += 1;

        // Mark unhealthy after 3 consecutive failures
        if health.failure_count >= 3 {
            health.is_healthy = false;
            warn!(
                url = %self.url,
                failures = health.failure_count,
                "Provider marked unhealthy"
            );
        }
    }

    fn is_healthy(&self) -> bool {
        self.health.read().is_healthy
    }

    fn get_health(&self) -> ProviderHealth {
        self.health.read().clone()
    }
}

/// Configuration for provider manager
#[derive(Debug, Clone)]
pub struct ProviderManagerConfig {
    /// Primary RPC URL
    pub primary_url: String,
    /// Fallback RPC URLs
    pub fallback_urls: Vec<String>,
    /// Request timeout
    pub request_timeout: Duration,
    /// Retry count per provider
    pub retry_count: usize,
    /// Backoff between retries
    pub retry_backoff: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Parent subnet ID
    pub parent_subnet_id: SubnetID,
}

/// Multi-provider manager with automatic failover
pub struct ProviderManager {
    providers: Vec<Arc<Provider>>,
    current_index: AtomicUsize,
    config: ProviderManagerConfig,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new(config: ProviderManagerConfig) -> Result<Self> {
        let mut providers = Vec::new();

        // Add primary provider
        providers.push(Arc::new(Provider::new(
            config.primary_url.clone(),
            &config.parent_subnet_id,
        )?));

        // Add fallback providers
        for url in &config.fallback_urls {
            match Provider::new(url.clone(), &config.parent_subnet_id) {
                Ok(provider) => providers.push(Arc::new(provider)),
                Err(e) => {
                    warn!(url = %url, error = %e, "Failed to create fallback provider");
                }
            }
        }

        if providers.is_empty() {
            anyhow::bail!("No valid providers configured");
        }

        info!(
            primary = %config.primary_url,
            fallbacks = config.fallback_urls.len(),
            "Initialized provider manager"
        );

        let manager = Self {
            providers,
            current_index: AtomicUsize::new(0),
            config,
        };

        Ok(manager)
    }

    /// Fetch F3 certificate with automatic failover
    pub async fn fetch_certificate_by_instance(
        &self,
        instance_id: u64,
    ) -> Result<Option<F3CertificateResponse>> {
        let start_index = self.current_index.load(Ordering::Acquire);
        let mut attempts = 0;

        for i in 0..self.providers.len() {
            let index = (start_index + i) % self.providers.len();
            let provider = &self.providers[index];

            // Skip unhealthy providers unless it's the last resort
            if !provider.is_healthy() && i < self.providers.len() - 1 {
                debug!(
                    url = %provider.url,
                    "Skipping unhealthy provider"
                );
                continue;
            }

            attempts += 1;
            debug!(
                url = %provider.url,
                instance_id,
                attempt = attempts,
                "Fetching certificate from provider"
            );

            match self
                .fetch_with_retry(&provider, instance_id)
                .await
            {
                Ok(cert) => {
                    // Update current provider on success
                    self.current_index.store(index, Ordering::Release);
                    return Ok(cert);
                }
                Err(e) => {
                    warn!(
                        url = %provider.url,
                        instance_id,
                        error = %e,
                        "Failed to fetch from provider"
                    );

                    // Try next provider
                    continue;
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed to fetch certificate from all {} providers after {} attempts",
            self.providers.len(),
            attempts
        ))
    }

    /// Fetch with retry logic for a single provider
    async fn fetch_with_retry(
        &self,
        provider: &Arc<Provider>,
        instance_id: u64,
    ) -> Result<Option<F3CertificateResponse>> {
        for attempt in 0..self.config.retry_count {
            if attempt > 0 {
                sleep(self.config.retry_backoff).await;
            }

            let start = Instant::now();

            let result = tokio::time::timeout(
                self.config.request_timeout,
                provider.client.f3_get_cert_by_instance(instance_id),
            )
            .await;

            match result {
                Ok(Ok(cert)) => {
                    provider.mark_success(start.elapsed());
                    debug!(
                        url = %provider.url,
                        instance_id,
                        latency_ms = start.elapsed().as_millis(),
                        "Successfully fetched certificate"
                    );
                    return Ok(cert);
                }
                Ok(Err(e)) => {
                    provider.mark_failure();
                    if attempt == self.config.retry_count - 1 {
                        return Err(e).context("RPC call failed");
                    }
                }
                Err(_) => {
                    provider.mark_failure();
                    if attempt == self.config.retry_count - 1 {
                        anyhow::bail!("Request timeout after {} ms", self.config.request_timeout.as_millis());
                    }
                }
            }
        }

        unreachable!()
    }

    /// Get the latest F3 certificate from any available provider
    pub async fn fetch_latest_certificate(&self) -> Result<Option<F3CertificateResponse>> {
        for provider in &self.providers {
            if !provider.is_healthy() {
                continue;
            }

            match provider.client.f3_get_certificate().await {
                Ok(cert) => return Ok(cert),
                Err(e) => {
                    warn!(
                        url = %provider.url,
                        error = %e,
                        "Failed to fetch latest certificate"
                    );
                }
            }
        }

        Err(anyhow::anyhow!("Failed to fetch latest certificate from all providers"))
    }

    /// Fetch power table with failover
    pub async fn fetch_power_table(
        &self,
        instance_id: u64,
    ) -> Result<Vec<ipc_provider::lotus::message::f3::F3PowerEntry>> {
        for provider in &self.providers {
            if !provider.is_healthy() {
                continue;
            }

            match provider.client.f3_get_power_table(instance_id).await {
                Ok(table) => return Ok(table),
                Err(e) => {
                    warn!(
                        url = %provider.url,
                        error = %e,
                        "Failed to fetch power table"
                    );
                }
            }
        }

        Err(anyhow::anyhow!("Failed to fetch power table from all providers"))
    }

    /// Rotate to the next healthy provider
    pub fn rotate_provider(&self) -> Result<()> {
        let current = self.current_index.load(Ordering::Acquire);
        let mut next = (current + 1) % self.providers.len();
        let start = next;

        // Find next healthy provider
        loop {
            if self.providers[next].is_healthy() {
                self.current_index.store(next, Ordering::Release);
                info!(
                    old_url = %self.providers[current].url,
                    new_url = %self.providers[next].url,
                    "Rotated to next provider"
                );
                return Ok(());
            }

            next = (next + 1) % self.providers.len();
            
            // If we've checked all providers, stick with current
            if next == start {
                warn!("No healthy providers available for rotation");
                return Err(anyhow::anyhow!("No healthy providers available"));
            }
        }
    }

    /// Get health status of all providers
    pub fn get_health_status(&self) -> Vec<ProviderHealth> {
        self.providers
            .iter()
            .map(|p| p.get_health())
            .collect()
    }

    /// Perform health check on all providers
    pub async fn health_check(&self) {
        debug!("Performing health check on all providers");

        for provider in &self.providers {
            let start = Instant::now();
            
            // Simple health check - try to get latest certificate
            match tokio::time::timeout(
                Duration::from_secs(5),
                provider.client.f3_get_certificate(),
            )
            .await
            {
                Ok(Ok(_)) => {
                    provider.mark_success(start.elapsed());
                    debug!(url = %provider.url, "Provider health check passed");
                }
                Ok(Err(e)) => {
                    provider.mark_failure();
                    debug!(
                        url = %provider.url,
                        error = %e,
                        "Provider health check failed"
                    );
                }
                Err(_) => {
                    provider.mark_failure();
                    debug!(url = %provider.url, "Provider health check timed out");
                }
            }
        }
    }

    /// Start background health checker
    pub fn start_health_checker(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let interval = self.config.health_check_interval;
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                ticker.tick().await;
                self.health_check().await;
            }
        })
    }

    /// Get the current active provider URL
    pub fn current_provider_url(&self) -> String {
        let index = self.current_index.load(Ordering::Acquire);
        self.providers[index].url.clone()
    }

    /// Get the number of healthy providers
    pub fn healthy_provider_count(&self) -> usize {
        self.providers.iter().filter(|p| p.is_healthy()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_health_tracking() {
        let subnet = SubnetID::from_str("/r314159").unwrap();
        let provider = Provider::new("http://localhost:1234".to_string(), &subnet).unwrap();

        assert!(provider.is_healthy());

        // Mark failures
        provider.mark_failure();
        provider.mark_failure();
        assert!(provider.is_healthy()); // Still healthy after 2 failures

        provider.mark_failure();
        assert!(!provider.is_healthy()); // Unhealthy after 3 failures

        // Success resets failure count
        provider.mark_success(Duration::from_millis(100));
        assert!(provider.is_healthy());
    }

    #[test]
    fn test_manager_creation() {
        let config = ProviderManagerConfig {
            primary_url: "http://primary:1234".to_string(),
            fallback_urls: vec![
                "http://fallback1:1234".to_string(),
                "http://fallback2:1234".to_string(),
            ],
            request_timeout: Duration::from_secs(30),
            retry_count: 3,
            retry_backoff: Duration::from_secs(1),
            health_check_interval: Duration::from_secs(60),
            parent_subnet_id: SubnetID::from_str("/r314159").unwrap(),
        };

        let manager = ProviderManager::new(config).unwrap();
        assert_eq!(manager.providers.len(), 3);
        assert_eq!(manager.current_provider_url(), "http://primary:1234");
    }
}
