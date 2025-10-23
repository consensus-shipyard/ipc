// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Parent chain client for fetching F3 certificates and Filecoin data
//!
//! Merges the functionality of the previous watcher and provider_manager modules
//! into a single, cohesive client with automatic failover.

use anyhow::{Context, Result};
use ipc_api::subnet_id::SubnetID;
use ipc_provider::jsonrpc::JsonRpcClientImpl;
use ipc_provider::lotus::client::{DefaultLotusJsonRPCClient, LotusJsonRPCClient};
use ipc_provider::lotus::message::f3::F3CertificateResponse;
use ipc_provider::lotus::LotusClient as LotusClientTrait;
use parking_lot::RwLock;
use serde_json::json;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info, warn};
use url::Url;

/// Health status of a provider
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub url: String,
    pub is_healthy: bool,
    pub last_success: Option<Instant>,
    pub last_failure: Option<Instant>,
    pub failure_count: usize,
    pub success_count: usize,
}

/// Single RPC provider with health tracking
struct Provider {
    url: String,
    lotus_client: Arc<DefaultLotusJsonRPCClient>,
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
        });

        Ok(Self {
            url,
            lotus_client,
            health,
        })
    }

    fn mark_success(&self, latency: Duration) {
        let mut health = self.health.write();
        let was_unhealthy = !health.is_healthy;
        
        health.is_healthy = true;
        health.last_success = Some(Instant::now());
        health.success_count += 1;
        health.failure_count = 0;

        if was_unhealthy {
            info!(url = %self.url, "Provider recovered and marked healthy");
        } else {
            debug!(
                url = %self.url,
                latency_ms = latency.as_millis(),
                "Provider request succeeded"
            );
        }
    }

    fn mark_failure(&self) {
        let mut health = self.health.write();
        health.last_failure = Some(Instant::now());
        health.failure_count += 1;

        if health.failure_count >= 3 {
            health.is_healthy = false;
            warn!(
                url = %self.url,
                failures = health.failure_count,
                "Provider marked unhealthy"
            );
        }
    }
    
    /// Try a health check probe (lightweight test request)
    async fn health_check_probe(&self) -> bool {
        let start = Instant::now();
        
        match tokio::time::timeout(
            Duration::from_secs(5),
            self.lotus_client.as_ref().f3_get_certificate(),
        )
        .await
        {
            Ok(Ok(_)) => {
                self.mark_success(start.elapsed());
                true
            }
            _ => {
                // Don't mark failure - this is just a probe
                false
            }
        }
    }

    fn is_healthy(&self) -> bool {
        self.health.read().is_healthy
    }

    fn get_health(&self) -> ProviderHealth {
        self.health.read().clone()
    }
}

/// Configuration for parent client
#[derive(Debug, Clone)]
pub struct ParentClientConfig {
    pub primary_url: String,
    pub fallback_urls: Vec<String>,
    pub parent_subnet_id: String,
    pub request_timeout: Duration,
    pub retry_count: usize,
}

impl Default for ParentClientConfig {
    fn default() -> Self {
        Self {
            primary_url: String::new(),
            fallback_urls: Vec::new(),
            parent_subnet_id: "/r314159".to_string(),
            request_timeout: Duration::from_secs(30),
            retry_count: 3,
        }
    }
}

/// Client for fetching data from parent chain with automatic failover
pub struct ParentClient {
    providers: Vec<Arc<Provider>>,
    current_index: AtomicUsize,
    config: ParentClientConfig,
}

impl ParentClient {
    /// Create a new parent client with multi-provider support
    pub fn new(config: ParentClientConfig) -> Result<Self> {
        let subnet_id = SubnetID::from_str(&config.parent_subnet_id)
            .context("Failed to parse parent subnet ID")?;

        let mut providers = Vec::new();

        // Add primary provider
        providers.push(Arc::new(Provider::new(
            config.primary_url.clone(),
            &subnet_id,
        )?));

        // Add fallback providers
        for url in &config.fallback_urls {
            match Provider::new(url.clone(), &subnet_id) {
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
            "Initialized parent client with {} providers",
            providers.len()
        );

        Ok(Self {
            providers,
            current_index: AtomicUsize::new(0),
            config,
        })
    }

    /// Fetch F3 certificate for a specific instance with automatic failover
    pub async fn fetch_certificate(
        &self,
        instance_id: u64,
    ) -> Result<Option<F3CertificateResponse>> {
        let start_index = self.current_index.load(Ordering::Acquire);

        for i in 0..self.providers.len() {
            let index = (start_index + i) % self.providers.len();
            let provider = &self.providers[index];

            // Skip unhealthy providers unless it's the last resort
            if !provider.is_healthy() && i < self.providers.len() - 1 {
                debug!(url = %provider.url, "Skipping unhealthy provider");
                continue;
            }

            debug!(
                url = %provider.url,
                instance_id,
                "Fetching certificate from provider"
            );

            match self.fetch_with_retry(provider, instance_id).await {
                Ok(cert) => {
                    // Update current provider on success and auto-rotate
                    self.current_index.store(index, Ordering::Release);
                    return Ok(cert);
                }
                Err(e) => {
                    warn!(
                        url = %provider.url,
                        instance_id,
                        error = %e,
                        "Failed to fetch from provider, trying next"
                    );
                    continue;
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed to fetch certificate {} from all {} providers",
            instance_id,
            self.providers.len()
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
                sleep(Duration::from_secs(1)).await;
            }

            let start = Instant::now();

            let result = tokio::time::timeout(
                self.config.request_timeout,
                provider
                    .lotus_client
                    .as_ref()
                    .f3_get_cert_by_instance(instance_id),
            )
            .await;

            match result {
                Ok(Ok(cert)) => {
                    provider.mark_success(start.elapsed());
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
                        anyhow::bail!("Request timeout");
                    }
                }
            }
        }

        unreachable!()
    }

    /// Fetch the latest F3 certificate
    pub async fn fetch_latest_certificate(&self) -> Result<Option<F3CertificateResponse>> {
        for provider in &self.providers {
            if !provider.is_healthy() {
                continue;
            }

            match provider.lotus_client.as_ref().f3_get_certificate().await {
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

        Err(anyhow::anyhow!(
            "Failed to fetch latest certificate from all providers"
        ))
    }

    /// Fetch tipsets for a specific epoch (parent and child)
    pub async fn fetch_tipsets(
        &self,
        epoch: i64,
    ) -> Result<(serde_json::Value, serde_json::Value)> {
        let provider = self.get_healthy_provider()?;

        // Use proofs library LotusClient for raw JSON-RPC calls
        let lotus_client = proofs::client::LotusClient::new(Url::parse(&provider.url)?, None);

        let parent = lotus_client
            .request("Filecoin.ChainGetTipSetByHeight", json!([epoch, null]))
            .await
            .context("Failed to fetch parent tipset")?;

        let child = lotus_client
            .request("Filecoin.ChainGetTipSetByHeight", json!([epoch + 1, null]))
            .await
            .context("Failed to fetch child tipset")?;

        Ok((parent, child))
    }

    /// Get a healthy provider or return error
    fn get_healthy_provider(&self) -> Result<&Provider> {
        let start_index = self.current_index.load(Ordering::Acquire);

        for i in 0..self.providers.len() {
            let index = (start_index + i) % self.providers.len();
            if self.providers[index].is_healthy() {
                return Ok(&self.providers[index]);
            }
        }

        // If no healthy providers, return the current one anyway (last resort)
        Ok(&self.providers[start_index])
    }

    /// Get current provider URL
    pub fn current_provider_url(&self) -> String {
        let index = self.current_index.load(Ordering::Acquire);
        self.providers[index].url.clone()
    }

    /// Get health status of all providers
    pub fn get_health_status(&self) -> Vec<ProviderHealth> {
        self.providers.iter().map(|p| p.get_health()).collect()
    }

    /// Perform health check on all unhealthy providers to allow recovery
    ///
    /// This should be called periodically (e.g., every 60s) to give failed
    /// providers a chance to recover and become healthy again.
    pub async fn health_check_unhealthy(&self) {
        debug!("Checking unhealthy providers for recovery");
        
        for provider in &self.providers {
            if !provider.is_healthy() {
                debug!(url = %provider.url, "Probing unhealthy provider");
                
                if provider.health_check_probe().await {
                    info!(url = %provider.url, "Unhealthy provider recovered!");
                } else {
                    debug!(url = %provider.url, "Provider still unhealthy");
                }
            }
        }
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

        provider.mark_failure();
        provider.mark_failure();
        assert!(provider.is_healthy()); // Still healthy after 2 failures

        provider.mark_failure();
        assert!(!provider.is_healthy()); // Unhealthy after 3 failures

        provider.mark_success(Duration::from_millis(100));
        assert!(provider.is_healthy()); // Healthy again after success
    }

    #[test]
    fn test_client_creation() {
        let config = ParentClientConfig {
            primary_url: "http://primary:1234".to_string(),
            fallback_urls: vec!["http://fallback:1234".to_string()],
            parent_subnet_id: "/r314159".to_string(),
            ..Default::default()
        };

        let client = ParentClient::new(config).unwrap();
        assert_eq!(client.providers.len(), 2);
        assert_eq!(
            client.current_provider_url(),
            "http://primary:1234".to_string()
        );
    }

    #[test]
    fn test_provider_recovery() {
        let subnet = SubnetID::from_str("/r314159").unwrap();
        let provider = Provider::new("http://localhost:1234".to_string(), &subnet).unwrap();

        // Mark as unhealthy
        provider.mark_failure();
        provider.mark_failure();
        provider.mark_failure();
        assert!(!provider.is_healthy());

        // Simulate successful request - should recover
        provider.mark_success(Duration::from_millis(100));
        assert!(provider.is_healthy(), "Provider should recover after success");
        
        let health = provider.get_health();
        assert_eq!(health.failure_count, 0, "Failure count should reset");
        assert!(health.success_count > 0, "Success count should increment");
    }
}
