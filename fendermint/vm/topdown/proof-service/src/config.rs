// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Configuration for the proof generator service

use ipc_api::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const FILECOIN_MAINNET_CHAIN_ID: u64 = 314;
const FILECOIN_CALIBRATION_CHAIN_ID: u64 = 314159;

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
}

impl ProofServiceConfig {
    pub fn f3_network_name(&self, subnet_id: &SubnetID) -> String {
        let root_id = subnet_id.root_id();

        match root_id {
            FILECOIN_MAINNET_CHAIN_ID => "mainnet".to_string(),
            FILECOIN_CALIBRATION_CHAIN_ID => "calibrationnet".to_string(),
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
        }
    }
}

/// Configuration for the proof cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// How many epochs ahead to generate proofs for
    /// This determines how far ahead of the last committed epoch we pre-generate proofs
    pub lookahead_epochs: u64,
    
    /// How many epochs to retain after they've been committed
    /// Old epochs outside this window will be cleaned up
    pub retention_epochs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            // Default: generate proofs for ~50 epochs ahead (~25 minutes at 30s/epoch)
            lookahead_epochs: 50,
            // Default: keep proofs for 10 epochs after commit
            retention_epochs: 10,
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
        assert_eq!(config.cache_config.lookahead_epochs, 50);
        assert_eq!(config.cache_config.retention_epochs, 10);
    }
}
