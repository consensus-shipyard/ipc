// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof generator service - orchestrates proof generation pipeline
//!
//! The service implements a clear 4-step flow:
//! 1. FETCH - Get F3 certificates from parent chain
//! 2. VALIDATE - Cryptographically validate certificates
//! 3. GENERATE - Create proof bundles
//! 4. CACHE - Store proofs for proposers

use crate::assembler::ProofAssembler;
use crate::cache::ProofCache;
use crate::config::{GatewayId, ProofServiceConfig};
use crate::f3_client::F3Client;
use crate::types::CacheEntry;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, MissedTickBehavior};

/// Main proof generator service
pub struct ProofGeneratorService {
    config: ProofServiceConfig,
    cache: Arc<ProofCache>,
    f3_client: Arc<Mutex<F3Client>>,
    assembler: Arc<ProofAssembler>,
}

impl ProofGeneratorService {
    /// Create a new proof generator service
    ///
    /// # Arguments
    /// * `config` - Service configuration
    /// * `cache` - Proof cache
    /// * `initial_instance` - F3 instance to bootstrap from (from F3CertManager actor)
    /// * `initial_power_table` - Initial power table (from F3CertManager actor)
    ///
    /// Both `initial_instance` and `initial_power_table` should come from the F3CertManager
    /// actor on-chain, which holds the last committed certificate and its power table.
    pub async fn new(
        config: ProofServiceConfig,
        cache: Arc<ProofCache>,
        initial_instance: u64,
        initial_power_table: filecoin_f3_gpbft::PowerEntries,
    ) -> Result<Self> {
        let gateway_actor_id = extract_gateway_actor_id_from_config(&config).await?;

        // Get the current highest instance from the cache
        // or the last committed instance if the cache is empty
        let highest_cached_instance = cache
            .highest_cached_instance()
            .unwrap_or_else(|| cache.last_committed_instance());

        let (mut initial_instance, mut initial_power_table) =
            (initial_instance, initial_power_table);

        if highest_cached_instance > initial_instance {
            tracing::info!(
                highest_cached_instance,
                initial_instance,
                "Using cached instance instead of initial instance"
            );

            initial_instance = highest_cached_instance;

            initial_power_table = cache
                .get(highest_cached_instance)
                .context("Failed to get cached power table")?
                .power_table;
        }

        // Create F3 client for certificate fetching + validation
        let f3_client = Arc::new(Mutex::new(
            F3Client::new(
                &config.parent_rpc_url,
                &config.f3_network_name(),
                initial_instance,
                initial_power_table,
            )
            .context("Failed to create F3 client")?,
        ));

        // Create proof assembler
        let assembler = Arc::new(
            ProofAssembler::new(
                config.parent_rpc_url.clone(),
                gateway_actor_id,
                config.subnet_id.to_string(),
            )
            .context("Failed to create proof assembler")?,
        );

        Ok(Self {
            config,
            cache,
            f3_client,
            assembler,
        })
    }

    /// Main service loop - runs continuously and polls parent chain periodically
    ///
    /// Maintains a ticker that triggers proof generation at regular intervals.
    /// Errors are logged but don't stop the service - it will retry on next tick.
    pub async fn run(self) {
        tracing::info!(
            polling_interval = ?self.config.polling_interval,
            lookahead = self.config.cache_config.lookahead_instances,
            "Starting proof generator service"
        );

        // Validator is already initialized in new() with trusted power table
        let mut poll_interval = interval(self.config.polling_interval);
        poll_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            poll_interval.tick().await;

            tracing::debug!("Poll interval tick");
            if let Err(e) = self.generate_next_proofs().await {
                tracing::error!(
                    error = %e,
                    "Failed to generate proofs, will retry on next tick"
                );
            }
        }
    }

    /// Generate proofs for next needed instances
    ///
    /// Called by run() on each tick. Implements the core flow:
    /// FETCH → VALIDATE → GENERATE → CACHE
    ///
    /// CRITICAL: Processes F3 instances SEQUENTIALLY - never skips!
    async fn generate_next_proofs(&self) -> Result<()> {
        let (current_instance, rpc_endpoint) = {
            let f3_client = self.f3_client.lock().await;
            (f3_client.current_instance(), f3_client.rpc_endpoint())
        };

        let next_instance = current_instance + 1;
        let max_instance = current_instance + self.config.cache_config.lookahead_instances;

        tracing::debug!(
            current_instance,
            next_instance,
            max_instance,
            "Checking for new F3 certificates"
        );

        // Process instances IN ORDER - this is critical for F3
        for instance_id in next_instance..=max_instance {
            // Fetch and validate certificate
            let certificate = {
                let mut client = self.f3_client.lock().await;
                let result = client.fetch_and_validate().await;
                drop(client);

                match result {
                    Ok(cert) => cert,
                    Err(err) if is_certificate_unavailable(&err) => {
                        tracing::debug!(
                            instance_id,
                            "Certificate not available, stopping lookahead"
                        );
                        break;
                    }
                    Err(err) => {
                        return Err(err).with_context(|| {
                            format!(
                                "Failed to fetch and validate certificate for instance {}",
                                instance_id
                            )
                        });
                    }
                }
            };

            // Log detailed certificate information for debugging
            let suffix = &certificate.ec_chain.suffix();
            let base_epoch = certificate.ec_chain.base().map(|b| b.epoch);
            let suffix_epochs: Vec<i64> = suffix.iter().map(|ts| ts.epoch).collect();

            tracing::info!(
                instance_id,
                ec_chain_len = suffix.len(),
                base_epoch = ?base_epoch,
                suffix_epochs = ?suffix_epochs,
                "Certificate fetched and validated successfully"
            );

            // Skip certificates with empty suffix (no epochs to prove)
            let proof_bundle = if !certificate.ec_chain.suffix().is_empty() {
                match self.generate_proof_for_certificate(&certificate).await {
                    Ok(bundle) => bundle,
                    Err(e) => {
                        tracing::error!(instance_id, error = %e, "Failed to generate proof bundle - detailed error");
                        return Err(e).context("Failed to generate proof bundle");
                    }
                }
            } else {
                None
            };

            // Cache the result
            let power_table = {
                let client = self.f3_client.lock().await;
                client.state.power_table.clone()
            };

            self.cache.insert(CacheEntry::new(
                certificate,
                proof_bundle,
                power_table,
                rpc_endpoint.clone(),
            ))?;

            tracing::info!(
                instance_id,
                "Successfully cached validated certificate and proof bundle"
            );
        }

        Ok(())
    }

    /// Generate proof bundle for a specific certificate
    ///
    /// Extracts the highest epoch, fetches tipsets, and generates proofs.
    async fn generate_proof_for_certificate(
        &self,
        f3_cert: &filecoin_f3_certs::FinalityCertificate,
    ) -> Result<Option<proofs::proofs::common::bundle::UnifiedProofBundle>> {
        // Extract highest epoch from validated F3 certificate
        let highest_epoch = f3_cert
            .ec_chain
            .suffix()
            .last()
            .map(|ts| ts.epoch)
            .context("Certificate has no epochs")?;

        tracing::debug!(
            instance_id = f3_cert.gpbft_instance,
            highest_epoch,
            "Generating proof for certificate"
        );

        // Generate proof (assembler fetches its own tipsets)
        let bundle = self
            .assembler
            .generate_proof_bundle(f3_cert.ec_chain.clone())
            .await
            .with_context(|| {
                format!(
                    "Failed to generate proof bundle for instance {} - check RPC tipset availability and network connectivity",
                    f3_cert.gpbft_instance
                )
            })?;

        Ok(bundle)
    }

    /// Get reference to the cache (for proposers)
    pub fn cache(&self) -> &Arc<ProofCache> {
        &self.cache
    }
}

fn is_certificate_unavailable(err: &anyhow::Error) -> bool {
    let message = err.to_string();
    message.contains("not found") || message.contains("not available")
}

async fn extract_gateway_actor_id_from_config(config: &ProofServiceConfig) -> Result<u64> {
    match &config.gateway_id {
        GatewayId::ActorId(id) => Ok(*id),
        GatewayId::EthAddress(eth_addr) => {
            resolve_eth_address_to_actor_id(eth_addr, &config.parent_rpc_url).await
        }
    }
}

async fn resolve_eth_address_to_actor_id(eth_addr: &str, parent_rpc_url: &str) -> Result<u64> {
    let client = proofs::client::LotusClient::new(url::Url::parse(parent_rpc_url)?, None);
    let actor_id = proofs::proofs::resolve_eth_address_to_actor_id(&client, eth_addr)
        .await
        .with_context(|| format!("Failed to resolve gateway Ethereum address: {}", eth_addr))?;
    Ok(actor_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        use filecoin_f3_gpbft::PowerEntries;

        let config = ProofServiceConfig {
            enabled: true,
            parent_rpc_url: "http://localhost:1234/rpc/v1".to_string(),
            gateway_id: GatewayId::ActorId(1001),
            subnet_id: Default::default(),
            cache_config: Default::default(),
            ..Default::default()
        };

        let cache = Arc::new(ProofCache::new(0, config.cache_config.clone()));
        let power_table = PowerEntries(vec![]);

        // Note: Service creation succeeds with F3Client::new() even with a fake RPC endpoint
        // The actual RPC calls will fail later when the service tries to fetch certificates
        let result = ProofGeneratorService::new(config, cache, 0, power_table).await;
        assert!(result.is_ok());
    }
}
