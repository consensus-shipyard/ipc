// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Gateway service for managing gateway contracts through the UI
//!
//! This service provides methods for discovering and managing IPC gateway contracts.

use super::super::api::types::{ApiResponse, InvalidRequest, ServerError};
use crate::ipc_config_store::IpcConfigStore;
use crate::{GlobalArguments, get_ipc_provider};
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use warp::http::HeaderMap;
use ipc_api::subnet_id::SubnetID;

/// Gateway information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayInfo {
    pub id: String,
    pub address: String,
    pub registry_address: String,
    pub deployer_address: String,
    pub parent_network: String,
    pub name: Option<String>,
    pub subnet_count: u32,
    pub is_active: bool,
    pub deployed_at: chrono::DateTime<chrono::Utc>,
}

/// Service for handling gateway operations via the UI
pub struct GatewayService {
    global: GlobalArguments,
}

impl GatewayService {
    pub fn new(global: GlobalArguments) -> Self {
        Self { global }
    }

    /// Get the parent network from network headers or default
    fn get_parent_network_from_headers(headers: &HeaderMap) -> String {
        // Extract chain ID from network headers
        if let Some(chain_id_header) = headers.get("x-network-chain-id") {
            if let Ok(chain_id_str) = chain_id_header.to_str() {
                if let Ok(chain_id) = chain_id_str.parse::<u64>() {
                    // Map common chain IDs to their subnet IDs
                    return match chain_id {
                        31337 => "/r31337".to_string(),      // Local Anvil
                        314159 => "/r314159".to_string(),    // Calibration Testnet
                        1 => "/r1".to_string(),              // Ethereum Mainnet
                        _ => format!("/r{}", chain_id),      // Generic mapping
                    };
                }
            }
        }

        // Try to get from network type header as fallback
        if let Some(network_type_header) = headers.get("x-network-type") {
            if let Ok(network_type) = network_type_header.to_str() {
                return match network_type {
                    "local" => "/r31337".to_string(),
                    "testnet" => "/r314159".to_string(),
                    "mainnet" => "/r1".to_string(),
                    _ => "/r314159".to_string(), // Default to testnet
                };
            }
        }

        // Default fallback
        "/r314159".to_string()
    }

    /// Discover gateways from the IPC configuration, optionally filtered by network
    pub async fn discover_gateways(&self, headers: Option<&HeaderMap>) -> Result<Vec<GatewayInfo>> {
        let config_store = IpcConfigStore::load_or_init(&self.global).await?;
        let config = config_store.snapshot().await;

        let mut gateways_map: std::collections::HashMap<String, GatewayInfo> = std::collections::HashMap::new();
        let target_network = headers.map(|h| Self::get_parent_network_from_headers(h));

        log::info!("Discovering gateways, target network: {:?}", target_network);

        // Iterate through subnets in the configuration to find gateway information
        for (subnet_id_str, subnet_config) in &config.subnets {
            // Filter by target network if specified
            if let Some(ref target) = target_network {
                // Check if this subnet belongs to the target parent network
                if let Ok(subnet_id) = SubnetID::from_str(&subnet_id_str.to_string()) {
                    if subnet_id.is_root() {
                        // This is a root network, check if it matches
                        if subnet_id_str.to_string() != *target {
                            continue;
                        }
                    } else {
                        // This is a subnet, check if its parent matches
                        if let Some(parent_subnet) = subnet_id.parent() {
                            if parent_subnet.to_string() != *target {
                                continue;
                            }
                        } else {
                            // Fallback: check if the subnet string contains the target
                            if !subnet_id_str.to_string().starts_with(target) {
                                continue;
                            }
                        }
                    }
                } else {
                    log::warn!("Failed to parse subnet ID: {}", subnet_id_str.to_string());
                    continue;
                }
            }

            // Try to get gateway information from the subnet configuration
            match &subnet_config.config {
                ipc_provider::config::SubnetConfig::Fevm(evm_subnet) => {
                    let gateway_addr_str = evm_subnet.gateway_addr.to_string();
                    let registry_addr_str = evm_subnet.registry_addr.to_string();

                    // Use gateway address as the key for deduplication
                    let gateway_key = gateway_addr_str.clone();

                    if let Some(existing_gateway) = gateways_map.get_mut(&gateway_key) {
                        // Gateway already exists, increment subnet count
                        existing_gateway.subnet_count += 1;
                        log::info!("Found additional subnet {} for existing gateway {}", subnet_id_str, gateway_addr_str);
                    } else {
                        // New gateway, create entry
                        let parent_network = if let Ok(subnet_id) = SubnetID::from_str(&subnet_id_str.to_string()) {
                            if subnet_id.is_root() {
                                subnet_id_str.to_string()
                            } else {
                                subnet_id.parent().map(|p| p.to_string()).unwrap_or_else(|| subnet_id_str.to_string())
                            }
                        } else {
                            subnet_id_str.to_string()
                        };

                        let gateway_info = GatewayInfo {
                            id: format!("gateway-{}", &gateway_addr_str[gateway_addr_str.len().saturating_sub(12)..]),
                            address: gateway_addr_str.clone(),
                            registry_address: registry_addr_str,
                            deployer_address: "unknown".to_string(), // TODO: Track deployer
                            parent_network,
                            name: Some(format!("Gateway {}", &gateway_addr_str[gateway_addr_str.len().saturating_sub(8)..])),
                            subnet_count: 1,
                            is_active: true, // Assume active if in config
                            deployed_at: chrono::Utc::now(), // TODO: Track actual deployment time
                        };

                        log::info!("Found new gateway: {} with ID: {} serving subnet: {}", gateway_addr_str, gateway_info.id, subnet_id_str);
                        gateways_map.insert(gateway_key, gateway_info);
                    }
                }
                _ => {
                    // Skip non-EVM subnets for now
                    continue;
                }
            }
        }

        // Convert HashMap to Vec
        let gateways: Vec<GatewayInfo> = gateways_map.into_values().collect();
        log::info!("Discovered {} unique gateways for target network {:?}", gateways.len(), target_network);

        Ok(gateways)
    }

    /// List deployed contracts for gateways
    pub async fn list_deployed_contracts(&self) -> Result<Vec<serde_json::Value>> {
        let gateways = self.discover_gateways(None).await?; // No headers for this call
        let mut contracts = Vec::new();

        for gateway in &gateways {
            // Add gateway contract
            contracts.push(serde_json::json!({
                "id": format!("gateway-{}", gateway.address),
                "name": format!("Gateway Contract - {}", gateway.name.as_ref().unwrap_or(&"Unnamed".to_string())),
                "address": gateway.address,
                "type": "Gateway",
                "network": gateway.parent_network,
                "deployed_at": gateway.deployed_at,
                "is_active": gateway.is_active,
            }));

            // Add registry contract if available
            if !gateway.registry_address.is_empty() {
                contracts.push(serde_json::json!({
                    "id": format!("registry-{}", gateway.registry_address),
                    "name": format!("Registry Contract - {}", gateway.name.as_ref().unwrap_or(&"Unnamed".to_string())),
                    "address": gateway.registry_address,
                    "type": "Registry",
                    "network": gateway.parent_network,
                    "deployed_at": gateway.deployed_at,
                    "is_active": gateway.is_active,
                }));
            }
        }

        Ok(contracts)
    }

    /// Get detailed information about a specific gateway
    pub async fn get_gateway_info(&self, gateway_id: &str) -> Result<Option<GatewayInfo>> {
        let gateways = self.discover_gateways(None).await?; // No headers for this call
        Ok(gateways.into_iter().find(|g| g.id == gateway_id))
    }

    /// Update gateway information
    pub async fn update_gateway(&self, gateway_id: &str, updates: &serde_json::Value) -> Result<GatewayInfo> {
        // For now, this is a placeholder implementation
        // In a real implementation, this would update the gateway configuration

        let name = updates.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Create a mock updated gateway info
        let gateway_info = GatewayInfo {
            id: gateway_id.to_string(),
            address: "0x0000000000000000000000000000000000000000".to_string(),
            registry_address: "0x0000000000000000000000000000000000000000".to_string(),
            deployer_address: "0x0000000000000000000000000000000000000000".to_string(),
            parent_network: "/r314159".to_string(),
            name,
            subnet_count: 0,
            is_active: true,
            deployed_at: chrono::Utc::now(),
        };

        Ok(gateway_info)
    }

    /// Track a newly deployed gateway
    pub async fn track_deployed_gateway(
        &self,
        gateway_address: String,
        registry_address: String,
        deployer_address: String,
        parent_network: String,
        name: Option<String>,
    ) -> Result<GatewayInfo> {
        let gateway_info = GatewayInfo {
            id: format!("gateway-{}", gateway_address),
            address: gateway_address,
            registry_address,
            deployer_address,
            parent_network,
            name,
            subnet_count: 0,
            is_active: true,
            deployed_at: chrono::Utc::now(),
        };

        // TODO: Persist the gateway information to configuration or storage

        Ok(gateway_info)
    }

    /// Get gateway statistics
    pub async fn get_gateway_stats(&self, gateway_id: &str) -> Result<serde_json::Value> {
        // TODO: Get actual gateway statistics from the provider
        Ok(serde_json::json!({
            "total_transactions": 0,
            "active_subnets": 0,
            "total_value_locked": "0",
            "last_activity": "2024-01-01T00:00:00Z",
            "uptime_percentage": 100.0,
            "gas_used": "0",
            "fees_collected": "0"
        }))
    }

    /// Validate gateway configuration
    pub async fn validate_gateway(&self, gateway_address: &str) -> Result<bool> {
        let mut provider = get_ipc_provider(&self.global)?;

        // Try to connect to the gateway and validate it's working
        // This is a simplified check - in reality, you'd want to perform
        // more comprehensive validation

        // For now, just check if the address is valid
        let _address = ethers::types::Address::from_str(gateway_address)
            .context("Invalid gateway address format")?;

        // TODO: Add actual gateway validation logic
        // - Check if gateway contract exists
        // - Verify gateway is properly configured
        // - Test connectivity

        Ok(true)
    }

    /// Get all contracts associated with gateways
    pub async fn get_gateway_contracts(&self) -> Result<Vec<serde_json::Value>> {
        let gateways = self.discover_gateways(None).await?; // No headers for this call

        let mut contracts = Vec::new();

        for gateway in &gateways {
            // Add gateway contract
            contracts.push(serde_json::json!({
                "id": format!("gateway-{}", gateway.address),
                "name": format!("Gateway Contract - {}", gateway.name.as_ref().unwrap_or(&"Unnamed".to_string())),
                "address": gateway.address,
                "type": "Gateway",
                "network": gateway.parent_network,
                "deployed_at": gateway.deployed_at,
                "is_active": gateway.is_active,
            }));

            // Add registry contract if available
            if !gateway.registry_address.is_empty() {
                contracts.push(serde_json::json!({
                    "id": format!("registry-{}", gateway.registry_address),
                    "name": format!("Registry Contract - {}", gateway.name.as_ref().unwrap_or(&"Unnamed".to_string())),
                    "address": gateway.registry_address,
                    "type": "SubnetRegistry",
                    "network": gateway.parent_network,
                    "deployed_at": gateway.deployed_at,
                    "is_active": gateway.is_active,
                }));
            }
        }

        Ok(contracts)
    }

    /// Inspect a gateway contract
    pub async fn inspect_gateway_contract(&self, address: &str) -> Result<serde_json::Value> {
        // TODO: Use the provider to inspect the actual contract
        Ok(serde_json::json!({
            "address": address,
            "type": "Gateway",
            "abi": [], // TODO: Get actual ABI
            "bytecode": "", // TODO: Get actual bytecode if needed
            "functions": [], // TODO: Get contract functions
            "events": [], // TODO: Get contract events
            "storage": {}, // TODO: Get contract storage
        }))
    }
}