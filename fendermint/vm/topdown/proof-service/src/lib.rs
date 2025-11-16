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
pub mod observe;
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

use anyhow::{Context, Result};
use std::sync::Arc;

/// Initialize and launch the proof generator service
///
/// This is the main entry point for starting the service.
/// It creates the cache, initializes the service, and spawns the background task.
///
/// # Arguments
/// * `config` - Service configuration
/// * `initial_committed_instance` - The last committed F3 instance (from F3CertManager actor)
/// * `initial_power_table` - Initial power table (from F3CertManager actor)
/// * `db_path` - Optional database path for persistence
///
/// # Returns
/// * `Arc<ProofCache>` - Shared cache that proposers can query
/// * `tokio::task::JoinHandle` - Handle to the background service task
pub async fn launch_service(
    config: ProofServiceConfig,
    initial_committed_instance: u64,
    initial_power_table: filecoin_f3_gpbft::PowerEntries,
    db_path: Option<std::path::PathBuf>,
) -> Result<(Arc<ProofCache>, tokio::task::JoinHandle<()>)> {
    // Validate configuration
    if !config.enabled {
        anyhow::bail!("Proof service is disabled in configuration");
    }

    if config.parent_rpc_url.is_empty() {
        anyhow::bail!("parent_rpc_url is required");
    }

    if config.f3_network_name.is_empty() {
        anyhow::bail!("f3_network_name is required (e.g., 'calibrationnet', 'mainnet')");
    }

    if config.lookahead_instances == 0 {
        anyhow::bail!("lookahead_instances must be > 0");
    }

    if config.retention_instances == 0 {
        anyhow::bail!("retention_instances must be > 0");
    }

    // Validate URL format
    url::Url::parse(&config.parent_rpc_url)
        .with_context(|| format!("Invalid parent_rpc_url: {}", config.parent_rpc_url))?;

    tracing::info!(
        initial_instance = initial_committed_instance,
        parent_rpc = config.parent_rpc_url,
        f3_network = config.f3_network_name,
        lookahead = config.lookahead_instances,
        "Launching proof generator service with validated configuration"
    );

    // Create cache (with optional persistence)
    let cache_config = CacheConfig::from(&config);
    let cache = if let Some(path) = db_path {
        tracing::info!(path = %path.display(), "Creating cache with persistence");
        Arc::new(ProofCache::new_with_persistence(
            cache_config,
            &path,
            initial_committed_instance,
        )?)
    } else {
        tracing::info!("Creating in-memory cache (no persistence)");
        Arc::new(ProofCache::new(initial_committed_instance, cache_config))
    };

    // Clone what we need for the background task
    let config_clone = config.clone();
    let cache_clone = cache.clone();
    let power_table_clone = initial_power_table.clone();

    // Spawn background task
    let handle = tokio::spawn(async move {
        match ProofGeneratorService::new(
            config_clone,
            cache_clone,
            initial_committed_instance,
            power_table_clone,
        )
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
        use filecoin_f3_gpbft::PowerEntries;

        let config = ProofServiceConfig {
            enabled: false,
            ..Default::default()
        };

        let power_table = PowerEntries(vec![]);
        let result = launch_service(config, 0, power_table, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_launch_service_enabled() {
        use filecoin_f3_gpbft::PowerEntries;

        let config = ProofServiceConfig {
            enabled: true,
            parent_rpc_url: "http://localhost:1234/rpc/v1".to_string(),
            parent_subnet_id: "/r314159".to_string(),
            f3_network_name: "calibrationnet".to_string(),
            gateway_actor_id: Some(1001),
            subnet_id: Some("test-subnet".to_string()),
            polling_interval: std::time::Duration::from_secs(60),
            ..Default::default()
        };

        let power_table = PowerEntries(vec![]);
        let result = launch_service(config, 100, power_table, None).await;
        assert!(result.is_ok());

        let (cache, handle) = result.unwrap();
        handle.abort();

        assert_eq!(cache.last_committed_instance(), 100);
    }
}
