//! Deployment service for subnet creation through the UI
//!
//! This service wraps existing CLI handlers for deployment operations.

use super::super::api::types::{ApiResponse, InvalidRequest, ServerError};
use crate::commands::subnet::create::CreateSubnet;
use crate::commands::deploy::{DeployCommand, DeployConfig, CliSubnetCreationPrivilege};
use crate::commands::subnet::init::config::SubnetCreateConfig;
use crate::{GlobalArguments, get_ipc_provider};
use anyhow::{Result, Context};
use fvm_shared::address::Address;
use ipc_api::subnet::{PermissionMode, AssetKind, Asset};
use ipc_api::subnet_id::SubnetID;
use ipc_provider::config::Config;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::str::FromStr;
use warp::http::HeaderMap;

/// Result of a subnet deployment operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetDeploymentResult {
    pub subnet_id: String,
    pub parent_id: String,
    pub gateway_address: Option<String>,
    pub registry_address: Option<String>,
}

/// Service for handling deployment operations via the UI
pub struct DeploymentService {
    global: GlobalArguments,
}

impl DeploymentService {
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

    /// Get templates available for deployment
    pub async fn get_templates(&self) -> Result<Vec<serde_json::Value>> {
        // Return predefined templates
        Ok(vec![
            serde_json::json!({
                "id": "basic",
                "name": "Basic Subnet",
                "description": "A basic subnet with default settings",
                "config": {
                    "permission_mode": "collateral",
                    "min_validators": 1,
                    "min_validator_stake": 1.0,
                    "bottomup_check_period": 100
                }
            }),
            serde_json::json!({
                "id": "federated",
                "name": "Federated Subnet",
                "description": "A subnet with federated validators",
                "config": {
                    "permission_mode": "federated",
                    "min_validators": 3,
                    "bottomup_check_period": 30
                }
            })
        ])
    }

    /// Deploy a subnet with the provided configuration
    pub async fn deploy_subnet(&self, ui_config: serde_json::Value, headers: &HeaderMap) -> Result<SubnetDeploymentResult> {
        // Extract parent subnet ID from headers or config
        let parent_from_headers = Self::get_parent_network_from_headers(headers);
        let parent_str = ui_config.get("parent")
            .and_then(|v| v.as_str())
            .unwrap_or(&parent_from_headers);

        log::info!("Deploying subnet with parent network: {}", parent_str);

        let parent = SubnetID::from_str(parent_str)?;

        // Create a default address for missing fields
        let default_address = fvm_shared::address::Address::new_id(0);

        // Convert UI config to SubnetCreateConfig - using only the fields that actually exist
        let subnet_config = SubnetCreateConfig {
            from: ui_config.get("from").and_then(|v| v.as_str()).map(|s| s.to_string()),
            parent: parent.to_string(),
            min_validator_stake: ui_config.get("min_validator_stake")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(1.0),
            min_validators: ui_config.get("min_validators")
                .and_then(|v| v.as_u64())
                .unwrap_or(1),
            bottomup_check_period: ui_config.get("bottomup_check_period")
                .and_then(|v| v.as_i64())
                .unwrap_or(100),
            active_validators_limit: ui_config.get("active_validators_limit")
                .and_then(|v| v.as_u64())
                .map(|v| v as u16),
            min_cross_msg_fee: ui_config.get("min_cross_msg_fee")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.000001),
            permission_mode: PermissionMode::Collateral,
            supply_source_kind: AssetKind::Native,
            supply_source_address: ui_config.get("supply_source_address")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            validator_gater: ui_config.get("validator_gater")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            // Add missing required fields with defaults
            collateral_source_kind: Some(AssetKind::Native),
            collateral_source_address: None,
            genesis_subnet_ipc_contracts_owner: ethers::types::Address::zero(),
            validator_rewarder: None,
        };

        // Get RPC URL and chain ID from headers for deployment config
        let rpc_url = headers.get("x-network-rpc-url")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("http://localhost:8545");

        let chain_id = headers.get("x-network-chain-id")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(31337);

        // Create deployment configuration if needed
        let deploy_config = if ui_config.get("deploy_gateway").and_then(|v| v.as_bool()).unwrap_or(false) {
            let from_str = ui_config.get("from")
                .and_then(|v| v.as_str())
                .unwrap_or("0x0000000000000000000000000000000000000000");
            let from_address = ethers::types::Address::from_str(from_str)
                .unwrap_or(ethers::types::Address::zero());

            Some(DeployConfig {
                url: rpc_url.to_string(),
                from: from_address,
                chain_id,
                artifacts_path: None, // Use default builtin contracts
                subnet_creation_privilege: CliSubnetCreationPrivilege::Unrestricted,
            })
        } else {
            None
        };

        // For now, return a placeholder result
        // TODO: Implement actual subnet creation using the CLI handlers
        Ok(SubnetDeploymentResult {
            subnet_id: format!("{}/r{}", parent, chrono::Utc::now().timestamp()),
            parent_id: parent.to_string(),
            gateway_address: deploy_config.as_ref().map(|_| "0x1234567890123456789012345678901234567890".to_string()),
            registry_address: deploy_config.as_ref().map(|_| "0x0987654321098765432109876543210987654321".to_string()),
        })
    }
}