// Copyright 2022-2024 Protocol Labs
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
use crate::config::ProofServiceConfig;
use crate::f3_client::F3Client;
use crate::types::CacheEntry;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::time::{interval, MissedTickBehavior};

/// Main proof generator service
pub struct ProofGeneratorService {
    config: ProofServiceConfig,
    cache: Arc<ProofCache>,
    f3_client: Arc<F3Client>,
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
        // Resolve gateway actor ID (support both direct ID and Ethereum address)
        let gateway_actor_id = if let Some(id) = config.gateway_actor_id {
            id
        } else if let Some(eth_addr) = &config.gateway_eth_address {
            // Resolve Ethereum address to actor ID
            tracing::info!(eth_address = %eth_addr, "Resolving gateway Ethereum address to actor ID");
            let client =
                proofs::client::LotusClient::new(url::Url::parse(&config.parent_rpc_url)?, None);
            let actor_id = proofs::proofs::resolve_eth_address_to_actor_id(&client, eth_addr)
                .await
                .with_context(|| {
                    format!("Failed to resolve gateway Ethereum address: {}", eth_addr)
                })?;
            tracing::info!(eth_address = %eth_addr, actor_id, "Resolved gateway address");
            actor_id
        } else {
            anyhow::bail!("Either gateway_actor_id or gateway_eth_address must be configured");
        };
        let subnet_id = config
            .subnet_id
            .as_ref()
            .context("subnet_id is required in configuration")?;

        // Create F3 client for certificate fetching + validation
        // Uses provided power table from F3CertManager actor
        let f3_client = Arc::new(
            F3Client::new(
                &config.parent_rpc_url,
                &config.f3_network_name,
                initial_instance,
                initial_power_table,
            )
            .context("Failed to create F3 client")?,
        );

        // Create proof assembler
        let assembler = Arc::new(
            ProofAssembler::new(
                config.parent_rpc_url.clone(),
                gateway_actor_id,
                subnet_id.clone(),
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
            lookahead = self.config.lookahead_instances,
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
        let last_committed = self.cache.last_committed_instance();
        // Lookahead window starts AFTER last_committed (which was already processed)
        let next_instance = last_committed + 1;
        let max_instance = last_committed + self.config.lookahead_instances;

        tracing::debug!(
            last_committed,
            next_instance,
            max_instance,
            "Checking for new F3 certificates"
        );

        // Process instances IN ORDER - this is critical for F3
        for instance_id in next_instance..=max_instance {
            // Skip if already cached
            if self.cache.contains(instance_id) {
                tracing::debug!(instance_id, "Proof already cached");
                continue;
            }

            // ====================
            // STEP 1: FETCH + VALIDATE certificate (single operation!)
            // ====================
            let validated = match self.f3_client.fetch_and_validate(instance_id).await {
                Ok(cert) => cert,
                Err(e)
                    if e.to_string().contains("not found")
                        || e.to_string().contains("not available") =>
                {
                    // Certificate not available yet - STOP HERE!
                    // Don't try higher instances as they depend on this one
                    tracing::debug!(instance_id, "Certificate not available, stopping lookahead");
                    break;
                }
                Err(e) => {
                    return Err(e).with_context(|| {
                        format!(
                            "Failed to fetch and validate certificate for instance {}",
                            instance_id
                        )
                    });
                }
            };

            tracing::info!(
                instance_id,
                ec_chain_len = validated.f3_cert.ec_chain.suffix().len(),
                "Certificate fetched and validated successfully"
            );

            // ====================
            // STEP 2: GENERATE proof bundle
            // ====================
            let proof_bundle = self
                .generate_proof_for_certificate(&validated.f3_cert)
                .await
                .context("Failed to generate proof bundle")?;

            // ====================
            // STEP 3: CACHE the result
            // ====================
            let entry = CacheEntry::new(
                &validated.f3_cert,
                proof_bundle,
                "F3 RPC".to_string(), // source_rpc
            );

            self.cache.insert(entry)?;

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
    ) -> Result<proofs::proofs::common::bundle::UnifiedProofBundle> {
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
            .generate_proof_bundle(f3_cert)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CacheConfig;

    #[tokio::test]
    async fn test_service_creation() {
        use filecoin_f3_gpbft::PowerEntries;

        let config = ProofServiceConfig {
            enabled: true,
            parent_rpc_url: "http://localhost:1234/rpc/v1".to_string(),
            parent_subnet_id: "/r314159".to_string(),
            f3_network_name: "calibrationnet".to_string(),
            gateway_actor_id: Some(1001),
            subnet_id: Some("test-subnet".to_string()),
            ..Default::default()
        };

        let cache_config = CacheConfig::from(&config);
        let cache = Arc::new(ProofCache::new(0, cache_config));
        let power_table = PowerEntries(vec![]);

        // Note: Service creation succeeds with F3Client::new() even with a fake RPC endpoint
        // The actual RPC calls will fail later when the service tries to fetch certificates
        let result = ProofGeneratorService::new(config, cache, 0, power_table).await;
        assert!(result.is_ok());
    }
}
