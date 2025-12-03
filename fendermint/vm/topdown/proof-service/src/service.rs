// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof generator service - orchestrates proof generation pipeline
//!
//! # Architecture
//!
//! The service implements a "delayed processing" flow to ensure that
//! child tipsets are finalized before generating proofs:
//!
//! ```text
//! 1. FETCH Certificate N+1
//! 2. CHECK continuity: pending_cert.last_epoch + 1 == new_cert.first_epoch
//! 3. GENERATE proofs for ALL epochs in pending_cert.suffix
//!    - For each epoch E: generate proof using (E, E+1) as (parent, child)
//! 4. CACHE certificates and epoch proofs
//! 5. pending_cert = new_cert
//! ```
//!
//! This ensures that when we prove epoch E, both E and E+1 are certified
//! by F3 certificates, making the witness blocks verifiable.

use crate::assembler::ProofAssembler;
use crate::cache::ProofCache;
use crate::config::{GatewayId, ProofServiceConfig};
use crate::f3_client::F3Client;
use crate::types::{CertificateEntry, EpochProofEntry};
use anyhow::{Context, Result};
use filecoin_f3_certs::FinalityCertificate;
use ipc_api::subnet_id::SubnetID;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, MissedTickBehavior};

/// Main proof generator service
pub struct ProofGeneratorService {
    config: ProofServiceConfig,
    cache: Arc<ProofCache>,
    f3_client: Arc<F3Client>,
    assembler: Arc<ProofAssembler>,

    /// The certificate waiting for its child to be finalized
    /// When the next certificate arrives, we can process this one's epochs
    pending_certificate: Mutex<Option<PendingCertificate>>,
}

/// A certificate that is waiting for its child tipset to be finalized
#[derive(Debug, Clone)]
struct PendingCertificate {
    certificate: FinalityCertificate,
    power_table: filecoin_f3_gpbft::PowerEntries,
    source_rpc: String,
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
        subnet_id: &SubnetID,
        initial_instance: u64,
        initial_power_table: filecoin_f3_gpbft::PowerEntries,
    ) -> Result<Self> {
        let gateway_actor_id = extract_gateway_actor_id_from_config(&config).await?;

        // Get the current highest instance from the cache
        let highest_cached_instance = cache.highest_cached_instance();

        let (start_instance, start_power_table) = if let Some(cached) = highest_cached_instance {
            if cached > initial_instance {
                tracing::info!(
                    highest_cached_instance = cached,
                    initial_instance,
                    "Using cached instance instead of initial instance"
                );

                let cert_entry = cache
                    .get_certificate(cached)
                    .context("Failed to get cached certificate")?;
                (cached, cert_entry.power_table)
            } else {
                (initial_instance, initial_power_table)
            }
        } else {
            (initial_instance, initial_power_table)
        };

        // Create F3 client for certificate fetching + validation
        let f3_client = Arc::new(
            F3Client::new(
                &config.parent_rpc_url,
                &config.f3_network_name(subnet_id),
                start_instance,
                start_power_table,
            )
            .context("Failed to create F3 client")?,
        );

        // Create proof assembler
        let assembler = Arc::new(
            ProofAssembler::new(
                config.parent_rpc_url.clone(),
                gateway_actor_id,
                subnet_id.to_string(),
            )
            .context("Failed to create proof assembler")?,
        );

        Ok(Self {
            config,
            cache,
            f3_client,
            assembler,
            pending_certificate: Mutex::new(None),
        })
    }

    /// Main service loop - runs continuously and polls parent chain periodically
    ///
    /// Maintains a ticker that triggers proof generation at regular intervals.
    /// Errors are logged but don't stop the service - it will retry on next tick.
    pub async fn run(self) {
        tracing::info!(
            polling_interval = ?self.config.polling_interval,
            lookahead_epochs = self.config.cache_config.lookahead_epochs,
            "Starting proof generator service"
        );

        let mut poll_interval = interval(self.config.polling_interval);
        poll_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            poll_interval.tick().await;

            tracing::debug!("Poll interval tick");
            if let Err(e) = self.process_next_certificates().await {
                tracing::error!(
                    error = %e,
                    "Failed to process certificates, will retry on next tick"
                );
            }
        }
    }

    /// Process next certificates and generate proofs
    ///
    /// Implements the delayed processing flow:
    /// 1. Fetch next certificate
    /// 2. If we have a pending certificate and continuity is satisfied, process it
    /// 3. Store the new certificate as pending
    async fn process_next_certificates(&self) -> Result<()> {
        let current_instance = self.f3_client.current_instance().await;
        let rpc_endpoint = self.f3_client.rpc_endpoint();

        // Calculate how many instances to look ahead based on epochs
        // This is approximate since we don't know exactly how many epochs per instance
        let lookahead_instances = self.config.cache_config.lookahead_epochs / 3 + 5;
        let max_instance = current_instance + lookahead_instances;

        tracing::debug!(
            current_instance,
            max_instance,
            "Checking for new F3 certificates"
        );

        // Process instances IN ORDER - this is critical for F3
        for _i in 0..lookahead_instances {
            // Fetch and validate next certificate
            let new_cert = {
                let result = self.f3_client.fetch_and_validate().await;
                match result {
                    Ok(cert) => cert,
                    Err(err) if is_certificate_unavailable(&err) => {
                        tracing::debug!("Certificate not available, stopping lookahead");
                        break;
                    }
                    Err(err) => {
                        return Err(err).context("Failed to fetch and validate certificate");
                    }
                }
            };

            let new_instance = new_cert.gpbft_instance;
            let new_power_table = self.f3_client.get_state().await.power_table;

            // Log certificate info
            let suffix = new_cert.ec_chain.suffix();
            let base_epoch = new_cert.ec_chain.base().map(|b| b.epoch);
            let suffix_epochs: Vec<i64> = suffix.iter().map(|ts| ts.epoch).collect();

            tracing::info!(
                instance = new_instance,
                base_epoch = ?base_epoch,
                suffix_epochs = ?suffix_epochs,
                "Fetched and validated certificate"
            );

            // Check if we have a pending certificate to process
            let mut pending_guard = self.pending_certificate.lock().await;

            if let Some(pending) = pending_guard.take() {
                // Check continuity: pending's last epoch + 1 should equal new cert's first epoch
                let can_process = check_continuity(&pending.certificate, &new_cert);

                if can_process {
                    // Process all epochs from the pending certificate
                    self.process_pending_certificate(
                        &pending,
                        &new_cert,
                        &new_power_table,
                        &rpc_endpoint,
                    )
                    .await?;
                } else {
                    // Continuity broken - log warning and skip the pending cert
                    let pending_last = pending
                        .certificate
                        .ec_chain
                        .suffix()
                        .last()
                        .map(|t| t.epoch);
                    let new_first = new_cert.ec_chain.base().map(|t| t.epoch);
                    tracing::warn!(
                        pending_instance = pending.certificate.gpbft_instance,
                        pending_last_epoch = ?pending_last,
                        new_instance,
                        new_first_epoch = ?new_first,
                        "Certificate continuity broken, skipping pending certificate"
                    );
                }
            }

            // Store new certificate as pending (it will be processed when next cert arrives)
            // Also cache the certificate immediately for reference
            let cert_entry = CertificateEntry::new(
                new_cert.clone(),
                new_power_table.clone(),
                rpc_endpoint.clone(),
            );
            self.cache.insert_certificate(cert_entry)?;

            *pending_guard = Some(PendingCertificate {
                certificate: new_cert,
                power_table: new_power_table,
                source_rpc: rpc_endpoint.clone(),
            });

            tracing::debug!(
                instance = new_instance,
                "Stored certificate as pending, waiting for next certificate"
            );
        }

        Ok(())
    }

    /// Process a pending certificate now that we have the child certificate
    ///
    /// Generates proofs for ALL epochs in the pending certificate's suffix.
    async fn process_pending_certificate(
        &self,
        pending: &PendingCertificate,
        child_cert: &FinalityCertificate,
        child_power_table: &filecoin_f3_gpbft::PowerEntries,
        rpc_endpoint: &str,
    ) -> Result<()> {
        let pending_instance = pending.certificate.gpbft_instance;
        let child_instance = child_cert.gpbft_instance;
        let suffix = pending.certificate.ec_chain.suffix();

        if suffix.is_empty() {
            tracing::debug!(
                pending_instance,
                "Pending certificate has empty suffix, nothing to prove"
            );
            return Ok(());
        }

        let epochs: Vec<i64> = suffix.iter().map(|ts| ts.epoch).collect();
        tracing::info!(
            pending_instance,
            child_instance,
            epochs = ?epochs,
            "Processing pending certificate - generating proofs for all epochs"
        );

        // Ensure child certificate is cached
        let child_entry = CertificateEntry::new(
            child_cert.clone(),
            child_power_table.clone(),
            rpc_endpoint.to_string(),
        );
        self.cache.insert_certificate(child_entry)?;

        // Generate proofs for each epoch in the suffix
        let mut epoch_proofs = Vec::new();

        for tipset in suffix.iter() {
            let parent_epoch = tipset.epoch;
            let child_epoch = parent_epoch + 1;

            tracing::debug!(parent_epoch, child_epoch, "Generating proof for epoch");

            // Generate proof for this epoch
            let proof_bundle = self
                .assembler
                .generate_proof_for_epoch(parent_epoch, child_epoch)
                .await
                .with_context(|| {
                    format!(
                        "Failed to generate proof for epoch {} (parent_cert={}, child_cert={})",
                        parent_epoch, pending_instance, child_instance
                    )
                })?;

            epoch_proofs.push(EpochProofEntry::new(
                parent_epoch,
                proof_bundle,
                pending_instance,
                child_instance,
            ));
        }

        // Cache all epoch proofs
        self.cache.insert_epoch_proofs(epoch_proofs)?;

        tracing::info!(
            pending_instance,
            child_instance,
            epoch_count = epochs.len(),
            "Successfully generated and cached proofs for all epochs"
        );

        Ok(())
    }

    /// Get reference to the cache (for proposers)
    pub fn cache(&self) -> &Arc<ProofCache> {
        &self.cache
    }
}

/// Check if two certificates have continuity (pending's last epoch + 1 == new's first epoch)
fn check_continuity(pending: &FinalityCertificate, new_cert: &FinalityCertificate) -> bool {
    let pending_last = pending.ec_chain.suffix().last().map(|t| t.epoch);
    let new_base = new_cert.ec_chain.base().map(|t| t.epoch);

    match (pending_last, new_base) {
        (Some(last), Some(base)) => {
            // The new cert's base should be the pending's last epoch
            // (F3 chains overlap at the boundary)
            last == base
        }
        (None, _) => {
            // Pending has empty suffix - just accept continuity
            true
        }
        _ => false,
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
    use crate::config::CacheConfig;

    #[tokio::test]
    async fn test_service_creation() {
        use filecoin_f3_gpbft::PowerEntries;

        let config = ProofServiceConfig {
            enabled: true,
            parent_rpc_url: "http://localhost:1234/rpc/v1".to_string(),
            gateway_id: GatewayId::ActorId(1001),
            cache_config: CacheConfig::default(),
            ..Default::default()
        };

        let cache = Arc::new(ProofCache::new(100, config.cache_config.clone()));
        let power_table = PowerEntries(vec![]);
        let subnet_id = SubnetID::default();

        // Note: Service creation succeeds with F3Client::new() even with a fake RPC endpoint
        // The actual RPC calls will fail later when the service tries to fetch certificates
        let result = ProofGeneratorService::new(config, cache, &subnet_id, 0, power_table).await;
        assert!(result.is_ok());
    }
}
