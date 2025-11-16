// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Configuration for the proof generator service

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the proof generator service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofServiceConfig {
    /// Enable/disable the service
    pub enabled: bool,

    /// Polling interval for checking parent chain
    #[serde(with = "humantime_serde")]
    pub polling_interval: Duration,

    /// How many instances ahead to generate proofs (lookahead window)
    pub lookahead_instances: u64,

    /// How many old instances to retain after commitment
    pub retention_instances: u64,

    /// Lotus/parent RPC endpoint URL
    pub parent_rpc_url: String,

    /// Parent subnet ID (e.g., "/r314159" for calibration)
    pub parent_subnet_id: String,

    /// F3 network name (e.g., "calibrationnet", "mainnet")
    pub f3_network_name: String,

    /// Optional: Additional RPC URLs for failover (not yet implemented - future enhancement)
    #[serde(default)]
    pub fallback_rpc_urls: Vec<String>,

    /// Maximum cache size in bytes (0 = unlimited)
    #[serde(default)]
    pub max_cache_size_bytes: usize,

    /// Gateway actor on parent chain (for proof generation).
    ///
    /// Can be either:
    /// - Actor ID: 176609
    /// - Ethereum address: 0xE4c61299c16323C4B58376b60A77F68Aa59afC8b (will be resolved to actor ID)
    ///
    /// Will be configured from subnet genesis info.
    #[serde(default)]
    pub gateway_actor_id: Option<u64>,

    /// Gateway ethereum address (alternative to gateway_actor_id).
    ///
    /// If provided, will be resolved to actor ID on service startup.
    #[serde(default)]
    pub gateway_eth_address: Option<String>,

    /// Subnet ID (for event filtering)
    /// Will be derived from genesis
    #[serde(default)]
    pub subnet_id: Option<String>,

    /// Maximum epoch lag before considering a certificate too old to generate proofs for.
    /// If a certificate's highest epoch is more than this many epochs behind the current
    /// parent chain epoch, it will be skipped.
    /// Default: 100 epochs (~50 minutes on Filecoin mainnet)
    #[serde(default = "default_max_epoch_lag")]
    pub max_epoch_lag: u64,

    /// Maximum lookback window that the parent RPC supports.
    /// Most Lotus nodes have a limited lookback window (e.g., 2000 epochs).
    /// If we're further behind than this, we can't generate proofs.
    /// Default: 2000 epochs
    #[serde(default = "default_rpc_lookback_limit")]
    pub rpc_lookback_limit: u64,
}

fn default_max_epoch_lag() -> u64 {
    100
}

fn default_rpc_lookback_limit() -> u64 {
    2000
}

impl Default for ProofServiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            polling_interval: Duration::from_secs(10),
            lookahead_instances: 5,
            retention_instances: 2,
            parent_rpc_url: String::new(),
            parent_subnet_id: String::new(),
            f3_network_name: "calibrationnet".to_string(),
            fallback_rpc_urls: Vec::new(),
            max_cache_size_bytes: 0,
            gateway_actor_id: None,
            gateway_eth_address: None,
            subnet_id: None,
            max_epoch_lag: default_max_epoch_lag(),
            rpc_lookback_limit: default_rpc_lookback_limit(),
        }
    }
}

/// Configuration for the proof cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Lookahead window
    pub lookahead_instances: u64,
    /// Retention window
    pub retention_instances: u64,
    /// Maximum size in bytes
    pub max_size_bytes: usize,
}

impl From<&ProofServiceConfig> for CacheConfig {
    fn from(config: &ProofServiceConfig) -> Self {
        Self {
            lookahead_instances: config.lookahead_instances,
            retention_instances: config.retention_instances,
            max_size_bytes: config.max_cache_size_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProofServiceConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.polling_interval, Duration::from_secs(10));
        assert_eq!(config.lookahead_instances, 5);
        assert_eq!(config.retention_instances, 2);
    }

    #[test]
    fn test_cache_config_from_service_config() {
        let service_config = ProofServiceConfig {
            lookahead_instances: 10,
            retention_instances: 3,
            max_cache_size_bytes: 1024,
            ..Default::default()
        };

        let cache_config = CacheConfig::from(&service_config);
        assert_eq!(cache_config.lookahead_instances, 10);
        assert_eq!(cache_config.retention_instances, 3);
        assert_eq!(cache_config.max_size_bytes, 1024);
    }
}
