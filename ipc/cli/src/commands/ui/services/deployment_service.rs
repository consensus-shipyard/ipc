//! Deployment service for subnet creation through the UI
//!
//! This service wraps existing CLI handlers for deployment operations.

use super::super::api::types::{ApiResponse, InvalidRequest, ServerError};
use crate::commands::subnet::create::{CreateSubnet, CreateSubnetArgs};
use crate::commands::deploy::{DeployCommand, DeployConfig, CliSubnetCreationPrivilege, deploy_contracts};
use crate::commands::subnet::init::config::SubnetCreateConfig;
use crate::{GlobalArguments, get_ipc_provider};
use anyhow::{Result, Context};
use fvm_shared::address::Address;
use ipc_api::subnet::{PermissionMode, AssetKind, Asset};
use ipc_api::subnet_id::SubnetID;
use ipc_provider::{config::Config, new_evm_keystore_from_config};
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

    /// Validate that the deployment address has sufficient balance
    async fn validate_deployment_balance(&self, rpc_url: &str, address: ethers::types::Address) -> Result<(), anyhow::Error> {
        use ethers::prelude::*;

        let provider = Provider::<Http>::try_from(rpc_url)
            .context("Failed to create provider for balance check")?;

        let balance = provider.get_balance(address, None).await
            .context("Failed to get balance for deployment address")?;

        // Require at least 0.1 ETH for deployment (conservative estimate)
        let min_balance = U256::from_dec_str("100000000000000000").unwrap(); // 0.1 ETH in wei

        if balance < min_balance {
            let balance_eth = balance.as_u128() as f64 / 1e18;
            return Err(anyhow::anyhow!(
                "Insufficient balance for deployment. Address {} has {:.6} ETH but needs at least 0.1 ETH for contract deployment. Please fund this address before deploying.",
                address, balance_eth
            ));
        }

        Ok(())
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

        // Always deploy contracts for new networks (like Anvil) that don't have them yet
        let from_str = ui_config.get("from")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0000000000000000000000000000000000000000");
        let from_address = ethers::types::Address::from_str(from_str)
            .unwrap_or(ethers::types::Address::zero());

        // Validate that the from address has sufficient balance for deployment
        self.validate_deployment_balance(rpc_url, from_address).await?;

        let deploy_config = DeployConfig {
            url: rpc_url.to_string(),
            from: from_address,
            chain_id,
            artifacts_path: None, // Use default builtin contracts
            subnet_creation_privilege: CliSubnetCreationPrivilege::Unrestricted,
        };

        // Deploy the IPC contracts first (gateway and registry)
        log::info!("Deploying IPC contracts to {}", rpc_url);
        let deployed_contracts = crate::commands::deploy::deploy_contracts(
            new_evm_keystore_from_config(&self.global.config()?)?,
            &deploy_config
        ).await?;

        log::info!("Deployed contracts - Registry: {:?}, Gateway: {:?}",
                   deployed_contracts.registry, deployed_contracts.gateway);

        // Create the subnet using the actual CLI logic
        let subnet_args = CreateSubnetArgs {
            config: subnet_config,
        };
        let subnet_addr_str = CreateSubnet::create(&self.global, &subnet_args).await?;

        // Build subnet ID from parent + new address
        let subnet_id = format!("{}/{}", parent, subnet_addr_str);

        log::info!("Successfully created subnet: {}", subnet_id);

        Ok(SubnetDeploymentResult {
            subnet_id,
            parent_id: parent.to_string(),
            gateway_address: Some(format!("{:?}", deployed_contracts.gateway)),
            registry_address: Some(format!("{:?}", deployed_contracts.registry)),
        })
    }
}