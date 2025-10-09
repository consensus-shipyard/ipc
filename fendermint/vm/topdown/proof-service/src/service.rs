// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof generator service - main orchestrator

use crate::assembler::ProofAssembler;
use crate::cache::ProofCache;
use crate::config::ProofServiceConfig;
use crate::types::CacheEntry;
use crate::watcher::ParentWatcher;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::time::{interval, MissedTickBehavior};

/// Main proof generator service
pub struct ProofGeneratorService {
    /// Configuration
    config: ProofServiceConfig,

    /// Proof cache
    cache: Arc<ProofCache>,

    /// Parent chain watcher
    watcher: Arc<ParentWatcher>,

    /// Proof assembler
    assembler: Arc<ProofAssembler>,
}

impl ProofGeneratorService {
    /// Create a new proof generator service
    pub fn new(config: ProofServiceConfig, cache: Arc<ProofCache>) -> Result<Self> {
        let watcher = Arc::new(
            ParentWatcher::new(&config.parent_rpc_url, &config.parent_subnet_id)
                .context("Failed to create parent watcher")?,
        );

        let assembler = Arc::new(ProofAssembler::new(config.parent_rpc_url.clone()));

        Ok(Self {
            config,
            cache,
            watcher,
            assembler,
        })
    }

    /// Run the proof generator service (main loop)
    ///
    /// This polls the parent chain at regular intervals and generates proofs
    /// for new instances sequentially.
    pub async fn run(self) {
        tracing::info!(
            polling_interval = ?self.config.polling_interval,
            lookahead = self.config.lookahead_instances,
            "Starting proof generator service"
        );

        let mut poll_interval = interval(self.config.polling_interval);
        poll_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            poll_interval.tick().await;

            if let Err(e) = self.generate_next_proofs().await {
                tracing::error!(
                    error = %e,
                    "Failed to generate proofs"
                );
            }
        }
    }

    /// Generate proofs for the next needed instances
    ///
    /// This method fetches certificates SEQUENTIALLY by instance ID.
    /// This is critical for:
    /// - Handling restarts (fill gaps from last_committed to parent latest)
    /// - Avoiding missed instances (never skip an instance!)
    /// - Proper crash recovery
    async fn generate_next_proofs(&self) -> Result<()> {
        // 1. Determine what we need
        let last_committed = self.cache.last_committed_instance();
        let next_instance = last_committed + 1;
        let max_instance_to_generate = last_committed + self.config.lookahead_instances;

        tracing::debug!(
            last_committed,
            next_instance,
            max_instance_to_generate,
            "Checking for instances to generate"
        );

        // 2. Fetch certificates SEQUENTIALLY by instance ID
        // CRITICAL: We MUST process instances in order, never skip!
        for instance_id in next_instance..=max_instance_to_generate {
            // Skip if already cached
            if self.cache.contains(instance_id) {
                tracing::debug!(instance_id, "Proof already cached");
                continue;
            }

            // Fetch certificate for THIS SPECIFIC instance
            let cert = match self
                .watcher
                .fetch_certificate_by_instance(instance_id)
                .await?
            {
                Some(cert) => cert,
                None => {
                    // Parent hasn't finalized this instance yet - stop here
                    tracing::debug!(
                        instance_id,
                        "Instance not finalized on parent yet, stopping lookahead"
                    );
                    break; // Don't try to fetch higher instances
                }
            };

            // Generate proof for this certificate
            match self.generate_proof_for_instance(&cert).await {
                Ok(entry) => {
                    self.cache.insert(entry)?;
                    tracing::info!(
                        instance_id,
                        epochs_count = cert.ec_chain.len(),
                        "Successfully generated and cached proof"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        instance_id,
                        error = %e,
                        "Failed to generate proof, will retry next cycle"
                    );
                    // Stop here, retry on next poll cycle
                    break;
                }
            }
        }

        Ok(())
    }

    /// Generate a proof for a specific F3 certificate
    async fn generate_proof_for_instance(
        &self,
        lotus_cert: &ipc_provider::lotus::message::f3::F3CertificateResponse,
    ) -> Result<CacheEntry> {
        tracing::debug!(
            instance_id = lotus_cert.gpbft_instance,
            "Generating proof for instance"
        );

        // Use the assembler to build the proof bundle
        let entry = self.assembler.assemble_proof(lotus_cert).await?;

        Ok(entry)
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
        let config = ProofServiceConfig {
            enabled: true,
            parent_rpc_url: "http://localhost:1234/rpc/v1".to_string(),
            parent_subnet_id: "/r314159".to_string(),
            ..Default::default()
        };

        let cache_config = CacheConfig::from(&config);
        let cache = Arc::new(ProofCache::new(0, cache_config));

        let result = ProofGeneratorService::new(config, cache);
        assert!(result.is_ok());
    }

    // More comprehensive tests would require mocking the parent chain RPC
}
