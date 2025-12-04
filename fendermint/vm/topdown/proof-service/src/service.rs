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
use crate::types::{CertificateEntry, EpochProofEntry, FinalizedTipset};
use anyhow::{Context, Result};
use filecoin_f3_certs::FinalityCertificate;
use ipc_api::subnet_id::SubnetID;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{interval, MissedTickBehavior};

/// Main proof generator service
pub struct ProofGeneratorService {
    config: ProofServiceConfig,
    cache: Arc<ProofCache>,
    f3_client: Arc<F3Client>,
    assembler: Arc<ProofAssembler>,

    /// The certificate waiting for its child to be finalized
    /// When the next certificate arrives, we can process this one's epochs
    pending_certificate: Option<PendingCertificate>,
}

/// A certificate waiting for its child to be finalized before we can generate proofs.
/// We use a type alias for clarity - this certificate's epochs will be processed
/// when the next certificate arrives.
type PendingCertificate = FinalityCertificate;

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
            pending_certificate: None,
        })
    }

    /// Main service loop - runs continuously and polls parent chain periodically
    ///
    /// Each tick processes ONE certificate (if needed and available).
    /// The ticker acts as the outer loop - no inner loop needed.
    /// Errors are logged but don't stop the service - it will retry on next tick.
    pub async fn run(mut self) {
        tracing::info!(
            polling_interval = ?self.config.polling_interval,
            lookahead_instances = self.config.cache_config.lookahead_instances,
            "Starting proof generator service"
        );

        let mut poll_interval = interval(self.config.polling_interval);
        poll_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            poll_interval.tick().await;

            if let Err(e) = self.process_next_certificate().await {
                tracing::error!(
                    error = %e,
                    "Failed to process certificate, will retry on next tick"
                );
            }
        }
    }

    /// Process next certificate if we haven't reached the lookahead target.
    ///
    /// This is the main tick handler - processes at most one certificate per call.
    /// The ticker in `run()` provides the outer loop.
    ///
    /// # Future improvements
    /// TODO: Gap recovery could be added when multiple RPC endpoints are available.
    async fn process_next_certificate(&mut self) -> Result<()> {
        if !self.should_fetch_more().await {
            return Ok(());
        }

        let Some(new_cert) = self.fetch_next_certificate().await? else {
            return Ok(()); // No certificate available, caught up with F3
        };

        // Process pending certificate if we have one
        if let Some(pending) = self.pending_certificate.take() {
            self.process_pending(&pending, &new_cert).await?;
        }

        // Store new certificate as pending (will be processed on next tick)
        self.cache_and_store_pending(new_cert).await?;

        Ok(())
    }

    /// Check if we should fetch more certificates based on lookahead.
    async fn should_fetch_more(&self) -> bool {
        let current_instance = self.f3_client.current_instance().await;
        let last_committed = self.cache.last_committed_instance();
        let lookahead = self.config.cache_config.lookahead_instances;
        let target = last_committed + lookahead;

        if current_instance >= target {
            tracing::debug!(
                current_instance,
                last_committed,
                target,
                "Already at lookahead target, nothing to do"
            );
            false
        } else {
            true
        }
    }

    /// Fetch and validate the next certificate from F3.
    /// Returns `None` if no certificate is available (caught up).
    async fn fetch_next_certificate(&self) -> Result<Option<FinalityCertificate>> {
        match self.f3_client.fetch_and_validate().await {
            Ok(cert) => {
                self.log_certificate(&cert);
                Ok(Some(cert))
            }
            Err(err) if is_certificate_unavailable(&err) => {
                tracing::debug!("Caught up with F3 - no more certificates available");
                Ok(None)
            }
            Err(err) => Err(err).context("Failed to fetch and validate certificate"),
        }
    }

    /// Log certificate info for debugging.
    fn log_certificate(&self, cert: &FinalityCertificate) {
        let suffix_epochs: Vec<i64> = cert.ec_chain.suffix().iter().map(|ts| ts.epoch).collect();
        tracing::info!(
            instance = cert.gpbft_instance,
            base_epoch = ?cert.ec_chain.base().map(|b| b.epoch),
            suffix_epochs = ?suffix_epochs,
            "Fetched and validated certificate"
        );
    }

    /// Process a pending certificate now that we have its child.
    async fn process_pending(
        &self,
        pending: &PendingCertificate,
        child: &FinalityCertificate,
    ) -> Result<()> {
        if !check_continuity(pending, child) {
            return Err(continuity_error(pending, child));
        }
        self.generate_proofs_for_certificate(pending, child).await
    }

    /// Cache a certificate and store it as pending for next tick.
    async fn cache_and_store_pending(&mut self, cert: FinalityCertificate) -> Result<()> {
        let power_table = self.f3_client.get_state().await.power_table;
        let rpc_endpoint = self.f3_client.rpc_endpoint();

        let entry = CertificateEntry::new(cert.clone(), power_table, rpc_endpoint);
        self.cache.insert_certificate(entry)?;

        self.pending_certificate = Some(cert);
        Ok(())
    }

    /// Generate proofs for all epochs in a certificate's suffix.
    async fn generate_proofs_for_certificate(
        &self,
        cert: &FinalityCertificate,
        child_cert: &FinalityCertificate,
    ) -> Result<()> {
        let suffix = cert.ec_chain.suffix();
        if suffix.is_empty() {
            tracing::debug!(
                instance = cert.gpbft_instance,
                "Certificate has empty suffix, nothing to prove"
            );
            return Ok(());
        }

        let epochs: Vec<i64> = suffix.iter().map(|ts| ts.epoch).collect();
        tracing::info!(
            parent_instance = cert.gpbft_instance,
            child_instance = child_cert.gpbft_instance,
            epochs = ?epochs,
            "Generating proofs for certificate epochs"
        );

        // Build epoch -> tipset lookup from both certificates
        let tipset_map: HashMap<i64, FinalizedTipset> = cert
            .ec_chain
            .iter()
            .chain(child_cert.ec_chain.iter())
            .map(|ts| (ts.epoch, FinalizedTipset::from(ts)))
            .collect();

        // Generate proofs for each epoch
        let epoch_proofs = self
            .generate_epoch_proofs(
                suffix,
                &tipset_map,
                cert.gpbft_instance,
                child_cert.gpbft_instance,
            )
            .await?;

        self.cache.insert_epoch_proofs(epoch_proofs)?;

        tracing::info!(
            parent_instance = cert.gpbft_instance,
            child_instance = child_cert.gpbft_instance,
            epoch_count = epochs.len(),
            "Successfully generated and cached proofs"
        );

        Ok(())
    }

    /// Generate proofs for each epoch in the suffix.
    async fn generate_epoch_proofs(
        &self,
        finalized_epochs: &[filecoin_f3_gpbft::Tipset],
        tipset_map: &HashMap<i64, FinalizedTipset>,
        parent_cert_instance: u64,
        child_cert_instance: u64,
    ) -> Result<Vec<EpochProofEntry>> {
        let mut proofs = Vec::with_capacity(finalized_epochs.len());

        for tipset in finalized_epochs.iter() {
            let parent_epoch = tipset.epoch;
            let child_epoch = parent_epoch + 1;

            let parent_tipset = tipset_map
                .get(&parent_epoch)
                .cloned()
                .context("Parent tipset not found in certificate chain")?;
            let child_tipset = tipset_map
                .get(&child_epoch)
                .cloned()
                .context("Child tipset not found in certificate chain")?;

            tracing::debug!(parent_epoch, child_epoch, "Generating proof for epoch");

            let proof_bundle = self
                .assembler
                .generate_proof_for_epoch(parent_tipset, child_tipset)
                .await
                .with_context(|| format!("Failed to generate proof for epoch {}", parent_epoch))?;

            proofs.push(EpochProofEntry::new(
                parent_epoch,
                proof_bundle,
                parent_cert_instance,
                child_cert_instance,
            ));
        }

        Ok(proofs)
    }

    /// Get reference to the cache (for proposers)
    pub fn cache(&self) -> &Arc<ProofCache> {
        &self.cache
    }
}

/// Check if two certificates have continuity (pending's last epoch == new's base epoch).
fn check_continuity(pending: &FinalityCertificate, new_cert: &FinalityCertificate) -> bool {
    let pending_last = pending.ec_chain.suffix().last().map(|t| t.epoch);
    let new_base = new_cert.ec_chain.base().map(|t| t.epoch);

    match (pending_last, new_base) {
        (Some(last), Some(base)) => last == base, // F3 chains overlap at boundary
        (None, _) => true,                        // Empty suffix - accept continuity
        _ => false,
    }
}

/// Build a detailed error for certificate continuity breaks.
///
/// This is a fatal error - the F3 chain should always be continuous.
/// TODO: With multiple RPC endpoints, we could attempt gap recovery.
fn continuity_error(
    pending: &FinalityCertificate,
    new_cert: &FinalityCertificate,
) -> anyhow::Error {
    let pending_last = pending.ec_chain.suffix().last().map(|t| t.epoch);
    let new_base = new_cert.ec_chain.base().map(|t| t.epoch);

    anyhow::anyhow!(
        "Certificate continuity broken: instance {} (last epoch {:?}) does not connect to \
         instance {} (base epoch {:?}). Service may need re-bootstrap.",
        pending.gpbft_instance,
        pending_last,
        new_cert.gpbft_instance,
        new_base,
    )
}

/// Check if an error indicates the certificate is not yet available.
fn is_certificate_unavailable(err: &anyhow::Error) -> bool {
    let msg = err.to_string();
    msg.contains("not found") || msg.contains("not available")
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

        let cache = Arc::new(ProofCache::new(100, 0, config.cache_config.clone()));
        let power_table = PowerEntries(vec![]);
        let subnet_id = SubnetID::default();

        // Note: Service creation succeeds with F3Client::new() even with a fake RPC endpoint
        // The actual RPC calls will fail later when the service tries to fetch certificates
        let result = ProofGeneratorService::new(config, cache, &subnet_id, 0, power_table).await;
        assert!(result.is_ok());
    }
}
