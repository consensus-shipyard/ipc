//! Deployment service for subnet creation through the UI
//!
//! This service wraps existing CLI handlers for deployment operations.

use super::super::api::types::{ApiResponse, InvalidRequest, ServerError};
use crate::commands::deploy::{deploy_contracts as deploy_contracts_cmd, DeployConfig, CliSubnetCreationPrivilege};
use crate::commands::subnet::create::{CreateSubnet, CreateSubnetArgs};
use crate::commands::subnet::init::config::SubnetCreateConfig;
use crate::{GlobalArguments, get_ipc_provider};
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

        // Get RPC URL and chain ID from headers for deployment config
        let rpc_url = headers.get("x-network-rpc-url")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("http://localhost:8545");

        let chain_id = headers.get("x-network-chain-id")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(31337);

        // Debug: Print the entire UI config to see what we're receiving
        log::info!("UI Config received: {:#?}", ui_config);

        // Always deploy contracts for new networks (like Anvil) that don't have them yet
        let from_str = ui_config.get("from")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0000000000000000000000000000000000000000");
        let from_address = ethers::types::Address::from_str(from_str)
            .unwrap_or(ethers::types::Address::zero());

        // Convert UI config to SubnetCreateConfig - using the correct field names from the UI
        let subnet_config = SubnetCreateConfig {
            from: ui_config.get("from").and_then(|v| v.as_str()).map(|s| s.to_string()),
            parent: parent.to_string(),
            min_validator_stake: ui_config.get("minValidatorStake")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0),
            min_validators: ui_config.get("minValidators")
                .and_then(|v| v.as_u64())
                .unwrap_or(1),
            bottomup_check_period: ui_config.get("bottomupCheckPeriod")
                .and_then(|v| v.as_i64())
                .unwrap_or(100),
            active_validators_limit: ui_config.get("activeValidatorsLimit")
                .and_then(|v| v.as_u64())
                .map(|v| v as u16),
            min_cross_msg_fee: ui_config.get("minCrossMsgFee")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.000001),
            permission_mode: {
                let mode_str = ui_config.get("permissionMode")
                    .and_then(|v| v.as_str());
                log::info!("Permission mode from UI: {:?}", mode_str);

                let permission_mode = mode_str.and_then(|s| match s {
                    "collateral" => Some(PermissionMode::Collateral),
                    "federated" => Some(PermissionMode::Federated),
                    "static" => Some(PermissionMode::Static),
                    _ => None,
                })
                .unwrap_or(PermissionMode::Collateral);
                log::info!("Parsed permission mode: {:?}", permission_mode);
                permission_mode
            },
            supply_source_kind: ui_config.get("supplySourceKind")
                .and_then(|v| v.as_str())
                .and_then(|s| match s {
                    "native" => Some(AssetKind::Native),
                    "erc20" => Some(AssetKind::ERC20),
                    _ => None,
                })
                .unwrap_or(AssetKind::Native),
            supply_source_address: ui_config.get("supplySourceAddress")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            validator_gater: ui_config.get("validatorGater")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            // Add missing required fields with defaults
            collateral_source_kind: ui_config.get("collateralSourceKind")
                .and_then(|v| v.as_str())
                .and_then(|s| match s {
                    "native" => Some(AssetKind::Native),
                    "erc20" => Some(AssetKind::ERC20),
                    _ => None,
                }),
            collateral_source_address: ui_config.get("collateralSourceAddress")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            genesis_subnet_ipc_contracts_owner: {
                let owner_str = ui_config.get("genesisSubnetIpcContractsOwner")
                    .and_then(|v| v.as_str());
                log::info!("Genesis contracts owner from UI: {:?}", owner_str);

                let parsed_address = owner_str
                    .and_then(|s| s.parse::<ethers::types::Address>().ok());
                log::info!("Parsed genesis contracts owner: {:?}", parsed_address);

                parsed_address.unwrap_or_else(|| {
                    log::warn!("Failed to parse genesis contracts owner, using from_address: {:?}", from_address);
                    from_address
                })
            },
            validator_rewarder: None,
        };

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

    /// Deploy contracts with progress tracking
    pub async fn deploy_contracts_with_progress(
        &self,
        deploy_config: &DeployConfig,
        progress_callback: ProgressCallback,
    ) -> Result<DeployedContracts> {
        // Initialize list of all contracts that will be deployed
        let library_contracts = vec![
            "AccountHelper",
            "SubnetIDHelper",
            "CrossMsgHelper",
            "LibQuorum",
        ];

        let mut all_contracts = Vec::new();

        // Add library contracts
        for lib in &library_contracts {
            all_contracts.push(ContractInfo {
                name: lib.to_string(),
                contract_type: "library".to_string(),
                status: "pending".to_string(),
                deployed_at: None,
            });
        }

        // Add gateway
        all_contracts.push(ContractInfo {
            name: "Gateway".to_string(),
            contract_type: "gateway".to_string(),
            status: "pending".to_string(),
            deployed_at: None,
        });

        // Add registry
        all_contracts.push(ContractInfo {
            name: "Registry".to_string(),
            contract_type: "registry".to_string(),
            status: "pending".to_string(),
            deployed_at: None,
        });

        let total_contracts = all_contracts.len() as u32;
        let mut completed_contracts = 0u32;

        // Initial progress update
        let mut progress = ContractDeploymentProgress {
            total_contracts,
            completed_contracts,
            current_contract: Some("Initializing deployment...".to_string()),
            contracts: all_contracts.clone(),
        };
        progress_callback(progress.clone());

        // Simulate individual contract deployment progress
        let deployment_task = {
            let callback = progress_callback.clone();
            let mut progress = progress.clone();

            tokio::spawn(async move {
                // Libraries deployment simulation
                for i in 0..4 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
                    progress.current_contract = Some(format!("Deploying library: {}", library_contracts[i]));
                    progress.contracts[i].status = "deploying".to_string();
                    callback(progress.clone());

                    tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
                    progress.completed_contracts += 1;
                    progress.contracts[i].status = "completed".to_string();
                    progress.contracts[i].deployed_at = Some(chrono::Utc::now().to_rfc3339());
                    callback(progress.clone());
                }

                // Gateway deployment
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                progress.current_contract = Some("Deploying Gateway".to_string());
                progress.contracts[4].status = "deploying".to_string();
                callback(progress.clone());

                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
                progress.completed_contracts += 1;
                progress.contracts[4].status = "completed".to_string();
                progress.contracts[4].deployed_at = Some("Gateway deployed".to_string());
                callback(progress.clone());

                // Registry deployment
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                progress.current_contract = Some("Deploying Registry".to_string());
                progress.contracts[5].status = "deploying".to_string();
                callback(progress.clone());

                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
                progress.completed_contracts += 1;
                progress.contracts[5].status = "completed".to_string();
                progress.contracts[5].deployed_at = Some("Registry deployed".to_string());
                progress.current_contract = None;
                callback(progress.clone());
            })
        };

        // Start the progress simulation
        deployment_task.abort(); // Cancel after starting actual deployment

        // Perform actual deployment
        let keystore = new_evm_keystore_from_config(&self.global.config()?)?;
        let deployed_contracts = deploy_contracts_cmd(keystore, deploy_config).await?;

        // Final progress update
        progress.completed_contracts = total_contracts;
        progress.current_contract = None;
        for contract in &mut progress.contracts {
            contract.status = "completed".to_string();
        }
        progress_callback(progress);

        Ok(deployed_contracts)
    }
}