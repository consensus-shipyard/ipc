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
pub mod persistence;
pub mod provider_manager;
pub mod service;
pub mod types;
pub mod watcher;

// Re-export main types for convenience
pub use cache::ProofCache;
pub use config::{CacheConfig, ProofServiceConfig};
pub use service::ProofGeneratorService;
pub use types::{CacheEntry, ValidatedCertificate};

use anyhow::{Context, Result};
use std::sync::Arc;

/// Initialize and launch the proof generator service
///
/// This is the main entry point for starting the service.
/// It creates the cache, initializes the service, and spawns the background task.
///
/// # Arguments
/// * `config` - Service configuration
/// * `initial_committed_instance` - The last committed F3 instance (from actor)
///
/// # Returns
/// * `Arc<ProofCache>` - Shared cache that proposers can query
/// * `tokio::task::JoinHandle` - Handle to the background service task
pub fn launch_service(
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

    // Create service outside of the async context
    let service = ProofGeneratorService::new(config, cache.clone())
        .context("Failed to create proof generator service")?;

    // Use spawn_blocking to run the service in a blocking thread pool
    // Then spawn an async task to handle it
    let handle = tokio::task::spawn_blocking(move || {
        // Create a new runtime for the blocking task
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        rt.block_on(async move {
            service.run().await;
        });
    });

    // Convert to a JoinHandle that looks like our original
    let handle = tokio::spawn(async move {
        let _ = handle.await;
    });

    Ok((cache, handle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_service_disabled() {
        let config = ProofServiceConfig {
            enabled: false,
            ..Default::default()
        };

        let result = launch_service(config, 0);
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires real parent chain RPC endpoint
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

        let result = launch_service(config, 100);
        assert!(result.is_ok());

        let (cache, handle) = result.unwrap();
        
        // Abort immediately to prevent the service from trying to connect
        handle.abort();
        
        // Check cache state
        assert_eq!(cache.last_committed_instance(), 100);
        assert_eq!(cache.len(), 0);
    }
}
