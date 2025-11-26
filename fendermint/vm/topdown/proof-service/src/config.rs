// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Configuration for the proof generator service

use ipc_api::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Represents a value that can be either a numeric Actor ID or an Ethereum address string.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GatewayId {
    /// Actor ID (u64)
    ActorId(u64),
    /// Ethereum address (String)
    EthAddress(String),
}

/// Configuration for the proof generator service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofServiceConfig {
    /// Enable/disable the service
    pub enabled: bool,

    /// Polling interval for checking parent chain
    #[serde(with = "humantime_serde")]
    pub polling_interval: Duration,

    /// Configuration for the proof cache
    pub cache_config: CacheConfig,

    /// Lotus/parent RPC endpoint URL
    pub parent_rpc_url: String,

    /// Optional: Additional RPC URLs for failover (not yet implemented - future enhancement)
    #[serde(default)]
    pub fallback_rpc_urls: Vec<String>,

    /// Gateway identification on parent chain.
    /// Can be an Actor ID (u64) or an Ethereum address (String).
    pub gateway_id: GatewayId,

    /// Subnet ID (for event filtering)
    /// Will be derived from genesis
    pub subnet_id: SubnetID,
}

impl ProofServiceConfig {
    pub fn f3_network_name(&self) -> String {
        let root_id = self.subnet_id.root_id();

        match root_id {
            314 => "mainnet".to_string(),
            314159 => "calibrationnet".to_string(),
            _ => {
                tracing::warn!(
                    root_id,
                    "Unknown root chain ID for F3, defaulting to calibrationnet"
                );
                "calibrationnet".to_string()
            }
        }
    }
}

impl Default for ProofServiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            polling_interval: Duration::from_secs(10),
            cache_config: Default::default(),
            parent_rpc_url: String::new(),
            fallback_rpc_urls: Vec::new(),
            gateway_id: GatewayId::ActorId(0),
            subnet_id: SubnetID::default(),
        }
    }
}

/// Configuration for the proof cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Lookahead window
    pub lookahead_instances: u64,
    /// Retention window
    pub retention_instances: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            lookahead_instances: 5,
            retention_instances: 2,
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
        assert_eq!(config.cache_config.lookahead_instances, 5);
        assert_eq!(config.cache_config.retention_instances, 2);
    }
}
