// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Configuration for the proof generator service

use ipc_api::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const FILECOIN_MAINNET_CHAIN_ID: u64 = 314;
const FILECOIN_CALIBRATION_CHAIN_ID: u64 = 314159;

/// Derive the F3 network name from the subnet root chain ID.
///
/// This is used for interacting with the Filecoin F3 RPC.
pub fn f3_network_name(subnet_id: &SubnetID) -> String {
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

    /// Gateway identification on parent chain.
    /// Can be an Actor ID (u64) or an Ethereum address (String).
    pub gateway_id: GatewayId,
}

impl ProofServiceConfig {
    /// Validate the configuration.
    ///
    /// Returns an error if any required fields are missing or invalid.
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(()); // No validation needed if disabled
        }

        if self.parent_rpc_url.is_empty() {
            anyhow::bail!("parent_rpc_url is required when service is enabled");
        }

        url::Url::parse(&self.parent_rpc_url).map_err(|e| {
            anyhow::anyhow!("Invalid parent_rpc_url '{}': {}", self.parent_rpc_url, e)
        })?;

        if self.cache_config.lookahead_instances == 0 {
            anyhow::bail!("lookahead_instances must be > 0");
        }

        Ok(())
    }

    pub fn f3_network_name(&self, subnet_id: &SubnetID) -> String {
        f3_network_name(subnet_id)
    }
}

impl Default for ProofServiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            polling_interval: Duration::from_secs(10),
            cache_config: Default::default(),
            parent_rpc_url: String::new(),
            gateway_id: GatewayId::ActorId(0),
        }
    }
}

/// Configuration for the proof cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// How many F3 instances ahead of last_committed_instance to stay.
    /// The service will stop fetching new certificates once:
    ///   current_instance >= last_committed_instance + lookahead_instances
    pub lookahead_instances: u64,

    /// How many epochs to retain after they've been committed.
    /// Old epochs outside this window will be cleaned up.
    pub retention_epochs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            // Default: stay ~20 instances ahead
            lookahead_instances: 20,
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
        assert_eq!(config.cache_config.lookahead_instances, 20);
        assert_eq!(config.cache_config.retention_epochs, 10);
    }
}
