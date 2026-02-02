// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof generator service - orchestrates proof generation pipeline
//!
//! # Architecture
//!
//! Each F3 certificate contains tipsets [base, suffix...] where:
//! - `base` = last finalized epoch from previous certificate (overlap point)
//! - `suffix` = new epochs being finalized
//!
//! For each certificate, we generate proofs for all (parent, child) pairs:
//! - Given [E0, E1, E2, E3], we prove E0, E1, E2 (E3 has no child yet)
//! - E3 will be proven when next certificate arrives (as its base)
//!
//! Each proof requires both parent (epoch E) and child (typically epoch E+1) because
//! Filecoin stores `parentReceipts` in the child block, not the parent.

use crate::assembler::{resolve_eth_address_to_actor_id, ProofAssembler};
use crate::cache::ProofCache;
use crate::config::{GatewayId, ProofServiceConfig};
use crate::f3_client::F3Client;
use crate::types::{CertificateEntry, EpochProofEntry, FinalizedTipset, FinalizedTipsets};
use crate::verifier::ProofVerifier;
use anyhow::{Context, Result};
use filecoin_f3_certs::FinalityCertificate;
use filecoin_f3_gpbft::PowerEntries;
use ipc_api::subnet_id::SubnetID;
use std::sync::Arc;
use tokio::time::{interval, MissedTickBehavior};

/// Main proof generator service
pub struct ProofGeneratorService {
    config: ProofServiceConfig,
    cache: Arc<ProofCache>,
    f3_client: F3Client,
    assembler: ProofAssembler,
    verifier: ProofVerifier,
}

impl ProofGeneratorService {
    /// Create a new proof generator service
    ///
    /// # Arguments
    /// * `config` - Service configuration
    /// * `cache` - Proof cache
    /// * `subnet_id` - id of the subnet to prove
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
        initial_power_table: PowerEntries,
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
        let f3_client = F3Client::new(
            &config.parent_rpc_url,
            &config.f3_network_name(subnet_id),
            start_instance,
            start_power_table,
        )
        .context("Failed to create F3 client")?;

        // Create proof assembler
        let assembler = ProofAssembler::new(
            config.parent_rpc_url.clone(),
            gateway_actor_id,
            subnet_id.to_string(),
        )
        .context("Failed to create proof assembler")?;

        Ok(Self {
            config,
            cache,
            f3_client,
            assembler,
            verifier: ProofVerifier::new(subnet_id.to_string()),
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
        if !self.should_fetch_more() {
            return Ok(());
        }

        // Provide *all-or-nothing* semantics per certificate.
        //
        // `fetch_next_certificate()` advances the internal F3 light-client state to the newly
        // validated instance. If we fail later while generating/verifying/caching proofs, we MUST
        // roll back that state; otherwise the next tick would fetch the next instance and we'd
        // permanently skip this certificate, leaving a cache hole that can stall catch-up.
        let checkpoint = self.f3_client.checkpoint_state();

        let Some((certificate, power_table)) = self.fetch_next_certificate().await? else {
            return Ok(()); // No certificate available, caught up with F3
        };

        if let Err(e) = self
            .generate_proofs_for_certificate(&certificate, &power_table)
            .await
        {
            tracing::error!(
                error = %e,
                instance = certificate.gpbft_instance,
                "failed to generate/verify proofs for certificate; rolling back and retrying later"
            );
            self.f3_client.restore_state(checkpoint);
            return Err(e);
        }

        Ok(())
    }

    /// Check if we should fetch more certificates based on lookahead.
    fn should_fetch_more(&self) -> bool {
        let current_instance = self.f3_client.current_instance();
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
    async fn fetch_next_certificate(
        &mut self,
    ) -> Result<Option<(FinalityCertificate, PowerEntries)>> {
        match self.f3_client.fetch_and_validate().await {
            Ok((cert, power_table)) => {
                self.log_certificate(&cert);
                Ok(Some((cert, power_table)))
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

    /// Generate proofs for all (parent, child) tipset pairs in the certificate.
    ///
    /// Each proof requires both the parent tipset (epoch E) and child tipset (typically epoch E+1).
    /// The child contains `parentReceipts` which commits to the parent's execution results.
    ///
    /// Given tipsets [E0, E1, E2, E3], we generate proofs for:
    /// - E0 (using E1 as child)
    /// - E1 (using E2 as child)  
    /// - E2 (using E3 as child)
    /// - E3 has no child in this certificate, will be proven with next certificate
    async fn generate_proofs_for_certificate(
        &self,
        cert: &FinalityCertificate,
        power_table: &PowerEntries,
    ) -> Result<()> {
        // Build (parent, child) pairs using windows - this makes the requirement explicit
        let tipset_pairs: Vec<_> = cert
            .ec_chain
            .iter()
            .map(FinalizedTipset::from)
            .collect::<Vec<_>>()
            .windows(2)
            .map(|w| (w[0].clone(), w[1].clone()))
            .collect();

        if tipset_pairs.is_empty() {
            tracing::debug!(
                instance = cert.gpbft_instance,
                "Certificate has fewer than 2 tipsets, no (parent, child) pairs to prove"
            );
            return Ok(());
        }

        let epochs_to_prove: Vec<i64> = tipset_pairs.iter().map(|(p, _)| p.epoch).collect();

        tracing::info!(
            instance = cert.gpbft_instance,
            epochs = ?epochs_to_prove,
            "Generating proofs for certificate epochs"
        );

        // Verification needs to accept witness blocks from *both* the parent and the child tipset
        // of each (parent, child) pair (receipts/state for the parent live in the child).
        // Therefore, pass the whole certified chain from the certificate.
        //
        // Note: we still only *generate* proofs for the parent epochs via `windows(2)`, so the
        // last tipset in the chain (which has no child in this certificate) is not proven yet.
        let finalized_tipsets = FinalizedTipsets::from(&cert.ec_chain);

        let mut epoch_proofs = Vec::with_capacity(tipset_pairs.len());
        let mut cursor: Option<crate::verifier::EventNumberCursor> = None;

        // Generate proofs for each (parent, child) pair.
        // The child tipset contains `parentReceipts` which commits to the parent's execution.
        for (parent_tipset, child_tipset) in tipset_pairs {
            let parent_epoch = parent_tipset.epoch;

            tracing::debug!(
                parent_epoch,
                child_epoch = child_tipset.epoch,
                "Generating proof for epoch"
            );

            let proof_bundle = self
                .assembler
                .generate_proof_for_epoch(parent_tipset.clone(), child_tipset.clone())
                .await
                .with_context(|| format!("Failed to generate proof for epoch {}", parent_epoch))?;

            self.verifier
                .verify_proof_bundle_with_tipsets(&proof_bundle, &finalized_tipsets)
                .with_context(|| format!("Failed to verify proof for epoch {}", parent_epoch))?;

            // Additional semantic checks:
            // - top-down message nonce continuity (from decoded events)
            // - power change configurationNumber continuity, and consistency with proved storage slot
            // Maintain a cursor across epochs within this certificate so we can detect omitted
            // events at the beginning of an epoch (anchored to proved end-of-epoch storage).
            //
            // Note: the first epoch proven in a certificate does not have a previous cursor.
            self.verifier
                .verify_event_number_continuity(parent_epoch, &proof_bundle, &mut cursor)
                .with_context(|| {
                    format!(
                        "Nonce/config continuity check failed for epoch {}",
                        parent_epoch
                    )
                })?;

            epoch_proofs.push(EpochProofEntry::new(
                parent_epoch,
                proof_bundle,
                cert.gpbft_instance,
            ));
        }

        // Cache the certificate and proofs
        let rpc_endpoint = self.f3_client.rpc_endpoint().to_string();
        let cert_entry = CertificateEntry::new(cert.clone(), power_table.clone(), rpc_endpoint);
        self.cache
            .insert_certificate_with_epoch_proofs(cert_entry, epoch_proofs)?;

        tracing::info!(
            epoch_count = epochs_to_prove.len(),
            "Successfully generated and cached proofs"
        );

        Ok(())
    }

    /// Get reference to the cache (for proposers)
    pub fn cache(&self) -> &Arc<ProofCache> {
        &self.cache
    }
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
            resolve_eth_address_to_actor_id(&config.parent_rpc_url, eth_addr).await
        }
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
