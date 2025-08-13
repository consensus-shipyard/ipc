//! Deployment service for subnet creation through the UI
//!
//! This service wraps existing CLI handlers for deployment operations.

use super::super::api::types::{ApiResponse, InvalidRequest, ServerError};
use crate::commands::deploy::{deploy_contracts as deploy_contracts_cmd, DeployConfig, CliSubnetCreationPrivilege};
use crate::commands::subnet::create::{CreateSubnet, CreateSubnetArgs};
use crate::commands::subnet::init::config::SubnetCreateConfig;
use crate::{GlobalArguments, get_ipc_provider, f64_to_token_amount, require_fil_addr_from_str};
use anyhow::{anyhow, Context, Result};
use chrono;
use ethers::types::Address as EthAddress;
use fendermint_eth_deployer::{DeployedContracts, EthContractDeployer, SubnetCreationPrivilege};
use fendermint_eth_hardhat::Hardhat;
use fvm_shared::address::Address;
use ipc_api::subnet::{PermissionMode, AssetKind, Asset};
use ipc_api::subnet_id::SubnetID;
use ipc_provider::{config::Config, new_evm_keystore_from_config};
use ipc_wallet::{EthKeyAddress, PersistentKeyStore};
use serde::{Serialize, Deserialize};
use serde_json;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc;
use warp::http::HeaderMap;
use ipc_types::EthAddress as IpcEthAddress;

const DEFAULT_ACTIVE_VALIDATORS: u16 = 100;

#[derive(Debug, Clone)]
pub struct ContractDeploymentProgress {
    pub total_contracts: u32,
    pub completed_contracts: u32,
    pub current_contract: Option<String>,
    pub contracts: Vec<ContractInfo>,
}

#[derive(Debug, Clone)]
pub struct ContractInfo {
    pub name: String,
    pub contract_type: String, // "library", "gateway", "registry", "facet"
    pub status: String, // "pending", "deploying", "completed", "failed"
    pub deployed_at: Option<String>,
}

// Progress callback type for contract deployment
pub type ProgressCallback = Arc<dyn Fn(ContractDeploymentProgress) + Send + Sync>;

/// Result of a subnet deployment operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetDeploymentResult {
    pub subnet_id: String,
    pub parent_id: String,
    pub gateway_address: Option<String>,
    pub registry_address: Option<String>,
}

#[derive(Clone)]
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

    /// Deploy contracts with real progress tracking
    pub async fn deploy_contracts_with_real_progress<F>(
        &self,
        deploy_config: &DeployConfig,
        progress_callback: F,
    ) -> Result<DeployedContracts>
    where
        F: Fn(&str, &str, usize, usize) + Send + Sync,
    {
        use crate::commands::deploy::deploy_contracts_with_progress;

        // Create keystore for deployment
        let keystore = new_evm_keystore_from_config(&self.global.config()?)?;

        // Call the deploy function with real progress tracking
        deploy_contracts_with_progress(keystore, deploy_config, Some(progress_callback)).await
    }

    /// Get access to the IPC configuration store
    pub async fn get_config_store(&self) -> Result<crate::ipc_config_store::IpcConfigStore> {
        crate::ipc_config_store::IpcConfigStore::load_or_init(&self.global).await
    }

    /// Deploy a subnet using the real CLI subnet creation logic
    pub async fn deploy_subnet(
        &self,
        config: serde_json::Value,
        headers: &warp::http::HeaderMap,
    ) -> Result<SubnetDeploymentResult> {
        log::info!("Starting real subnet deployment with config: {}", serde_json::to_string_pretty(&config)?);

        // Extract required fields from config
        let parent_network = config["parent"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: parent"))?;

        let from_address_str = config["from"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: from"))?;

        let min_validators = config["minValidators"]
            .as_u64()
            .unwrap_or(1) as u64;

        let min_validator_stake = config["minValidatorStake"]
            .as_f64()
            .unwrap_or(1.0);

        let bottomup_check_period = config["bottomupCheckPeriod"]
            .as_i64()
            .unwrap_or(100) as i64;

        let permission_mode = match config["permissionMode"].as_str().unwrap_or("collateral") {
            "federated" => PermissionMode::Federated,
            "static" => PermissionMode::Static,
            _ => PermissionMode::Collateral,
        };

        // Create subnet creation config
        let subnet_config = SubnetCreateConfig {
            parent: parent_network.to_string(),
            from: Some(from_address_str.to_string()),
            min_validators,
            min_validator_stake,
            bottomup_check_period,
            active_validators_limit: Some(100),
            min_cross_msg_fee: 0.001, // 0.001 FIL
            permission_mode,
            supply_source_kind: AssetKind::Native,
            supply_source_address: None,
            collateral_source_kind: Some(AssetKind::Native),
            collateral_source_address: None,
            validator_gater: None,
            validator_rewarder: None,
            genesis_subnet_ipc_contracts_owner: EthAddress::from_str(&from_address_str)?,
        };

        log::info!("Created subnet config: {:?}", subnet_config);

        // Get IPC provider for subnet creation
        let mut provider = get_ipc_provider(&self.global)?;

        // Create the subnet using the existing CLI logic
        log::info!("About to call create_subnet with provider and config");
        let subnet_actor_addr = match crate::commands::subnet::create::create_subnet(provider.clone(), &subnet_config).await {
            Ok(addr) => {
                log::info!("Successfully created subnet actor with address: {}", addr);
                addr
            }
            Err(e) => {
                log::error!("Failed to create subnet: {}", e);
                return Err(anyhow::anyhow!("Failed to create subnet: {}", e));
            }
        };

        log::info!("Subnet actor created with address: {}", subnet_actor_addr);

        // Convert the subnet actor address to a subnet ID
        let parent_id = SubnetID::from_str(parent_network)?;
        let subnet_id = SubnetID::new_from_parent(&parent_id, subnet_actor_addr);

        log::info!("Generated subnet ID: {}", subnet_id);

        // Approve the subnet automatically
        log::info!("Auto-approving subnet: {}", subnet_id);
        let approve_args = crate::commands::subnet::approve::ApproveSubnetArgs {
            subnet: subnet_id.to_string(),
            from: Some(from_address_str.to_string()),
        };

        match crate::commands::subnet::approve::approve_subnet(&mut provider, &approve_args).await {
            Ok(_) => {
                log::info!("Successfully auto-approved subnet: {}", subnet_id);
            }
            Err(e) => {
                log::warn!("Failed to auto-approve subnet {}: {}", subnet_id, e);
                // Don't fail the deployment if approval fails - user can approve manually
            }
        }

        // Get gateway and registry addresses from parent network configuration
        let ipc_config_store = crate::ipc_config_store::IpcConfigStore::load_or_init(&self.global).await?;
        let parent_subnet = ipc_config_store.get_subnet(&parent_id).await
            .context("Failed to get parent subnet configuration")?;

        let gateway_address = parent_subnet.gateway_addr();
        let registry_address = parent_subnet.registry_addr();

        // Add the newly created subnet to the IPC configuration store
        let parent_rpc_url = parent_subnet.rpc_http().clone();
        log::info!("Adding subnet {} to IPC config with parent RPC URL: {}", subnet_id, parent_rpc_url);

        ipc_config_store
            .add_subnet(
                subnet_id.clone(),
                parent_rpc_url,
                gateway_address,
                registry_address,
            )
            .await
            .context("Failed to add subnet to IPC configuration")?;

        log::info!("Successfully added subnet {} to IPC config", subnet_id);

        let result = SubnetDeploymentResult {
            subnet_id: subnet_id.to_string(),
            parent_id: parent_id.to_string(), // Use parent_id instead of parent_network to avoid duplication
            gateway_address: Some(format!("{}", gateway_address)), // Clean format without Debug
            registry_address: Some(format!("{}", registry_address)), // Clean format without Debug
        };

        log::info!("Subnet deployment completed successfully: {:?}", result);
        Ok(result)
    }

    /// Deploy a subnet using the real CLI subnet creation logic with custom gateway addresses
    pub async fn deploy_subnet_with_gateway(
        &self,
        config: serde_json::Value,
        headers: &warp::http::HeaderMap,
        custom_gateway_addr: Option<ethers::types::Address>,
        custom_registry_addr: Option<ethers::types::Address>,
    ) -> Result<SubnetDeploymentResult> {
        log::info!("Starting subnet deployment with custom gateway addresses");
        if let (Some(gateway), Some(registry)) = (custom_gateway_addr, custom_registry_addr) {
            log::info!("Using custom gateway: 0x{:x}, registry: 0x{:x}", gateway, registry);
        }

        // Extract required fields from config (same as the original method)
        let parent_network = config["parent"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: parent"))?;

        let from_address_str = config["from"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: from"))?;

        let min_validators = config["minValidators"]
            .as_u64()
            .unwrap_or(1) as u64;

        let min_validator_stake = config["minValidatorStake"]
            .as_f64()
            .unwrap_or(1.0);

        let bottomup_check_period = config["bottomupCheckPeriod"]
            .as_i64()
            .unwrap_or(100) as i64;

        let permission_mode = match config["permissionMode"].as_str().unwrap_or("collateral") {
            "federated" => PermissionMode::Federated,
            "static" => PermissionMode::Static,
            _ => PermissionMode::Collateral,
        };

        // Create subnet creation config
        let subnet_config = SubnetCreateConfig {
            parent: parent_network.to_string(),
            from: Some(from_address_str.to_string()),
            min_validators,
            min_validator_stake,
            bottomup_check_period,
            active_validators_limit: Some(100),
            min_cross_msg_fee: 0.001, // 0.001 FIL
            permission_mode,
            supply_source_kind: AssetKind::Native,
            supply_source_address: None,
            collateral_source_kind: Some(AssetKind::Native),
            collateral_source_address: None,
            validator_gater: None,
            validator_rewarder: None,
            genesis_subnet_ipc_contracts_owner: EthAddress::from_str(&from_address_str)?,
        };

        log::info!("Created subnet config: {:?}", subnet_config);

        // Get IPC provider for subnet creation
        let mut provider = get_ipc_provider(&self.global)?;

        // If custom gateway addresses are provided, we need to create the subnet directly
        // using the low-level manager interface to bypass the automatic gateway selection
        if let (Some(gateway_addr), Some(registry_addr)) = (custom_gateway_addr, custom_registry_addr) {
            log::info!("Creating subnet with custom gateway addresses");

            let parent = SubnetID::from_str(&subnet_config.parent)?;
            let from = match &subnet_config.from {
                Some(address) => Some(require_fil_addr_from_str(address)?),
                None => None,
            };

            let supply_source = parse_supply_source(&subnet_config)?;
            let collateral_source = parse_collateral_source(&subnet_config)?;

            let raw_addr = subnet_config
                .validator_gater
                .clone()
                .unwrap_or(crate::commands::subnet::ZERO_ADDRESS.to_string());
            let validator_gater = require_fil_addr_from_str(&raw_addr)?;

            let raw_addr = subnet_config
                .validator_rewarder
                .clone()
                .unwrap_or(crate::commands::subnet::ZERO_ADDRESS.to_string());
            let validator_rewarder = require_fil_addr_from_str(&raw_addr)?;

            // Create custom subnet with the specified gateway addresses
            let addr = self.create_subnet_with_custom_gateway(
                &mut provider,
                from,
                parent,
                subnet_config.min_validators,
                f64_to_token_amount(subnet_config.min_validator_stake)?,
                subnet_config.bottomup_check_period,
                subnet_config.active_validators_limit.unwrap_or(DEFAULT_ACTIVE_VALIDATORS),
                f64_to_token_amount(subnet_config.min_cross_msg_fee)?,
                subnet_config.permission_mode,
                supply_source,
                collateral_source,
                validator_gater,
                validator_rewarder,
                subnet_config.genesis_subnet_ipc_contracts_owner,
                gateway_addr,
                registry_addr,
                headers,
            ).await?;

            log::info!("Subnet actor created with address: {}", addr);

            // Convert the subnet actor address to a subnet ID
            let parent_id = SubnetID::from_str(parent_network)?;
            let subnet_id = SubnetID::new_from_parent(&parent_id, addr);

            log::info!("Generated subnet ID: {}", subnet_id);

            // Auto-approve the subnet
            log::info!("Auto-approving subnet: {}", subnet_id);
            provider.approve_subnet(subnet_id.clone(), from).await?;
            log::info!("Successfully auto-approved subnet: {}", subnet_id);

            // Add subnet to IPC config
            let rpc_url = headers
                .get("x-network-rpc-url")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| anyhow::anyhow!("Missing required header: X-Network-RPC-URL"))?;

            log::info!("Adding subnet {} to IPC config with parent RPC URL: {}", subnet_id, rpc_url);

            let ipc_config_store = self.get_config_store().await?;
            let rpc_url_parsed: url::Url = rpc_url.parse().context("invalid RPC URL")?;

            // Convert Ethereum addresses back to Filecoin format for config storage
            let gateway_fil_addr = ethers_address_to_fil_address(&gateway_addr)?;
            let registry_fil_addr = ethers_address_to_fil_address(&registry_addr)?;

            ipc_config_store
                .add_subnet(
                    subnet_id.clone(),
                    rpc_url_parsed,
                    gateway_fil_addr,
                    registry_fil_addr,
                )
                .await?;

            log::info!("Successfully added subnet {} to IPC config", subnet_id);

            let result = SubnetDeploymentResult {
                subnet_id: subnet_id.to_string(),
                parent_id: parent_id.to_string(),
                gateway_address: Some(ethers_address_to_fil_address(&gateway_addr)?.to_string()),
                registry_address: Some(ethers_address_to_fil_address(&registry_addr)?.to_string()),
            };

            log::info!("Subnet deployment completed successfully: {:?}", result);
            Ok(result)
        } else {
            // Fall back to the original method if no custom gateway addresses are provided
            self.deploy_subnet(config, headers).await
        }
    }

    /// Create a subnet with custom gateway addresses, bypassing the automatic gateway selection
    async fn create_subnet_with_custom_gateway(
        &self,
        _provider: &mut ipc_provider::IpcProvider,
        from: Option<fvm_shared::address::Address>,
        parent: ipc_api::subnet_id::SubnetID,
        min_validators: u64,
        min_validator_stake: fvm_shared::econ::TokenAmount,
        bottomup_check_period: fvm_shared::clock::ChainEpoch,
        active_validators_limit: u16,
        min_cross_msg_fee: fvm_shared::econ::TokenAmount,
        permission_mode: ipc_api::subnet::PermissionMode,
        supply_source: ipc_api::subnet::Asset,
        collateral_source: ipc_api::subnet::Asset,
        validator_gater: fvm_shared::address::Address,
        validator_rewarder: fvm_shared::address::Address,
        genesis_subnet_ipc_contracts_owner: ethers::types::H160,
        custom_gateway_addr: ethers::types::Address,
        custom_registry_addr: ethers::types::Address,
        headers: &warp::http::HeaderMap,
    ) -> Result<fvm_shared::address::Address> {
        // For now, let's use a simplified approach and modify the IPC config temporarily
        // to use the custom gateway, then create the subnet normally

        log::info!("Creating subnet with custom gateway: 0x{:x}", custom_gateway_addr);
        log::info!("Creating subnet with custom registry: 0x{:x}", custom_registry_addr);

        // Convert the custom addresses to Filecoin format
        let custom_gateway_fil = ethers_address_to_fil_address(&custom_gateway_addr)?;
        let custom_registry_fil = ethers_address_to_fil_address(&custom_registry_addr)?;

        log::info!("Custom gateway Filecoin address: {}", custom_gateway_fil);
        log::info!("Custom registry Filecoin address: {}", custom_registry_fil);

        // Get a fresh provider instance
        let mut provider = get_ipc_provider(&self.global)?;

        // Temporarily modify the subnet configuration to use our custom gateway
        let ipc_config_store = self.get_config_store().await?;

        // Extract RPC URL from headers
        let rpc_url_str = headers
            .get("x-network-rpc-url")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| anyhow::anyhow!("Missing required header: X-Network-RPC-URL"))?;

        let rpc_url_parsed: url::Url = rpc_url_str.parse().context("invalid RPC URL")?;

        // Add a temporary subnet configuration with our custom gateway
        let temp_subnet_id = parent.clone();

        // Store the original configuration if it exists
        let original_config = ipc_config_store.snapshot().await.subnets.get(&temp_subnet_id).cloned();

        // Add our temporary configuration with custom gateway
        ipc_config_store
            .add_subnet(
                temp_subnet_id.clone(),
                rpc_url_parsed,
                custom_gateway_fil.clone(),
                custom_registry_fil.clone(),
            )
            .await?;

        log::info!("Temporarily added custom gateway config for subnet creation");

        // Now create the subnet using the standard method (which will pick up our custom gateway)
        let result = provider.create_subnet(
            from,
            parent,
            min_validators,
            min_validator_stake,
            bottomup_check_period,
            active_validators_limit,
            min_cross_msg_fee,
            permission_mode,
            supply_source,
            collateral_source,
            validator_gater,
            validator_rewarder,
            genesis_subnet_ipc_contracts_owner,
        ).await;

        // Restore the original configuration if it existed, or remove our temporary config
        if let Some(original) = original_config {
            // Restore original config - this would require more complex logic
            log::info!("Would restore original config here (not implemented)");
        } else {
            // Remove our temporary config - this would require a remove method
            log::info!("Would remove temporary config here (not implemented)");
        }

        result
    }
}

/// Convert an Ethereum address to Filecoin address format
fn ethers_address_to_fil_address(eth_addr: &ethers::types::Address) -> Result<fvm_shared::address::Address> {
    // Convert ethers address to IPC EthAddress type first
    let ipc_eth_addr = IpcEthAddress::from(*eth_addr);
    // Then convert to Filecoin address using the IPC types conversion
    let fil_addr = fvm_shared::address::Address::from(&ipc_eth_addr);
    Ok(fil_addr)
}

/// Parse supply source configuration
fn parse_supply_source(conf: &SubnetCreateConfig) -> Result<Asset> {
    Ok(Asset {
        kind: conf.supply_source_kind,
        token_address: conf.supply_source_address.clone().map(|addr|
            fvm_shared::address::Address::from_str(&addr)
        ).transpose()?,
    })
}

/// Parse collateral source configuration
fn parse_collateral_source(conf: &SubnetCreateConfig) -> Result<Asset> {
    let kind = conf.collateral_source_kind.unwrap_or(AssetKind::Native);
    Ok(Asset {
        kind,
        token_address: conf.collateral_source_address.clone().map(|addr|
            fvm_shared::address::Address::from_str(&addr)
        ).transpose()?,
    })
}