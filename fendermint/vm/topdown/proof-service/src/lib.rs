// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Proof generator service for F3-based parent finality
//!
//! This crate implements a background service that:
//! - Monitors the parent chain for new F3 certificates
//! - Generates proof bundles ahead of time
//! - Caches proofs for instant use by block proposers
//! - Ensures sequential processing of F3 instances

pub mod assembler;
pub mod cache;
pub mod config;
pub mod f3_client;
pub mod parent_client;
pub mod persistence;
pub mod service;
pub mod types;
pub mod verifier;

// Re-export main types for convenience
pub use cache::ProofCache;
pub use config::{CacheConfig, ProofServiceConfig};
pub use service::ProofGeneratorService;
pub use types::{CacheEntry, SerializableF3Certificate, ValidatedCertificate};
pub use verifier::verify_proof_bundle;

use anyhow::Result;
use std::sync::Arc;

/// Initialize and launch the proof generator service
///
/// This is the main entry point for starting the service.
/// It creates the cache, initializes the service, and spawns the background task.
///
/// # Arguments
/// * `config` - Service configuration
/// * `initial_committed_instance` - The last committed F3 instance (from F3CertManager actor)
///
/// # Returns
/// * `Arc<ProofCache>` - Shared cache that proposers can query
/// * `tokio::task::JoinHandle` - Handle to the background service task
///
/// # Note
/// This function fetches the initial power table from RPC for MVP.
/// In production, the power table should come from the F3CertManager actor.
pub async fn launch_service(
    config: ProofServiceConfig,
    initial_committed_instance: u64,
) -> Result<(Arc<ProofCache>, tokio::task::JoinHandle<()>)> {
    if !config.enabled {
        anyhow::bail!("Proof service is disabled in configuration");
    }

    tracing::info!(
        initial_instance = initial_committed_instance,
        parent_rpc = config.parent_rpc_url,
        "Launching proof generator service"
    );

    // Create cache
    let cache_config = CacheConfig::from(&config);
    let cache = Arc::new(ProofCache::new(initial_committed_instance, cache_config));

    // TODO: When BLS deps resolved, fetch power table from F3CertManager actor
    // For MVP, power table not needed (structural validation only)

    // Clone what we need for the background task
    let config_clone = config.clone();
    let cache_clone = cache.clone();

    // Spawn background task
    let handle = tokio::spawn(async move {
        match ProofGeneratorService::new(config_clone, cache_clone, initial_committed_instance)
            .await
        {
            Ok(service) => service.run().await,
            Err(e) => {
                tracing::error!(error = %e, "Failed to create proof generator service");
            }
        }
    });

    Ok((cache, handle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_launch_service_disabled() {
        let config = ProofServiceConfig {
            enabled: false,
            ..Default::default()
        };

        let result = launch_service(config, 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_launch_service_enabled() {
        let config = ProofServiceConfig {
            enabled: true,
            parent_rpc_url: "http://localhost:1234/rpc/v1".to_string(),
            parent_subnet_id: "/r314159".to_string(),
            gateway_actor_id: Some(1001),
            subnet_id: Some("test-subnet".to_string()),
            polling_interval: std::time::Duration::from_secs(60),
            ..Default::default()
        };

        let result = launch_service(config, 100).await;
        assert!(result.is_ok());

        let (cache, handle) = result.unwrap();
        handle.abort();

        assert_eq!(cache.last_committed_instance(), 100);
    }
}
