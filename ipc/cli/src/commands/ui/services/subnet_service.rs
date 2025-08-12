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
use num_traits::ToPrimitive;

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

    /// Helper method to get the permission mode for a subnet
    async fn get_permission_mode(&self, subnet_id: &str) -> Result<String> {
        log::info!("=== GETTING PERMISSION MODE FOR SUBNET: {} ===", subnet_id);

        log::info!("Step 1: Getting IPC provider...");
        let provider = crate::get_ipc_provider(&self.global).map_err(|e| {
            log::error!("Failed to get IPC provider for permission mode: {}", e);
            e
        })?;
        log::info!("Step 1: ✓ Successfully got IPC provider");

        log::info!("Step 2: Parsing subnet ID '{}'...", subnet_id);
        let subnet = SubnetID::from_str(subnet_id).map_err(|e| {
            log::error!("Failed to parse subnet ID '{}': {}", subnet_id, e);
            e
        })?;
        log::info!("Step 2: ✓ Successfully parsed subnet ID: {:?}", subnet);

        // Check if this is a root subnet (no parent)
        if subnet.is_root() {
            log::error!("Step 3: ✗ Subnet {} is a root subnet - cannot get permission mode for root subnets", subnet_id);
            return Err(anyhow::anyhow!("Cannot get permission mode for root subnet {}", subnet_id));
        }
        log::info!("Step 3: ✓ Subnet is not a root subnet");

        // Log parent information
        if let Some(parent) = subnet.parent() {
            log::info!("Step 4: Subnet parent is: {}", parent);
        } else {
            log::error!("Step 4: ✗ Subnet has no parent but is not root - this is unexpected");
            return Err(anyhow::anyhow!("Subnet {} has no parent but is not root", subnet_id));
        }

        // Get genesis info which contains the permission mode
        log::info!("Step 5: Getting genesis info from provider...");
        match provider.get_genesis_info(&subnet).await {
            Ok(genesis_info) => {
                log::info!("Step 5: ✓ Successfully got genesis info");
                log::info!("Genesis info: permission_mode={:?}", genesis_info.permission_mode);

                let permission_mode = match genesis_info.permission_mode {
                    ipc_api::subnet::PermissionMode::Collateral => "collateral",
                    ipc_api::subnet::PermissionMode::Federated => "federated",
                    ipc_api::subnet::PermissionMode::Static => "static",
                };
                log::info!("Step 6: ✓ Mapped permission mode to string: '{}'", permission_mode);
                log::info!("=== PERMISSION MODE RETRIEVAL SUCCESSFUL: {} ===", permission_mode);
                Ok(permission_mode.to_string())
            }
            Err(e) => {
                log::error!("Step 5: ✗ Failed to get genesis info for subnet {}: {}", subnet_id, e);
                log::error!("=== PERMISSION MODE RETRIEVAL FAILED ===");
                Err(anyhow::anyhow!("Failed to get genesis info for subnet {}: {}", subnet_id, e))
            }
        }
    }

    /// List all subnets/instances
    pub async fn list_subnets(&self) -> Result<Vec<serde_json::Value>> {
        let config_store = crate::ipc_config_store::IpcConfigStore::load_or_init(&self.global).await?;
        let config = config_store.snapshot().await;

        log::info!("Loading subnet configurations from config store");
        log::info!("Found {} subnets in configuration", config.subnets.len());

        // Log all found subnets with their details
        for (subnet_id, subnet_config) in &config.subnets {
            let subnet_id_str = subnet_id.to_string();
            log::info!("Raw subnet found: ID='{}', is_root={}, gateway={:?}, registry={:?}",
                subnet_id_str,
                subnet_id.is_root(),
                match &subnet_config.config {
                    ipc_provider::config::SubnetConfig::Fevm(evm_subnet) => evm_subnet.gateway_addr.to_string(),
                },
                match &subnet_config.config {
                    ipc_provider::config::SubnetConfig::Fevm(evm_subnet) => evm_subnet.registry_addr.to_string(),
                }
            );
        }

        let mut instances = Vec::new();
        let mut root_networks_filtered = 0;

        // Convert subnet configurations to instances, filtering out root networks
        for (config_subnet_id, subnet_config) in &config.subnets {
            let subnet_id_str = config_subnet_id.to_string();

            // Skip root networks (they are not subnets to be displayed in UI)
            if config_subnet_id.is_root() {
                log::info!("Skipping root network (not a subnet): {}", subnet_id_str);
                root_networks_filtered += 1;
                continue;
            }

            log::info!("Processing actual subnet: {}", subnet_id_str);

            // Get validators from the blockchain
            let validators = self.get_validators_for_subnet(&subnet_id_str).await.unwrap_or_else(|e| {
                log::warn!("Failed to fetch validators for subnet {}: {}", subnet_id_str, e);
                Vec::new()
            });

            // Get the actual permission mode from the subnet contract
            let permission_mode = match self.get_permission_mode(&subnet_id_str).await {
                Ok(mode) => {
                    log::info!("Successfully retrieved permission mode for subnet {}: {}", subnet_id_str, mode);
                    mode
                }
                Err(e) => {
                    log::error!("Failed to get permission mode for subnet {}: {}", subnet_id_str, e);
                    "unknown".to_string()
                }
            };

            let instance = serde_json::json!({
                "id": subnet_id_str,
                "name": format!("Subnet {}", subnet_id_str.split('/').last().unwrap_or(&subnet_id_str)),
                "status": "active", // You might want to determine this dynamically
                "validators": validators,
                "config": {
                    "permissionMode": permission_mode,
                    "gateway_addr": match &subnet_config.config {
                        ipc_provider::config::SubnetConfig::Fevm(evm_subnet) => evm_subnet.gateway_addr.to_string(),
                    },
                    "registry_addr": match &subnet_config.config {
                        ipc_provider::config::SubnetConfig::Fevm(evm_subnet) => evm_subnet.registry_addr.to_string(),
                    }
                },
                "parent": config_subnet_id.parent().map(|p| p.to_string()),
                "chain_head": serde_json::Value::Null,
                "nonce": 0,
                "circulating_supply": "0"
            });

            instances.push(instance);
        }

        log::info!("Returning {} subnet instances (after filtering out {} root networks)", instances.len(), root_networks_filtered);
        Ok(instances)
    }

    /// Get specific subnet info
    pub async fn get_subnet_info(&self, subnet_id: &str) -> Result<serde_json::Value> {
        let config_store = crate::ipc_config_store::IpcConfigStore::load_or_init(&self.global).await?;
        let config = config_store.snapshot().await;

        log::info!("Looking for subnet with ID: {}", subnet_id);
        log::info!("Available subnets in config: {:?}", config.subnets.keys().collect::<Vec<_>>());

        // Try to find the subnet in configuration by iterating through subnets
        for (config_subnet_id, subnet_config) in &config.subnets {
            if config_subnet_id.to_string() == subnet_id {
                log::info!("Found matching subnet configuration for: {}", subnet_id);

                // Determine parent from subnet ID
                let parent = if config_subnet_id.is_root() {
                    // This is a root network, so it has no parent, but for UI purposes we'll use itself
                    subnet_id.to_string()
                } else {
                    // This is a subnet, get its parent
                    if let Some(parent_subnet) = config_subnet_id.parent() {
                        parent_subnet.to_string()
                    } else {
                        // Fallback: extract from string representation
                        let parts: Vec<&str> = subnet_id.split('/').collect();
                        if parts.len() >= 2 {
                            format!("/{}", parts[1]) // e.g., "/r31337" or "/r314159"
                        } else {
                            "/r314159".to_string() // fallback
                        }
                    }
                };

                // Get validators from the blockchain
                let validators = self.get_validators_for_subnet(subnet_id).await.unwrap_or_else(|e| {
                    log::warn!("Failed to fetch validators for subnet {}: {}", subnet_id, e);
                    Vec::new()
                });

                // Get the actual permission mode from the subnet contract
                let permission_mode = match self.get_permission_mode(subnet_id).await {
                    Ok(mode) => {
                        log::info!("Successfully retrieved permission mode for subnet {}: {}", subnet_id, mode);
                        mode
                    }
                    Err(e) => {
                        log::error!("Failed to get permission mode for subnet {}: {}", subnet_id, e);
                        "unknown".to_string()
                    }
                };

                let instance = serde_json::json!({
                    "id": subnet_id,
                    "name": format!("Subnet {}", subnet_id),
                    "status": "Active",
                    "parent": parent,
                    "type": "subnet",
                    "created_at": chrono::Utc::now().to_rfc3339(),
                    "last_updated": chrono::Utc::now().to_rfc3339(),
                    "validator_count": validators.len(),
                    "is_active": true,
                    "chain_id": subnet_config.id.chain_id(),
                    "validators": validators,
                    "config": {
                        "permissionMode": permission_mode,
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
                        "validator_count": validators.len(),
                        "last_checkpoint": chrono::Utc::now().to_rfc3339()
                    }
                });
                return Ok(instance);
            }
        }

        // If subnet not found in config, return an error instead of placeholder
        anyhow::bail!("Subnet with ID '{}' not found in configuration", subnet_id)
    }

        /// Helper method to get validators for a subnet
    async fn get_validators_for_subnet(&self, subnet_id: &str) -> Result<Vec<serde_json::Value>> {
        log::info!("=== FETCHING VALIDATORS FOR SUBNET: {} ===", subnet_id);

        log::info!("Step 1: Getting IPC provider...");
        let provider = crate::get_ipc_provider(&self.global).map_err(|e| {
            log::error!("Failed to get IPC provider: {}", e);
            e
        })?;
        log::info!("Step 1: ✓ Successfully got IPC provider");

        log::info!("Step 2: Parsing subnet ID '{}'...", subnet_id);
        let subnet = SubnetID::from_str(subnet_id).map_err(|e| {
            log::error!("Failed to parse subnet ID '{}': {}", subnet_id, e);
            e
        })?;
        log::info!("Step 2: ✓ Successfully parsed subnet ID: {:?}", subnet);

        // Check if this is a root subnet (no parent)
        if subnet.is_root() {
            log::info!("Step 3: Subnet {} is a root subnet, no validators to fetch from parent", subnet_id);
            log::info!("=== VALIDATOR FETCHING SKIPPED FOR ROOT SUBNET ===");
            return Ok(vec![]);
        }
        log::info!("Step 3: ✓ Subnet is not a root subnet");

        log::info!("Step 4: Getting parent subnet...");
        let parent = subnet.parent();
        match &parent {
            Some(parent_subnet) => {
                log::info!("Step 4: ✓ Subnet {} has parent: {}", subnet_id, parent_subnet);

                // Log available connections to help diagnose
                let connections = provider.list_connections();
                let available_subnets: Vec<String> = connections.keys().map(|k| k.to_string()).collect();
                log::info!("Available configured subnets in provider: {:?}", available_subnets);

                if !connections.contains_key(parent_subnet) {
                    log::error!("Step 4: ✗ Parent subnet {} is NOT configured in the IPC provider!", parent_subnet);
                    log::error!("You need to configure the parent network ({}) in your IPC config", parent_subnet);
                } else {
                    log::info!("Step 4: ✓ Parent subnet {} is configured in the IPC provider", parent_subnet);
                }
            }
            None => {
                log::error!("Step 4: ✗ Subnet {} has no parent but is not root - this is unexpected", subnet_id);
                return Ok(vec![]);
            }
        }

        log::info!("Step 5: Attempting to fetch validators for subnet: {}", subnet_id);

                match provider.list_validators(&subnet).await {
            Ok(validators) => {
                log::info!("Step 5: ✓ Successfully fetched {} validators for subnet {}", validators.len(), subnet_id);

                let mut validator_list = Vec::new();
                for (i, (address, validator_info)) in validators.iter().enumerate() {
                    log::info!("Step 6.{}: Processing validator {}: {}", i+1, i+1, address);

                    // Convert TokenAmount to string for stake/power
                    let stake = validator_info.staking.current_power().to_string();
                    let power = validator_info.staking.next_power().atto().to_u64().unwrap_or(0);

                    // Determine status based on validator state
                    let status = if validator_info.is_active {
                        "active"
                    } else if validator_info.is_waiting {
                        "waiting"
                    } else {
                        "inactive"
                    };

                    let validator_json = serde_json::json!({
                        "address": address.to_string(),
                        "stake": stake,
                        "power": power,
                        "status": status,
                        "is_active": validator_info.is_active,
                        "is_waiting": validator_info.is_waiting,
                        "current_power": validator_info.staking.current_power().to_string(),
                        "next_power": validator_info.staking.next_power().to_string()
                    });

                    log::info!("Step 6.{}: ✓ Validator {} - status: {}, stake: {}, power: {}",
                        i+1, address, status, stake, power);
                    validator_list.push(validator_json);
                }

                log::info!("=== VALIDATOR FETCHING SUCCESSFUL: {} validators ===", validator_list.len());
                Ok(validator_list)
            }
            Err(e) => {
                // Enhanced error logging to help diagnose the issue
                log::error!("Step 5: ✗ Failed to fetch validators for subnet {}: {}", subnet_id, e);
                log::error!("Error details: {:?}", e);

                if let Some(parent_subnet) = &parent {
                    log::error!("This error occurred while trying to query parent network: {}", parent_subnet);
                    log::error!("Make sure the parent network ({}) is properly configured and accessible", parent_subnet);
                }

                log::error!("=== VALIDATOR FETCHING FAILED ===");
                // Always return empty list - no mock data
                Ok(vec![])
            }
        }
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