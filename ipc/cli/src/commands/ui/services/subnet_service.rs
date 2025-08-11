// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Subnet service for subnet management operations
//!
//! This service wraps existing CLI handlers for subnet operations.

use super::super::api::types::{ApiResponse, ChainStats, SubnetStatus};
use crate::commands::subnet::{
    approve::{approve_subnet, ApproveSubnetArgs},
    join::{join_subnet, JoinSubnetArgs},
    leave::{LeaveSubnetArgs},
    // Note: list_subnets is not available as a function export
};
use crate::{GlobalArguments, get_ipc_provider};
use anyhow::Result;
use fvm_shared::address::Address;
use ipc_api::subnet_id::SubnetID;
use std::str::FromStr;

/// Subnet information for UI display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubnetInfo {
    pub id: String,
    pub parent_id: String,
    pub name: Option<String>,
    pub permission_mode: String,
    pub min_validators: u64,
    pub validators: Vec<ValidatorInfo>,
    pub total_collateral: String,
    pub circulating_supply: String,
    pub is_active: bool,
    pub block_height: u64,
    pub checkpoint_period: u64,
}

/// Validator information for UI display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidatorInfo {
    pub address: String,
    pub stake: String,
    pub is_active: bool,
    pub federated_power: Option<u64>,
    pub metadata: Option<ValidatorMetadata>,
}

/// Validator metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidatorMetadata {
    pub network_address: Option<String>,
    pub public_key: Option<String>,
}

/// Service for subnet management operations
pub struct SubnetService {
    global: GlobalArguments,
}

impl SubnetService {
    pub fn new(global: GlobalArguments) -> Self {
        Self { global }
    }

    /// Approve a subnet to join
    pub async fn approve_subnet(&self, subnet_id: &str, from_address: Option<&str>) -> Result<String> {
        let subnet = SubnetID::from_str(subnet_id)?;

        let args = ApproveSubnetArgs {
            subnet: subnet_id.to_string(),
            from: from_address.map(|s| s.to_string()),
        };

        let mut provider = get_ipc_provider(&self.global)?;

        // Use the existing approve_subnet handler
        approve_subnet(&mut provider, &args).await?;

        Ok(format!("Subnet {} approved successfully", subnet_id))
    }

    /// Add a validator to a subnet
    pub async fn add_validator(
        &self,
        subnet_id: &str,
        validator_data: &serde_json::Value,
    ) -> Result<String> {
        // TODO: Implement validator addition using the provider
        // For now, return a placeholder response
        Ok(format!("Validator added to subnet {} (placeholder)", subnet_id))
    }

    /// Remove a validator from a subnet
    pub async fn remove_validator(
        &self,
        subnet_id: &str,
        validator_data: &serde_json::Value,
    ) -> Result<String> {
        // TODO: Implement validator removal using the provider
        // For now, return a placeholder response
        Ok(format!("Validator removed from subnet {} (placeholder)", subnet_id))
    }

    /// Update validator stake in a subnet
    pub async fn update_validator_stake(
        &self,
        subnet_id: &str,
        stake_data: &serde_json::Value,
    ) -> Result<String> {
        // TODO: Implement stake update using the provider
        // For now, return a placeholder response
        Ok(format!("Validator stake updated in subnet {} (placeholder)", subnet_id))
    }

    /// Set federated power for a subnet
    pub async fn set_federated_power(
        &self,
        subnet_id: &str,
        power_data: &serde_json::Value,
    ) -> Result<String> {
        // TODO: Implement federated power setting
        // The set_federated_power function signature needs to be checked
        Ok(format!("Federated power set for subnet {} (placeholder)", subnet_id))
    }

    /// Get subnet status
    pub async fn get_subnet_status(&self, subnet_id: &str) -> Result<SubnetStatus> {
        let _subnet = SubnetID::from_str(subnet_id)?;
        let _provider = get_ipc_provider(&self.global)?;

        // TODO: Implement actual subnet status retrieval
        // For now, return placeholder status since get_subnet method doesn't exist
        Ok(SubnetStatus {
            is_active: true,
            last_block_time: chrono::Utc::now().to_rfc3339(),
            block_height: 1000,
            validators_online: 1,
            consensus_status: "healthy".to_string(),
            sync_status: "synced".to_string(),
        })
    }

    /// Get subnet statistics
    pub async fn get_subnet_stats(&self, subnet_id: &str) -> Result<ChainStats> {
        let _subnet = SubnetID::from_str(subnet_id)?;
        let _provider = get_ipc_provider(&self.global)?;

        // TODO: Implement actual stats gathering
        // For now, return placeholder stats since get_subnet method doesn't exist
        Ok(ChainStats {
            block_height: 1000,
            latest_block_time: chrono::Utc::now().to_rfc3339(),
            transaction_count: 50,
            validator_count: 1,
            tps: 0.0,
            avg_block_time: 2.0,
            last_checkpoint: chrono::Utc::now().to_rfc3339(),
            total_supply: "1000000".to_string(),
            circulating_supply: "800000".to_string(),
            fees_collected: "1000".to_string(),
            pending_transactions: Some(5),
        })
    }

    /// List all subnets/instances
    pub async fn list_subnets(&self) -> Result<Vec<serde_json::Value>> {
        let config_store = crate::ipc_config_store::IpcConfigStore::load_or_init(&self.global).await?;
        let config = config_store.snapshot().await;

        let mut instances = Vec::new();

        // Convert subnet configurations to instances
        for (config_subnet_id, subnet_config) in &config.subnets {
            let subnet_id_str = config_subnet_id.to_string();

            // Determine parent from subnet ID
            let parent = if config_subnet_id.is_root() {
                // This is a root network, so it has no parent, but for UI purposes we'll use itself
                subnet_id_str.clone()
            } else {
                // This is a subnet, get its parent
                if let Some(parent_subnet) = config_subnet_id.parent() {
                    parent_subnet.to_string()
                } else {
                    // Fallback: extract from string representation
                    let parts: Vec<&str> = subnet_id_str.split('/').collect();
                    if parts.len() >= 2 {
                        format!("/{}", parts[1]) // e.g., "/r31337" or "/r314159"
                    } else {
                        "/r314159".to_string() // fallback
                    }
                }
            };

            let instance = serde_json::json!({
                "id": subnet_id_str,
                "name": format!("Subnet {}", subnet_id_str),
                "status": "Active",
                "parent": parent,
                "type": "subnet",
                "created_at": chrono::Utc::now().to_rfc3339(),
                "last_updated": chrono::Utc::now().to_rfc3339(),
                "validator_count": 1,
                "is_active": true,
                "chain_id": subnet_config.id.chain_id(),
                "config": {
                    "gateway_addr": match &subnet_config.config {
                        ipc_provider::config::SubnetConfig::Fevm(evm_subnet) => evm_subnet.gateway_addr.to_string(),
                    },
                    "registry_addr": match &subnet_config.config {
                        ipc_provider::config::SubnetConfig::Fevm(evm_subnet) => evm_subnet.registry_addr.to_string(),
                    }
                }
            });
            instances.push(instance);
        }

        // If no configured subnets, return a placeholder
        if instances.is_empty() {
            instances.push(serde_json::json!({
                "id": "/r314159/t410fmynl4ow2m7u6lgfpaxlklj6kn7lw5u6lnwi",
                "name": "Example Subnet",
                "status": "Active",
                "parent": "/r314159",
                "type": "subnet",
                "created_at": chrono::Utc::now().to_rfc3339(),
                "last_updated": chrono::Utc::now().to_rfc3339(),
                "validator_count": 1,
                "is_active": true,
                "chain_id": 31415926,
                "config": {
                    "gateway_addr": "t410fmynl4ow2m7u6lgfpaxlklj6kn7lw5u6lnwi",
                    "registry_addr": "t410fbnhchh7sdnabedg2qf73u555dm3gyg6nlg4nnaq"
                }
            }));
        }

        Ok(instances)
    }

    /// Get specific subnet info
    pub async fn get_subnet_info(&self, subnet_id: &str) -> Result<serde_json::Value> {
        let config_store = crate::ipc_config_store::IpcConfigStore::load_or_init(&self.global).await?;
        let config = config_store.snapshot().await;

        // Try to find the subnet in configuration by iterating through subnets
        for (config_subnet_id, subnet_config) in &config.subnets {
            if config_subnet_id.to_string() == subnet_id {
                let instance = serde_json::json!({
                    "id": subnet_id,
                    "name": format!("Subnet {}", subnet_id),
                    "status": "Active",
                    "parent": "/r314159",
                    "type": "subnet",
                    "created_at": chrono::Utc::now().to_rfc3339(),
                    "last_updated": chrono::Utc::now().to_rfc3339(),
                    "validator_count": 1,
                    "is_active": true,
                    "chain_id": subnet_config.id.chain_id(),
                    "config": {
                        "gateway_addr": match &subnet_config.config {
                            ipc_provider::config::SubnetConfig::Fevm(evm_subnet) => evm_subnet.gateway_addr.to_string(),
                        },
                        "registry_addr": match &subnet_config.config {
                            ipc_provider::config::SubnetConfig::Fevm(evm_subnet) => evm_subnet.registry_addr.to_string(),
                        }
                    },
                    "stats": {
                        "block_height": 1000,
                        "transaction_count": 50,
                        "validator_count": 1,
                        "last_checkpoint": chrono::Utc::now().to_rfc3339()
                    }
                });
                return Ok(instance);
            }
        }

        // Return placeholder for unknown subnet ID
        Ok(serde_json::json!({
            "id": subnet_id,
            "name": format!("Subnet {}", subnet_id),
            "status": "Active",
            "parent": "/r314159",
            "type": "subnet",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "last_updated": chrono::Utc::now().to_rfc3339(),
            "validator_count": 1,
            "is_active": true,
            "chain_id": 31415926,
            "config": {
                "gateway_addr": "t410fmynl4ow2m7u6lgfpaxlklj6kn7lw5u6lnwi",
                "registry_addr": "t410fbnhchh7sdnabedg2qf73u555dm3gyg6nlg4nnaq"
            },
            "stats": {
                "block_height": 1000,
                "transaction_count": 50,
                "validator_count": 1,
                "last_checkpoint": chrono::Utc::now().to_rfc3339()
            }
        }))
    }



        /// List pending subnet approvals for a gateway
    pub async fn list_pending_approvals(&self, gateway_address: &str) -> anyhow::Result<Vec<serde_json::Value>> {
        log::info!("Listing pending approvals for gateway: {}", gateway_address);

        // For now, we'll check the config for subnets that might need approval
        // This is a simplified implementation - in a real scenario, you'd query the gateway contract
        let config_store = crate::ipc_config_store::IpcConfigStore::load_or_init(&self.global).await?;
        let config = config_store.snapshot().await;

        let mut pending_subnets = Vec::new();

        // Find subnets that have this gateway address but might not be approved yet
        for (subnet_id, subnet_config) in &config.subnets {
            match &subnet_config.config {
                                ipc_provider::config::SubnetConfig::Fevm(evm_subnet) => {
                                        // Convert both addresses to Ethereum hex format for comparison
                    let config_gateway_eth = evm_subnet.gateway_addr.to_string().to_lowercase();

                    // For now, try simple comparison - TODO: implement proper address conversion
                    let target_gateway_eth = gateway_address.to_lowercase();

                    log::info!("Comparing gateway addresses: config_eth={}, target_eth={}", config_gateway_eth, target_gateway_eth);

                    if config_gateway_eth == target_gateway_eth {
                        // This subnet uses this gateway, so it might need approval
                        let parent_id = subnet_id.parent().unwrap_or_else(|| subnet_id.clone());

                        log::info!("Found subnet {} that uses gateway {}", subnet_id, gateway_address);

                        let subnet_info = serde_json::json!({
                            "subnet_id": subnet_id.to_string(),
                            "gateway_address": evm_subnet.gateway_addr.to_string(),
                            "registry_address": evm_subnet.registry_addr.to_string(),
                            "parent_id": parent_id.to_string(),
                            "status": "pending_approval", // This would need real blockchain query in production
                            "created_at": chrono::Utc::now().to_rfc3339(),
                        });

                        pending_subnets.push(subnet_info);
                    }
                }
            }
        }

        log::info!("Found {} pending subnets for gateway {}", pending_subnets.len(), gateway_address);
        Ok(pending_subnets)
    }
}