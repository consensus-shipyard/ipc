// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Integration tests for the proof cache service

use fendermint_vm_topdown_proof_service::{launch_service, ProofServiceConfig};
use std::time::Duration;

#[tokio::test]
#[ignore] // Run with: cargo test --ignored
async fn test_proof_generation_from_calibration() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fendermint_vm_topdown_proof_service=debug".parse().unwrap()),
        )
        .init();
    
    // Use calibration testnet
    let config = ProofServiceConfig {
        enabled: true,
        parent_rpc_url: "https://api.calibration.node.glif.io/rpc/v1".to_string(),
        parent_subnet_id: "/r314159".to_string(),
        subnet_id: Some("test-subnet".to_string()),
        gateway_actor_id: Some(1001),
        lookahead_instances: 2,
        polling_interval: Duration::from_secs(5),
        retention_instances: 1,
        max_cache_size_bytes: 0, // Unlimited
        fallback_rpc_urls: vec![],
    };
    
    // Get current F3 instance from chain to start from valid point
    // For MVP, we'll start from instance 0
    let initial_instance = 0;
    
    println!("Starting proof service from instance {}...", initial_instance);
    let (cache, handle) = launch_service(config, initial_instance)
        .expect("Failed to launch service");
    
    println!("Service launched successfully!");
    
    // Wait for certificates to be fetched and validated
    println!("Waiting for F3 certificates and proofs...");
    for i in 1..=6 {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let cache_size = cache.len();
        println!("[{}s] Cache has {} entries", i * 5, cache_size);
        
        if cache_size > 0 {
            println!("✓ Successfully generated some proofs!");
            break;
        }
    }
    
    // Check that we have some proofs
    let cache_size = cache.len();
    println!("Final cache size: {} entries", cache_size);
    
    // Note: For MVP, we're not expecting real proofs yet since we're using placeholders
    // But we should at least have the cache working
    
    // Verify cache structure
    if let Some(entry) = cache.get_next_uncommitted() {
        println!("✓ Got proof for instance {}", entry.instance_id);
        println!("✓ Epochs: {:?}", entry.finalized_epochs);
        assert!(!entry.finalized_epochs.is_empty(), "Should have epochs");
        assert!(!entry.proof_bundle_bytes.is_empty(), "Should have proof bundle");
    } else {
        println!("Note: No uncommitted proofs yet (expected for MVP)");
    }
    
    // Clean up
    handle.abort();
    println!("Test completed!");
}

#[tokio::test]
async fn test_cache_operations() {
    use fendermint_vm_topdown_proof_service::{cache::ProofCache, config::CacheConfig};
    
    // Create a cache
    let config = CacheConfig {
        lookahead_instances: 5,
        retention_instances: 2,
        max_size_bytes: 0,
    };
    
    let cache = ProofCache::new(100, config);
    
    // Check initial state
    assert_eq!(cache.last_committed_instance(), 100);
    assert_eq!(cache.len(), 0);
    
    // Note: We can't easily test insertion without creating proper CacheEntry objects
    // which requires the full service setup. This is mostly a placeholder test.
    
    println!("✓ Basic cache operations work");
}
