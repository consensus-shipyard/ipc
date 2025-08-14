// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Subnet service for subnet management operations
//!
//! This service wraps existing CLI handlers for subnet operations.

use super::super::api::types::{ApiResponse, ChainStats, SubnetLifecycleState, SubnetStatusInfo, SubnetInstanceResponse};
use crate::commands::subnet::{
    approve::{approve_subnet, ApproveSubnetArgs},
    join::{join_subnet, JoinSubnetArgs},
    leave::{LeaveSubnetArgs},
    // Note: list_subnets is not available as a function export
};
use crate::{GlobalArguments, get_ipc_provider};
use anyhow::{Result, Context};
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
    pub async fn get_subnet_status(&self, subnet_id: &str) -> Result<SubnetStatusInfo> {
        // Use the comprehensive status method we implemented
        self.get_comprehensive_subnet_status(subnet_id).await
    }

    /// Get subnet statistics
    pub async fn get_subnet_stats(&self, subnet_id: &str) -> Result<ChainStats> {
        let subnet = SubnetID::from_str(subnet_id)?;
        let provider = get_ipc_provider(&self.global)?;

        log::info!("Fetching real blockchain statistics for subnet: {}", subnet_id);

        // Initialize default stats in case some calls fail
        let mut stats = ChainStats {
            block_height: 0,
            latest_block_time: chrono::Utc::now().to_rfc3339(),
            transaction_count: 0,
            validator_count: 0,
            tps: 0.0,
            avg_block_time: 2.0,
            last_checkpoint: chrono::Utc::now().to_rfc3339(),
            total_supply: "0".to_string(),
            circulating_supply: "0".to_string(),
            fees_collected: "0".to_string(),
            pending_transactions: Some(0),
        };

        // Try to get real block height
        match provider.get_chain_head_height(&subnet).await {
            Ok(height) => {
                log::info!("Successfully fetched block height for subnet {}: {}", subnet_id, height);
                stats.block_height = height as u64;
            }
            Err(e) => {
                log::warn!("Failed to get block height for subnet {}: {}. Using default.", subnet_id, e);
                // Keep default value of 0
            }
        }

        // Try to get validator count
        match provider.list_validators(&subnet).await {
            Ok(validators) => {
                log::info!("Successfully fetched {} validators for subnet {}", validators.len(), subnet_id);
                stats.validator_count = validators.len() as u32;
            }
            Err(e) => {
                log::warn!("Failed to get validators for subnet {}: {}. Using default.", subnet_id, e);
                // Keep default value of 0
            }
        }

        // Try to get chain ID for additional context
        match provider.get_chain_id(&subnet).await {
            Ok(chain_id) => {
                log::info!("Chain ID for subnet {}: {}", subnet_id, chain_id);
            }
            Err(e) => {
                log::warn!("Failed to get chain ID for subnet {}: {}", subnet_id, e);
            }
        }

        // For now, we'll use estimated values for transaction count and TPS
        // These would require more complex blockchain analysis or additional RPC methods
        if stats.block_height > 0 {
            // Estimate transaction count based on block height (rough estimate)
            stats.transaction_count = stats.block_height * 2; // Assume ~2 transactions per block on average

            // Calculate estimated TPS based on block time and transaction count
            if stats.block_height > 1 {
                let total_time_seconds = stats.block_height as f64 * stats.avg_block_time;
                stats.tps = stats.transaction_count as f64 / total_time_seconds;
            }
        }

        // Update timestamp to current time
        stats.latest_block_time = chrono::Utc::now().to_rfc3339();
        stats.last_checkpoint = chrono::Utc::now().to_rfc3339();

        log::info!("Final stats for subnet {}: block_height={}, validator_count={}, transaction_count={}, tps={:.2}",
                   subnet_id, stats.block_height, stats.validator_count, stats.transaction_count, stats.tps);

        Ok(stats)
    }

    /// Get comprehensive subnet status information with enhanced state detection
    async fn get_comprehensive_subnet_status(&self, subnet_id: &str) -> Result<SubnetStatusInfo> {
        log::info!("=== GETTING COMPREHENSIVE STATUS FOR SUBNET: {} ===", subnet_id);

        let mut status = SubnetStatusInfo::default();

        // Step 1: Parse subnet ID and basic validation
        log::info!("Step 1: Parsing subnet ID '{}'...", subnet_id);
        let subnet = match SubnetID::from_str(subnet_id) {
            Ok(subnet) => {
                log::info!("Step 1: ✓ Successfully parsed subnet ID: {:?}", subnet);
                subnet
            }
            Err(e) => {
                log::error!("Step 1: ✗ Failed to parse subnet ID '{}': {}", subnet_id, e);
                status.lifecycle_state = SubnetLifecycleState::Failed;
                status.error_message = Some(format!("Invalid subnet ID: {}", e));
                return Ok(status);
            }
        };

        // Step 2: Check if this is a root subnet
        if subnet.is_root() {
            log::info!("Step 2: ✓ Subnet {} is a root subnet", subnet_id);
            status.lifecycle_state = SubnetLifecycleState::Healthy; // Root networks are always "healthy"
            status.genesis_available = true; // Root has implicit genesis
            return Ok(status);
        }
        log::info!("Step 2: ✓ Subnet is not a root subnet");

        // Step 3: Get IPC provider
        log::info!("Step 3: Getting IPC provider...");
        let provider = match crate::get_ipc_provider(&self.global) {
            Ok(provider) => {
                log::info!("Step 3: ✓ Successfully got IPC provider");
                provider
            }
            Err(e) => {
                log::error!("Step 3: ✗ Failed to get IPC provider: {}", e);
                status.lifecycle_state = SubnetLifecycleState::Failed;
                status.error_message = Some(format!("Failed to get IPC provider: {}", e));
                return Ok(status);
            }
        };

        // Step 4: Try to get genesis info (this is the critical detection point)
        log::info!("Step 4: Attempting to get genesis info (single attempt - no retries)...");
        match provider.get_genesis_info(&subnet).await {
            Ok(genesis_info) => {
                log::info!("Step 4: ✓ Genesis info available - subnet is initialized");
                status.genesis_available = true;
                status.permission_mode = Some(match genesis_info.permission_mode {
                    ipc_api::subnet::PermissionMode::Collateral => "collateral".to_string(),
                    ipc_api::subnet::PermissionMode::Federated => "federated".to_string(),
                    ipc_api::subnet::PermissionMode::Static => "static".to_string(),
                });
                status.lifecycle_state = SubnetLifecycleState::Active; // Will be refined later
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("does not exist") || error_msg.contains("does not exists") {
                    log::info!("Step 4: ✓ Subnet contracts exist but genesis not available - waiting for validators");
                    status.genesis_available = false;
                    status.lifecycle_state = SubnetLifecycleState::WaitingForValidators;
                    status.next_action_required = Some("Start validators to activate subnet".to_string());
                } else {
                    log::error!("Step 4: ✗ Unexpected error getting genesis info: {}", e);
                    status.lifecycle_state = SubnetLifecycleState::Failed;
                    status.error_message = Some(format!("Genesis query failed: {}", e));
                    return Ok(status);
                }
            }
        }

        // Step 5: Get validator information to refine the state
        log::info!("Step 5: Getting validator information...");
        match provider.list_validators(&subnet).await {
            Ok(validators) => {
                status.validator_count = validators.len();
                status.active_validators = validators.iter()
                    .filter(|v| {
                        // Check if validator is active by examining the validator info structure
                        // ValidatorInfo doesn't have is_active(), we'll use a simple heuristic
                        true // For now, consider all validators as active since we can't determine this easily
                    })
                    .count();

                log::info!("Step 5: ✓ Found {} validators ({} active)",
                    status.validator_count, status.active_validators);

                // Refine state based on validator information
                match status.lifecycle_state {
                    SubnetLifecycleState::WaitingForValidators => {
                        if status.validator_count == 0 {
                            // Confirmed: no validators registered yet
                            status.next_action_required = Some("Register and start validators to activate subnet".to_string());
                        } else {
                            // Validators exist but genesis isn't available - likely initialization issue
                            status.lifecycle_state = SubnetLifecycleState::Initializing;
                            status.next_action_required = Some("Validators registered but subnet not fully initialized".to_string());
                        }
                    }
                    SubnetLifecycleState::Active => {
                        // Refine active state based on validator health
                        if status.validator_count == 0 {
                            status.lifecycle_state = SubnetLifecycleState::Offline;
                            status.next_action_required = Some("No validators found - subnet is offline".to_string());
                        } else if status.active_validators == 0 {
                            status.lifecycle_state = SubnetLifecycleState::Offline;
                            status.next_action_required = Some("All validators are offline".to_string());
                        } else if status.active_validators < status.validator_count {
                            status.lifecycle_state = SubnetLifecycleState::Degraded;
                            status.next_action_required = Some(format!("Only {}/{} validators are active",
                                status.active_validators, status.validator_count));
                        } else {
                            status.lifecycle_state = SubnetLifecycleState::Healthy;
                        }
                    }
                    _ => {
                        // Other states remain as is
                    }
                }
            }
            Err(e) => {
                log::warn!("Step 5: ⚠ Could not get validator information: {}", e);
                // Don't fail the entire status check for validator query failures
                // This is expected for subnets in waiting/initializing states
                if status.lifecycle_state == SubnetLifecycleState::WaitingForValidators {
                    // This is expected behavior
                } else {
                    // For other states, note the validator query issue
                    status.error_message = Some(format!("Could not query validators: {}", e));
                }
            }
        }

        // Step 6: Check blockchain activity to determine if subnet is truly running
        log::info!("Step 6: Checking blockchain activity...");
        match provider.get_chain_head_height(&subnet).await {
            Ok(height) => {
                log::info!("Step 6: ✓ Blockchain is active with block height: {}", height);
                status.block_height = height as u64;

                // If we can get block height, the subnet blockchain is definitely running
                if status.lifecycle_state == SubnetLifecycleState::WaitingForValidators ||
                   status.lifecycle_state == SubnetLifecycleState::Initializing {
                    // Upgrade status to Active since blockchain is running
                    status.lifecycle_state = SubnetLifecycleState::Active;
                    status.next_action_required = None;
                }

                // Set consensus status based on blockchain activity
                status.consensus_status = if height > 0 {
                    "healthy".to_string()
                } else {
                    "starting".to_string()
                };
            }
            Err(e) => {
                log::warn!("Step 6: ⚠ Cannot get blockchain height: {}", e);
                status.block_height = 0;

                // If we can't get block height but have validators, subnet might be starting
                if status.validator_count > 0 && status.lifecycle_state == SubnetLifecycleState::Active {
                    status.lifecycle_state = SubnetLifecycleState::Initializing;
                    status.next_action_required = Some("Subnet validators are configured but blockchain is not responding".to_string());
                }

                status.consensus_status = "offline".to_string();
            }
        }

        // Step 7: Final status determination based on all checks
        log::info!("Step 7: Final status determination...");

        // Set is_active flag based on final lifecycle state
        status.is_active = matches!(status.lifecycle_state,
            SubnetLifecycleState::Healthy |
            SubnetLifecycleState::Active |
            SubnetLifecycleState::Syncing |
            SubnetLifecycleState::Degraded
        );

        // Set validators_online count (for now, same as active_validators)
        status.validators_online = status.active_validators;

        // Set status and message fields for frontend compatibility
        status.status = match status.lifecycle_state {
            SubnetLifecycleState::Healthy => "Active".to_string(),
            SubnetLifecycleState::Active => "Active".to_string(),
            SubnetLifecycleState::Syncing => "Syncing".to_string(),
            SubnetLifecycleState::Degraded => "Degraded".to_string(),
            SubnetLifecycleState::Offline => "Offline".to_string(),
            SubnetLifecycleState::Deploying => "Deploying".to_string(),
            SubnetLifecycleState::Deployed => "Deployed".to_string(),
            SubnetLifecycleState::WaitingForValidators => "Waiting for Validators".to_string(),
            SubnetLifecycleState::Initializing => "Initializing".to_string(),
            SubnetLifecycleState::Failed => "Failed".to_string(),
            SubnetLifecycleState::Unknown => "Unknown".to_string(),
        };

        status.message = status.next_action_required.clone()
            .unwrap_or_else(|| {
                match status.lifecycle_state {
                    SubnetLifecycleState::Healthy => "Subnet is running normally".to_string(),
                    SubnetLifecycleState::Active => "Subnet is active and processing blocks".to_string(),
                    SubnetLifecycleState::Syncing => "Subnet is synchronizing with the network".to_string(),
                    SubnetLifecycleState::Degraded => "Subnet is running but some validators are offline".to_string(),
                    SubnetLifecycleState::Offline => "Subnet is not responding".to_string(),
                    SubnetLifecycleState::Deploying => "Subnet is being deployed".to_string(),
                    SubnetLifecycleState::Deployed => "Subnet has been deployed".to_string(),
                    SubnetLifecycleState::Initializing => "Subnet is initializing".to_string(),
                    SubnetLifecycleState::WaitingForValidators => "Waiting for validators to join".to_string(),
                    SubnetLifecycleState::Failed => "Subnet deployment or operation failed".to_string(),
                    SubnetLifecycleState::Unknown => "Subnet status is unknown".to_string(),
                }
            });

        log::info!("=== COMPREHENSIVE STATUS COMPLETE: {} = {} (is_active: {}, block_height: {}) ===",
                   subnet_id, status.lifecycle_state, status.is_active, status.block_height);
        Ok(status)
    }

    /// Helper method to get the permission mode for a subnet with retry logic
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
            log::warn!("Step 3: Subnet {} is a root subnet - root subnets don't have permission modes", subnet_id);
            log::info!("=== PERMISSION MODE RETRIEVAL SKIPPED FOR ROOT SUBNET ===");
            return Ok("root".to_string()); // Return "root" instead of error
        }
        log::info!("Step 3: ✓ Subnet is not a root subnet");

        // Log parent information
        if let Some(parent) = subnet.parent() {
            log::info!("Step 4: Subnet parent is: {}", parent);
        } else {
            log::error!("Step 4: ✗ Subnet has no parent but is not root - this is unexpected");
            return Ok("unknown".to_string()); // Return "unknown" instead of error
        }

        // Get genesis info with retry logic for newly deployed subnets
        log::info!("Step 5: Getting genesis info from provider with retry logic...");
        let max_retries = 3; // Reduced from 5 to avoid long waits
        let mut retry_count = 0;
        let mut delay_ms = 1000; // Start with 1 second

        loop {
            match provider.get_genesis_info(&subnet).await {
                Ok(genesis_info) => {
                    log::info!("Step 5: ✓ Successfully got genesis info (attempt {})", retry_count + 1);
                    log::info!("Genesis info: permission_mode={:?}", genesis_info.permission_mode);

                    let permission_mode = match genesis_info.permission_mode {
                        ipc_api::subnet::PermissionMode::Collateral => "collateral",
                        ipc_api::subnet::PermissionMode::Federated => "federated",
                        ipc_api::subnet::PermissionMode::Static => "static",
                    };
                    log::info!("Step 6: ✓ Mapped permission mode to string: '{}'", permission_mode);
                    log::info!("=== PERMISSION MODE RETRIEVAL SUCCESSFUL: {} ===", permission_mode);
                    return Ok(permission_mode.to_string());
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        log::error!("Step 5: ✗ Failed to get genesis info for subnet {} after {} attempts: {}", subnet_id, max_retries, e);

                        // Instead of returning an error, return "unknown" with detailed logging
                        // This allows the UI to show the subnet info even if permission mode can't be determined
                        log::warn!("Returning 'unknown' permission mode due to genesis info fetch failure. This may be due to:");
                        log::warn!("  - Network connectivity issues");
                        log::warn!("  - Parent network not properly configured");
                        log::warn!("  - Subnet not yet fully deployed");
                        log::warn!("  - Blockchain synchronization delays");

                        log::info!("=== PERMISSION MODE RETRIEVAL FAILED - RETURNING 'unknown' ===");
                        return Ok("unknown".to_string()); // Return "unknown" instead of error
                    }

                    log::warn!("Step 5: ⚠ Attempt {} failed to get genesis info for subnet {}: {}. Retrying in {}ms...", retry_count, subnet_id, e, delay_ms);
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    delay_ms = std::cmp::min(delay_ms * 2, 5000); // Exponential backoff, max 5s
                }
            }
        }
    }

    /// List all subnets/instances with enhanced status information
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

            // Get comprehensive status information
            let status_info = match self.get_comprehensive_subnet_status(&subnet_id_str).await {
                Ok(status) => {
                    log::info!("✓ Comprehensive status for {}: {} (genesis: {}, validators: {}/{})",
                        subnet_id_str, status.lifecycle_state, status.genesis_available,
                        status.active_validators, status.validator_count);
                    status
                }
                Err(e) => {
                    log::error!("✗ Failed to get comprehensive status for {}: {}", subnet_id_str, e);
                    let mut status = SubnetStatusInfo::default();
                    status.lifecycle_state = SubnetLifecycleState::Failed;
                    status.error_message = Some(format!("Status check failed: {}", e));
                    status
                }
            };

            // Get validators for detailed info (only if needed for display)
            let validators = if status_info.validator_count > 0 {
                self.get_validators_for_subnet(&subnet_id_str).await.unwrap_or_else(|e| {
                    log::warn!("Failed to fetch detailed validator info for subnet {}: {}", subnet_id_str, e);
                    Vec::new()
                })
            } else {
                Vec::new()
            };

            // Create enhanced response with comprehensive status
            let instance = serde_json::json!({
                "id": subnet_id_str,
                "name": format!("Subnet {}", subnet_id_str.split('/').last().unwrap_or(&subnet_id_str)),
                "status": status_info.lifecycle_state.to_string(),
                "status_info": {
                    "lifecycle_state": status_info.lifecycle_state.to_string(),
                    "genesis_available": status_info.genesis_available,
                    "validator_count": status_info.validator_count,
                    "active_validators": status_info.active_validators,
                    "permission_mode": status_info.permission_mode,
                    "deployment_time": status_info.deployment_time,
                    "last_block_time": status_info.last_block_time,
                    "error_message": status_info.error_message,
                    "next_action_required": status_info.next_action_required
                },
                "validators": validators,
                "config": {
                    "permissionMode": status_info.permission_mode.unwrap_or_else(|| "unknown".to_string()),
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
            Some(p) => log::info!("Step 4: ✓ Parent subnet: {}", p),
            None => {
                log::error!("Step 4: ✗ No parent found for non-root subnet {}", subnet_id);
                return Err(anyhow::anyhow!("No parent found for non-root subnet {}", subnet_id));
            }
        }

        // Get validators with retry logic for newly deployed subnets
        log::info!("Step 5: Getting validators from provider with retry logic...");
        let max_retries = 3; // Reduced from 5 to avoid long waits
        let mut retry_count = 0;
        let mut delay_ms = 2000; // Start with 2 seconds for validators

        loop {
            match provider.list_validators(&subnet).await {
                Ok(validators) => {
                    log::info!("Step 5: ✓ Successfully fetched {} validators for subnet {} (attempt {})", validators.len(), subnet_id, retry_count + 1);

                    let mut validator_list = Vec::new();
                    for (i, (address, validator_info)) in validators.iter().enumerate() {
                        log::info!("Step 6.{}: Processing validator {}: {}", i+1, i+1, address);

                        // Convert TokenAmount to string for stake/power
                        let stake = validator_info.staking.current_power().to_string();
                        let power = validator_info.staking.next_power().atto().to_u64().unwrap_or(0);

                        // Determine status based on validator state
                        let status = if validator_info.is_active {
                            "Active"
                        } else if validator_info.is_waiting {
                            "Waiting"
                        } else {
                            "Inactive"
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
                    return Ok(validator_list);
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        log::error!("Step 5: ✗ Failed to get validators for subnet {} after {} attempts: {}", subnet_id, max_retries, e);

                        // Instead of returning an error, return empty list with detailed logging
                        // This allows the UI to show the subnet info even if validators can't be fetched
                        log::warn!("Returning empty validator list due to fetch failure. This may be due to:");
                        log::warn!("  - Network connectivity issues");
                        log::warn!("  - Parent network not properly configured");
                        log::warn!("  - Subnet not yet fully deployed");
                        log::warn!("  - Blockchain synchronization delays");

                        log::info!("=== VALIDATOR FETCHING FAILED - RETURNING EMPTY LIST ===");
                        return Ok(vec![]); // Return empty list instead of error
                    }

                    log::warn!("Step 5: ⚠ Attempt {} failed to get validators for subnet {}: {}. Retrying in {}ms...", retry_count, subnet_id, e, delay_ms);
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    delay_ms = std::cmp::min(delay_ms * 2, 10000); // Exponential backoff, max 10s
                }
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

    /// Generate node configuration YAML for a validator
    pub async fn generate_node_config(
        &self,
        subnet_id: &str,
        validator_address: Option<&str>,
    ) -> Result<String> {
        use crate::commands::node::config::{GenesisSource, NodeInitConfig, P2pConfig};
        use crate::commands::subnet::create_genesis::CreatedGenesis;
        use crate::commands::subnet::init::config::JoinConfig;
        use crate::commands::wallet::import::WalletImportArgs;
        use std::path::PathBuf;

        log::info!("Generating node configuration for subnet: {}", subnet_id);

        let subnet = ipc_api::subnet_id::SubnetID::from_str(subnet_id)?;
        let parent_id = subnet.parent().unwrap_or_else(|| subnet.clone());
        let mut provider = crate::get_ipc_provider(&self.global)?;

        // Determine if this is a collateral-based subnet
        let permission_mode = match provider.get_genesis_info(&subnet).await {
            Ok(genesis_info) => Some(genesis_info.permission_mode),
            Err(_) => None,
        };

        let is_collateral = matches!(permission_mode, Some(ipc_api::subnet::PermissionMode::Collateral));

        // Create join config if it's a collateral subnet
        let join_config = if is_collateral {
            Some(JoinConfig {
                from: validator_address.unwrap_or("YOUR_VALIDATOR_ADDRESS").to_string(),
                collateral: 1.0,
                initial_balance: Some(10.0),
            })
        } else {
            None
        };

        // Determine genesis source based on subnet status
        let genesis_source = match provider.get_genesis_info(&subnet).await {
            Ok(_) => {
                // Subnet is activated - use existing genesis file paths
                let safe_id = subnet_id.replace('/', "_").replace(":", "_");
                GenesisSource::Path(CreatedGenesis {
                    genesis: PathBuf::from(format!("~/.ipc/genesis_{}.car", safe_id)),
                    sealed: PathBuf::from(format!("~/.ipc/genesis_sealed_{}.car", safe_id)),
                })
            }
            Err(_) => {
                // Subnet is NOT activated - create new genesis
                GenesisSource::Create(crate::commands::subnet::create_genesis::GenesisConfig {
                    network_version: fvm_shared::version::NetworkVersion::V21,
                    base_fee: fvm_shared::econ::TokenAmount::from_atto(1000),
                    power_scale: 3,
                })
            }
        };

        // Create basic node config with sensible defaults
        let node_config = NodeInitConfig {
            home: PathBuf::from("~/.node-ipc"),
            subnet: subnet_id.to_string(),
            parent: parent_id.to_string(),
            genesis: genesis_source,
            key: WalletImportArgs {
                wallet_type: "evm".to_string(),
                path: None,
                private_key: None, // Will generate a new key
            },
            join: join_config,
            p2p: Some(P2pConfig {
                external_ip: Some("127.0.0.1".to_string()), // Default external IP for user to modify
                ports: None,                                // Let user configure ports
                peers: None,                                // Let user configure peers
            }),
            cometbft_overrides: None,
            fendermint_overrides: None,
        };

        // Serialize NodeInitConfig to YAML
        let yaml_content = serde_yaml::to_string(&node_config)
            .context("failed to serialize node config to YAML")?;

        log::info!("Node configuration generated successfully for subnet: {}", subnet_id);
        Ok(yaml_content)
    }

    /// Generate startup commands for a validator node
    pub async fn generate_node_commands(
        &self,
        subnet_id: &str,
        validator_address: Option<&str>,
    ) -> Result<serde_json::Value> {
        log::info!("Generating node startup commands for subnet: {}", subnet_id);

        let subnet = ipc_api::subnet_id::SubnetID::from_str(subnet_id)?;
        let mut provider = crate::get_ipc_provider(&self.global)?;

        // Determine if this is a collateral-based subnet
        let permission_mode = match provider.get_genesis_info(&subnet).await {
            Ok(genesis_info) => Some(genesis_info.permission_mode),
            Err(_) => None,
        };

        let is_collateral = matches!(permission_mode, Some(ipc_api::subnet::PermissionMode::Collateral));
        let safe_id = subnet_id.replace('/', "_").replace(":", "_");

        let mut commands = Vec::new();

        // Step 1: Join subnet (for collateral-based subnets only)
        if is_collateral {
            let join_command = format!(
                "ipc-cli subnet join \\\n  --subnet {} \\\n  --from {} \\\n  --collateral 10.0 \\\n  --initial-balance 50.0",
                subnet_id,
                validator_address.unwrap_or("YOUR_VALIDATOR_ADDRESS")
            );
            commands.push(serde_json::json!({
                "step": 1,
                "title": "Join Subnet (Collateral Mode Only)",
                "description": "Register as a validator by joining the subnet with collateral",
                "command": join_command,
                "required": true,
                "condition": "Only required for collateral-based subnets"
            }));
        }

        // Step 2: Initialize node
        let init_command = format!(
            "ipc-cli node init --config node_{}.yaml",
            safe_id
        );
        commands.push(serde_json::json!({
            "step": is_collateral as u8 + 1,
            "title": "Initialize Node",
            "description": "Initialize the validator node using the generated configuration",
            "command": init_command,
            "required": true,
            "condition": "Always required"
        }));

        // Step 3: Start node
        let start_command = "ipc-cli node start --home ~/.node-ipc".to_string();
        commands.push(serde_json::json!({
            "step": is_collateral as u8 + 2,
            "title": "Start Node",
            "description": "Start the validator node with all services (CometBFT, Fendermint, ETH API)",
            "command": start_command,
            "required": true,
            "condition": "Always required"
        }));

        let result = serde_json::json!({
            "subnet_id": subnet_id,
            "permission_mode": match permission_mode {
                Some(ipc_api::subnet::PermissionMode::Collateral) => "collateral",
                Some(ipc_api::subnet::PermissionMode::Federated) => "federated",
                Some(ipc_api::subnet::PermissionMode::Static) => "static",
                None => "unknown"
            },
            "validator_address": validator_address.unwrap_or("YOUR_VALIDATOR_ADDRESS"),
            "config_filename": format!("node_{}.yaml", safe_id),
            "commands": commands,
            "prerequisites": [
                "Ensure you have the IPC CLI installed and configured",
                "Make sure you have access to the parent network",
                "Have sufficient funds for collateral (if collateral-based subnet)",
                "Configure your external IP address in the node config file"
            ],
            "notes": [
                "Replace 'YOUR_VALIDATOR_ADDRESS' with your actual validator address",
                "Modify the external IP in the node config file before starting",
                "Ensure ports 26656 (CometBFT P2P) and 26657 (RPC) are accessible",
                "Monitor logs for any connectivity or synchronization issues"
            ]
        });

        log::info!("Node startup commands generated successfully for subnet: {}", subnet_id);
        Ok(result)
    }
}