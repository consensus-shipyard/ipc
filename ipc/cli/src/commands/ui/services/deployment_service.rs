// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Deployment service for subnet creation through the UI
//!
//! This service wraps existing CLI handlers for deployment operations.

use crate::commands::subnet::init::config::{DeployConfig, SubnetCreateConfig};
use crate::{f64_to_token_amount, get_ipc_provider, require_fil_addr_from_str, GlobalArguments};
use anyhow::{anyhow, Context, Result};
use ethers::types::Address as EthAddress;
use fendermint_eth_deployer::DeployedContracts;
use ipc_api::subnet::{Asset, AssetKind, PermissionMode};
use ipc_api::subnet_id::SubnetID;
use ipc_provider::{config::SubnetConfig, new_evm_keystore_from_config};
use ipc_types::EthAddress as IpcEthAddress;
use serde::{Deserialize, Serialize};
use serde_json;
use std::str::FromStr;
use warp::http::HeaderMap;

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
    pub status: String,        // "pending", "deploying", "completed", "failed"
    pub deployed_at: Option<String>,
}

/// Result of a subnet deployment operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetDeploymentResult {
    pub subnet_id: String,
    pub parent_id: String,
    pub gateway_address: Option<String>,
    pub registry_address: Option<String>,
    pub status: String,
    pub message: Option<String>,
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
    #[allow(dead_code)]
    async fn validate_deployment_balance(
        &self,
        rpc_url: &str,
        address: ethers::types::Address,
    ) -> Result<(), anyhow::Error> {
        use ethers::prelude::*;

        let provider = Provider::<Http>::try_from(rpc_url)
            .context("Failed to create provider for balance check")?;

        let balance = provider
            .get_balance(address, None)
            .await
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
    #[allow(dead_code)]
    fn get_parent_network_from_headers(headers: &HeaderMap) -> String {
        // Extract chain ID from network headers
        if let Some(chain_id_header) = headers.get("x-network-chain-id") {
            if let Ok(chain_id_str) = chain_id_header.to_str() {
                if let Ok(chain_id) = chain_id_str.parse::<u64>() {
                    // Map common chain IDs to their subnet IDs
                    return match chain_id {
                        31337 => "/r31337".to_string(),   // Local Anvil
                        314159 => "/r314159".to_string(), // Calibration Testnet
                        1 => "/r1".to_string(),           // Ethereum Mainnet
                        _ => format!("/r{}", chain_id),   // Generic mapping
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
            }),
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
        _headers: &warp::http::HeaderMap,
    ) -> Result<SubnetDeploymentResult> {
        log::info!(
            "Starting real subnet deployment with config: {}",
            serde_json::to_string_pretty(&config)?
        );

        // Extract required fields from config
        let parent_network = config["parent"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: parent"))?;

        let from_address_str = config["from"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: from"))?;

        let min_validators = config["minValidators"].as_u64().unwrap_or(1);

        let min_validator_stake = config["minValidatorStake"].as_f64().unwrap_or(1.0);

        let bottomup_check_period = config["bottomupCheckPeriod"].as_i64().unwrap_or(100);

        let permission_mode = match config["permissionMode"].as_str().unwrap_or("collateral") {
            "federated" => PermissionMode::Federated,
            "static" => PermissionMode::Static,
            _ => PermissionMode::Collateral,
        };

        let subnet_chain_id = config["chainId"]
            .as_u64()
            .ok_or_else(|| anyhow!("missing subnet chain id configuration"))?;

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
            genesis_subnet_ipc_contracts_owner: EthAddress::from_str(from_address_str)?,
            chain_id: subnet_chain_id,
            parent_filecoin_rpc: None,
            parent_filecoin_auth_token: None,
        };

        log::info!("Created subnet config: {:?}", subnet_config);

        // Get IPC provider for subnet creation
        let mut provider = get_ipc_provider(&self.global)?;

        // Create the subnet using the existing CLI logic
        log::info!("About to call create_subnet with provider and config");
        let subnet_actor_addr =
            match crate::commands::subnet::create::create_subnet(provider.clone(), &subnet_config)
                .await
            {
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

        // For federated or static subnets, set federated power first
        if permission_mode == PermissionMode::Federated || permission_mode == PermissionMode::Static
        {
            log::info!(
                "Setting federated power for {:?} subnet: {}",
                permission_mode,
                subnet_id
            );

            // Extract validator configuration from UI config
            let validator_pubkeys = config["validatorPubkeys"]
                .as_array()
                .and_then(|arr| arr.iter().map(|v| v.as_str()).collect::<Option<Vec<_>>>())
                .unwrap_or_default();

            let validator_power = config["validatorPower"]
                .as_array()
                .and_then(|arr| arr.iter().map(|v| v.as_u64()).collect::<Option<Vec<_>>>())
                .unwrap_or_else(|| vec![1; validator_pubkeys.len()]);

            if validator_pubkeys.is_empty() {
                log::error!("No validator public keys provided for federated subnet");
                return Err(anyhow::anyhow!(
                    "Federated subnets require validator public keys to be configured"
                ));
            }

            log::info!(
                "Found {} validators with pubkeys: {:?}",
                validator_pubkeys.len(),
                validator_pubkeys
            );
            log::info!("Validator powers: {:?}", validator_power);

            // Convert public keys to validator addresses
            let validator_addresses: Vec<String> = validator_pubkeys
                .iter()
                .map(|pubkey| {
                    // Convert public key to Ethereum address
                    use ethers::{types::H160, utils::keccak256};
                    use hex::FromHex;

                    let key = pubkey
                        .strip_prefix("0x")
                        .or_else(|| pubkey.strip_prefix("0X"))
                        .unwrap_or(pubkey);
                    let bytes = Vec::from_hex(key)
                        .map_err(|e| anyhow::anyhow!("Invalid public key hex: {}", e))?;
                    if bytes.len() != 65 || bytes[0] != 0x04 {
                        return Err(anyhow::anyhow!(
                            "Expected 65-byte uncompressed key starting with 0x04"
                        ));
                    }
                    let hash = keccak256(&bytes[1..]);
                    let eth_addr = H160::from_slice(&hash[12..]);
                    Ok(format!("{:#x}", eth_addr))
                })
                .collect::<Result<Vec<_>, _>>()?;

            log::info!("Computed validator addresses: {:?}", validator_addresses);

            // Prepare validator pubkeys without 0x prefix for the set_federated_power call
            let pubkeys_clean: Vec<String> = validator_pubkeys
                .iter()
                .map(|pk| {
                    pk.strip_prefix("0x")
                        .or_else(|| pk.strip_prefix("0X"))
                        .unwrap_or(pk)
                        .to_string()
                })
                .collect();

            // Convert power values to u128
            let validator_power_u128: Vec<u128> =
                validator_power.iter().map(|&p| p as u128).collect();

            // CRITICAL FIX: Use the subnet owner's address for set_federated_power
            // The setFederatedPower function can only be called by the subnet owner

            // Need to use the subnet creator address, not the from address?????????
            let owner_address = subnet_config.genesis_subnet_ipc_contracts_owner;
            let owner_address_str = format!("0x{:x}", owner_address);
            log::info!(
                "🔍 Using subnet owner address for set_federated_power: {}",
                owner_address_str
            );

            // Create SetFederatedPowerArgs
            let set_power_args =
                crate::commands::subnet::set_federated_power::SetFederatedPowerArgs {
                    from: owner_address_str, // Use owner, not from! // No maybe use FROM!?!?!?! Is this a problem?
                    subnet: subnet_id.to_string(),
                    validator_addresses,
                    validator_pubkeys: pubkeys_clean,
                    validator_power: validator_power_u128,
                };

            // Call set_federated_power
            match crate::commands::subnet::set_federated_power::set_federated_power(
                &provider,
                &set_power_args,
            )
            .await
            {
                Ok(chain_epoch) => {
                    log::info!("Successfully set federated power at epoch {}", chain_epoch);
                }
                Err(e) => {
                    log::error!("Failed to set federated power: {}", e);
                    return Err(anyhow::anyhow!("Failed to set federated power: {}", e));
                }
            }

            // Wait a moment for the transaction to be processed and bootstrap to potentially occur
            log::info!("Waiting for potential subnet bootstrap...");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            log::info!(
                "Federated power setting completed for subnet: {}",
                subnet_id
            );
        }

        // Approve the subnet automatically
        log::info!("Auto-approving subnet: {}", subnet_id);
        let approve_args = crate::commands::subnet::approve::ApproveSubnetArgs {
            subnet: subnet_id.to_string(),
            from: Some(from_address_str.to_string()),
        };

        let auto_approval_error =
            match crate::commands::subnet::approve::approve_subnet(&mut provider, &approve_args)
                .await
            {
                Ok(_) => {
                    log::info!("Successfully auto-approved subnet: {}", subnet_id);
                    None
                }
                Err(e) => {
                    log::warn!("Failed to auto-approve subnet {}: {}", subnet_id, e);
                    // Don't fail the deployment if approval fails - user can approve manually
                    Some(format!(
                        "Subnet created but not approved: {}. Please approve manually.",
                        e
                    ))
                }
            };

        // Get gateway and registry addresses from parent network configuration
        let ipc_config_store =
            crate::ipc_config_store::IpcConfigStore::load_or_init(&self.global).await?;
        let parent_subnet = ipc_config_store
            .get_subnet(&parent_id)
            .await
            .context("Failed to get parent subnet configuration")?;

        let gateway_address = parent_subnet.gateway_addr();
        let registry_address = parent_subnet.registry_addr();

        // Add the newly created subnet to the IPC configuration store
        let parent_rpc_url = parent_subnet.rpc_http().clone();
        log::info!(
            "Adding subnet {} to IPC config with parent RPC URL: {}",
            subnet_id,
            parent_rpc_url
        );

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
            status: if auto_approval_error.is_some() {
                "pending-approval".to_string()
            } else {
                "active".to_string()
            },
            message: auto_approval_error,
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
        state: &super::super::AppState,
        deployment_id: &str,
    ) -> Result<SubnetDeploymentResult> {
        log::info!("Starting subnet deployment with custom gateway addresses");
        if let (Some(gateway), Some(registry)) = (custom_gateway_addr, custom_registry_addr) {
            log::info!(
                "Using custom gateway: 0x{:x}, registry: 0x{:x}",
                gateway,
                registry
            );
        }

        // Extract required fields from config (same as the original method)
        let parent_network = config["parent"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: parent"))?;

        let from_address_str = config["from"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required field: from"))?;

        let min_validators = config["minValidators"].as_u64().unwrap_or(1);

        let min_validator_stake = config["minValidatorStake"].as_f64().unwrap_or(1.0);

        let bottomup_check_period = config["bottomupCheckPeriod"].as_i64().unwrap_or(100);

        let permission_mode = match config["permissionMode"].as_str().unwrap_or("collateral") {
            "federated" => PermissionMode::Federated,
            "static" => PermissionMode::Static,
            _ => PermissionMode::Collateral,
        };

        let subnet_chain_id = config["chainId"]
            .as_u64()
            .ok_or_else(|| anyhow!("missing subnet chain id configuration"))?;

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
            genesis_subnet_ipc_contracts_owner: EthAddress::from_str(from_address_str)?,
            chain_id: subnet_chain_id,
            parent_filecoin_rpc: None,
            parent_filecoin_auth_token: None,
        };

        log::info!("Created subnet config: {:?}", subnet_config);

        // Get IPC provider for subnet creation
        let mut provider = get_ipc_provider(&self.global)?;

        // If custom gateway addresses are provided, we need to create the subnet directly
        // using the low-level manager interface to bypass the automatic gateway selection
        if let (Some(gateway_addr), Some(registry_addr)) =
            (custom_gateway_addr, custom_registry_addr)
        {
            log::info!("Creating subnet with custom gateway addresses");

            // Broadcast validators step for custom gateway path
            super::super::api::deployment::broadcast_progress(
                state,
                deployment_id,
                "validators",
                85,
                "in_progress",
                Some("Initializing validators...".to_string()),
            )
            .await;

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

            // Auto-approve the subnet
            // log::info!("Auto-approving subnet: {}", subnet_id);
            // provider.approve_subnet(subnet_id.clone(), from).await?;
            // log::info!("Successfully auto-approved subnet: {}", subnet_id);

            // Create custom subnet with the specified gateway addresses
            let addr = self
                .create_subnet_with_custom_gateway(
                    &mut provider,
                    from,
                    parent,
                    subnet_config.min_validators,
                    f64_to_token_amount(subnet_config.min_validator_stake)?,
                    subnet_config.bottomup_check_period,
                    subnet_config
                        .active_validators_limit
                        .unwrap_or(DEFAULT_ACTIVE_VALIDATORS),
                    f64_to_token_amount(subnet_config.min_cross_msg_fee)?,
                    subnet_config.permission_mode,
                    supply_source,
                    collateral_source,
                    validator_gater,
                    validator_rewarder,
                    subnet_config.genesis_subnet_ipc_contracts_owner,
                    subnet_config.chain_id,
                    gateway_addr,
                    registry_addr,
                    &config,
                    headers,
                )
                .await?;

            log::info!("Subnet actor created with address: {}", addr);

            // Convert the subnet actor address to a subnet ID
            let parent_id = SubnetID::from_str(parent_network)?;
            let subnet_id = SubnetID::new_from_parent(&parent_id, addr);

            log::info!("Generated subnet ID: {}", subnet_id);
            log::info!(
                "Federated power setting was handled during subnet creation with custom gateway"
            );

            // Broadcast activation step for custom gateway path
            super::super::api::deployment::broadcast_progress(
                state,
                deployment_id,
                "activation",
                95,
                "in_progress",
                Some("Activating subnet...".to_string()),
            )
            .await;

            // Add subnet to IPC config
            let rpc_url = headers
                .get("x-network-rpc-url")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| anyhow::anyhow!("Missing required header: X-Network-RPC-URL"))?;

            log::info!(
                "Adding subnet {} to IPC config with parent RPC URL: {}",
                subnet_id,
                rpc_url
            );

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
                status: "active".to_string(),
                message: None,
            };

            log::info!("Subnet deployment completed successfully: {:?}", result);
            Ok(result)
        } else {
            // Fall back to the original method if no custom gateway addresses are provided
            // Broadcast validators step before subnet creation
            super::super::api::deployment::broadcast_progress(
                state,
                deployment_id,
                "validators",
                85,
                "in_progress",
                Some("Initializing validators...".to_string()),
            )
            .await;

            let result = self.deploy_subnet(config, headers).await?;

            // Broadcast activation step after subnet creation
            super::super::api::deployment::broadcast_progress(
                state,
                deployment_id,
                "activation",
                95,
                "in_progress",
                Some("Activating subnet...".to_string()),
            )
            .await;

            Ok(result)
        }
    }

    /// Create a subnet with custom gateway addresses, bypassing the automatic gateway selection
    #[allow(clippy::too_many_arguments)]
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
        subnet_chain_id: u64,
        custom_gateway_addr: ethers::types::Address,
        custom_registry_addr: ethers::types::Address,
        config: &serde_json::Value,
        headers: &warp::http::HeaderMap,
    ) -> Result<fvm_shared::address::Address> {
        // For now, let's use a simplified approach and modify the IPC config temporarily
        // to use the custom gateway, then create the subnet normally

        log::info!(
            "Creating subnet with custom gateway: 0x{:x}",
            custom_gateway_addr
        );
        log::info!(
            "Creating subnet with custom registry: 0x{:x}",
            custom_registry_addr
        );

        // Convert the custom addresses to Filecoin format
        let custom_gateway_fil = ethers_address_to_fil_address(&custom_gateway_addr)?;
        let custom_registry_fil = ethers_address_to_fil_address(&custom_registry_addr)?;

        log::info!("Custom gateway Filecoin address: {}", custom_gateway_fil);
        log::info!("Custom registry Filecoin address: {}", custom_registry_fil);
        log::info!("From address: {:?}", from);

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
        let original_config = ipc_config_store
            .snapshot()
            .await
            .subnets
            .get(&temp_subnet_id)
            .cloned();

        // Add our temporary configuration with custom gateway
        ipc_config_store
            .add_subnet(
                temp_subnet_id.clone(),
                rpc_url_parsed,
                custom_gateway_fil,
                custom_registry_fil,
            )
            .await?;

        log::info!("Temporarily added custom gateway config for subnet creation");

        // Get a fresh provider instance AFTER config modification
        // This ensures the provider has the correct custom gateway context
        let mut provider = get_ipc_provider(&self.global)?;

        // Now create the subnet using the standard method (which will pick up our custom gateway)
        let result = provider
            .create_subnet(
                from,
                parent.clone(), // Clone parent since we need to use it later
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
                subnet_chain_id,
                None, // genesis_f3_instance_id - not provided from UI
            )
            .await;

        // Handle federated power setting BEFORE restoring configuration
        // This ensures we use the correct provider context with custom gateway
        if let Ok(subnet_actor_addr) = &result {
            // Convert the subnet actor address to a subnet ID
            let subnet_id =
                ipc_api::subnet_id::SubnetID::new_from_parent(&parent, *subnet_actor_addr);

            log::info!("Auto-approving subnet: {}", subnet_id);
            provider.approve_subnet(subnet_id.clone(), from).await?;
            log::info!("Successfully auto-approved subnet: {}", subnet_id);

            if permission_mode == ipc_api::subnet::PermissionMode::Federated
                || permission_mode == ipc_api::subnet::PermissionMode::Static
            {
                log::info!(
                    "Setting federated power for {:?} subnet before config restoration",
                    permission_mode
                );

                log::info!("🔍 Subnet actor address: {}", subnet_actor_addr);
                log::info!("🔍 Subnet ID: {}", subnet_id);
                log::info!("🔍 Parent subnet: {}", parent);

                // Log provider configuration
                log::info!("🔍 Provider configuration before set_federated_power:");
                log::info!("  - Custom gateway address: 0x{:x}", custom_gateway_addr);
                log::info!("  - Custom registry address: 0x{:x}", custom_registry_addr);

                // Extract validator configuration from UI config
                let validator_pubkeys = config["validatorPubkeys"]
                    .as_array()
                    .and_then(|arr| arr.iter().map(|v| v.as_str()).collect::<Option<Vec<_>>>())
                    .unwrap_or_default();

                let validator_power = config["validatorPower"]
                    .as_array()
                    .and_then(|arr| arr.iter().map(|v| v.as_u64()).collect::<Option<Vec<_>>>())
                    .unwrap_or_else(|| vec![1; validator_pubkeys.len()]);

                if !validator_pubkeys.is_empty() {
                    log::info!(
                        "Found {} validators with pubkeys: {:?}",
                        validator_pubkeys.len(),
                        validator_pubkeys
                    );
                    log::info!("Validator powers: {:?}", validator_power);

                    // Convert public keys to validator addresses
                    let validator_addresses: Vec<String> = validator_pubkeys
                        .iter()
                        .map(|pubkey| {
                            // Convert public key to Ethereum address
                            use ethers::{types::H160, utils::keccak256};
                            use hex::FromHex;

                            let key = pubkey
                                .strip_prefix("0x")
                                .or_else(|| pubkey.strip_prefix("0X"))
                                .unwrap_or(pubkey);
                            let bytes = Vec::from_hex(key)
                                .map_err(|e| anyhow::anyhow!("Invalid public key hex: {}", e))?;

                            if bytes.len() != 65 || bytes[0] != 0x04 {
                                return Err(anyhow::anyhow!(
                                    "Expected 65-byte uncompressed key starting with 0x04"
                                ));
                            }
                            let hash = keccak256(&bytes[1..]);
                            let eth_addr = H160::from_slice(&hash[12..]);
                            Ok(format!("{:#x}", eth_addr))
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    log::info!("Computed validator addresses: {:?}", validator_addresses);

                    // Prepare validator pubkeys without 0x prefix for the set_federated_power call
                    let pubkeys_clean: Vec<String> = validator_pubkeys
                        .iter()
                        .map(|pk| {
                            pk.strip_prefix("0x")
                                .or_else(|| pk.strip_prefix("0X"))
                                .unwrap_or(pk)
                                .to_string()
                        })
                        .collect();

                    // Convert power values to u128
                    let validator_power_u128: Vec<u128> =
                        validator_power.iter().map(|&p| p as u128).collect();

                    // Get from address string
                    let from_address_str = from
                        .map(|addr| addr.to_string())
                        .or_else(|| config["from"].as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| {
                            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_string()
                        });

                    log::info!("🔍 From address configuration:");
                    log::info!("  - from parameter: {:?}", from);
                    log::info!("  - config['from']: {:?}", config["from"]);
                    log::info!("  - Final from_address_str: {}", from_address_str);

                    // The critical issue: WHO is the owner of the subnet contract?
                    log::info!("🔍 Subnet ownership check:");
                    log::info!(
                        "  - genesis_subnet_ipc_contracts_owner: 0x{:x}",
                        genesis_subnet_ipc_contracts_owner
                    );
                    log::info!(
                        "  - from address (calling set_federated_power): {}",
                        from_address_str
                    );
                    log::warn!("⚠️  IMPORTANT: Only the subnet owner can call setFederatedPower!");

                    // Check if they match
                    if from_address_str.to_lowercase()
                        != format!("0x{:x}", genesis_subnet_ipc_contracts_owner).to_lowercase()
                    {
                        log::error!("❌ OWNERSHIP MISMATCH: The 'from' address {} does not match the subnet owner 0x{:x}",
                                   from_address_str, genesis_subnet_ipc_contracts_owner);
                        log::error!(
                            "This will cause NotAuthorized error when calling setFederatedPower!"
                        );
                    } else {
                        log::info!("✅ Ownership match: from address is the subnet owner");
                    }

                    // Log validator count before creating the args
                    log::info!(
                        "Calling set_federated_power with {} validators",
                        validator_addresses.len()
                    );

                    // CRITICAL FIX: Use the subnet owner's address, not the from address
                    // The setFederatedPower function can only be called by the subnet owner
                    let owner_address_str = format!("0x{:x}", genesis_subnet_ipc_contracts_owner);
                    log::info!(
                        "🔍 Using subnet owner address for set_federated_power: {}",
                        owner_address_str
                    );

                    // Create SetFederatedPowerArgs
                    let set_power_args =
                        crate::commands::subnet::set_federated_power::SetFederatedPowerArgs {
                            from: from_address_str.clone(),
                            subnet: subnet_id.to_string(),
                            validator_addresses: validator_addresses.clone(),
                            validator_pubkeys: pubkeys_clean.clone(),
                            validator_power: validator_power_u128.clone(),
                        };

                    log::info!("🔍 SetFederatedPowerArgs:");
                    log::info!("  - from: {}", set_power_args.from);
                    log::info!("  - subnet: {}", set_power_args.subnet);
                    log::info!(
                        "  - validator_addresses: {:?}",
                        set_power_args.validator_addresses
                    );
                    log::info!(
                        "  - validator_pubkeys: {:?}",
                        set_power_args.validator_pubkeys
                    );
                    log::info!("  - validator_power: {:?}", set_power_args.validator_power);

                    // Call set_federated_power using the provider with custom gateway config
                    log::info!("🔍 Calling set_federated_power function...");
                    match crate::commands::subnet::set_federated_power::set_federated_power(
                        &provider,
                        &set_power_args,
                    )
                    .await
                    {
                        Ok(chain_epoch) => {
                            log::info!("✅ Successfully set federated power at epoch {} (before config restoration)", chain_epoch);
                        }
                        Err(e) => {
                            log::error!("❌ CRITICAL: Failed to set federated power: {}", e);
                            log::error!("This means the subnet was created but validators were NOT configured!");
                            log::error!(
                                "The subnet will need manual intervention to set federated power."
                            );
                            // Return the error so the user knows deployment failed
                            return Err(anyhow::anyhow!("Failed to set federated power: {}. Subnet created but not fully configured.", e));
                        }
                    }
                } else {
                    log::warn!("No validator public keys provided for federated subnet - skipping federated power setting");
                }
            }
        }

        // Restore the original configuration if it existed, or remove our temporary config
        if let Some(original) = original_config {
            // Restore original config
            log::info!(
                "Restoring original config for parent subnet: {}",
                temp_subnet_id
            );
            match &original.config {
                SubnetConfig::Fevm(evm_subnet) => {
                    ipc_config_store
                        .add_subnet(
                            temp_subnet_id.clone(),
                            evm_subnet.provider_http.clone(),
                            evm_subnet.gateway_addr,
                            evm_subnet.registry_addr,
                        )
                        .await?;
                    log::info!("Successfully restored original config");
                }
            }
        } else {
            // The parent subnet didn't exist before, but we can't easily remove it
            // without affecting other operations. For now, leave it as is.
            // This is safe because we only added the parent, not the new subnet
            log::info!("Parent subnet config added temporarily - leaving in place (safe)");
        }

        result
    }
}

/// Convert an Ethereum address to Filecoin address format
fn ethers_address_to_fil_address(
    eth_addr: &ethers::types::Address,
) -> Result<fvm_shared::address::Address> {
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
        token_address: conf
            .supply_source_address
            .clone()
            .map(|addr| fvm_shared::address::Address::from_str(&addr))
            .transpose()?,
    })
}

/// Parse collateral source configuration
fn parse_collateral_source(conf: &SubnetCreateConfig) -> Result<Asset> {
    let kind = conf.collateral_source_kind.unwrap_or(AssetKind::Native);
    Ok(Asset {
        kind,
        token_address: conf
            .collateral_source_address
            .clone()
            .map(|addr| fvm_shared::address::Address::from_str(&addr))
            .transpose()?,
    })
}
