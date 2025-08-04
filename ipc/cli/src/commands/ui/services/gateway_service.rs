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

    /// Discover gateways from the IPC configuration
    pub async fn discover_gateways(&self) -> Result<Vec<GatewayInfo>> {
        let config_store = IpcConfigStore::load_or_init(&self.global).await?;
        let config = config_store.snapshot().await;

        let mut gateways = Vec::new();

        // Iterate through subnets in the configuration to find gateway information
        for (subnet_id_str, subnet_config) in &config.subnets {
            // Try to get gateway information from the subnet configuration
            match &subnet_config.config {
                ipc_provider::config::SubnetConfig::Fevm(evm_subnet) => {
                    let gateway_info = GatewayInfo {
                        id: format!("gateway-{}", evm_subnet.gateway_addr),
                        address: evm_subnet.gateway_addr.to_string(),
                        registry_address: evm_subnet.registry_addr.to_string(),
                        deployer_address: "unknown".to_string(), // TODO: Track deployer
                        parent_network: subnet_id_str.to_string(),
                        name: Some(format!("Gateway for {}", subnet_id_str)),
                        subnet_count: 1, // Each gateway typically serves one subnet
                        is_active: true, // Assume active if in config
                        deployed_at: chrono::Utc::now(), // TODO: Track actual deployment time
                    };
                    gateways.push(gateway_info);
                }
                _ => {
                    // Skip non-EVM subnets for now
                    continue;
                }
            }
        }

        Ok(gateways)
    }

    /// List deployed contracts for gateways
    pub async fn list_deployed_contracts(&self) -> Result<Vec<serde_json::Value>> {
        let gateways = self.discover_gateways().await?;
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
        let gateways = self.discover_gateways().await?;
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
        let gateways = self.discover_gateways().await?;

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