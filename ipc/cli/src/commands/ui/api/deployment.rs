// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Deployment API endpoints

use super::super::services::deployment_service::{DeploymentService, SubnetDeploymentResult, ContractDeploymentProgress};
use super::types::{ApiResponse, DeploymentRequest, DeploymentResponse};
use super::super::{AppState, DeploymentState};
use anyhow::{Result, Context};
use chrono;
use futures_util::SinkExt;
use serde_json;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{self, Filter, Reply};
use warp::ws::Message;
use uuid::Uuid;
use std::str::FromStr;
use ethers::types::Address;
use crate::commands::deploy::{DeployConfig, CliSubnetCreationPrivilege};
use url;
use ipc_types;
use fvm_shared::address::Address as FilecoinAddress;

/// Create deployment API routes
pub fn deployment_routes(
    state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let deploy_route = warp::path("deploy")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::header::headers_cloned())
        .and(with_state(state.clone()))
        .and_then(handle_deploy_request);

    let templates_route = warp::path("templates")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_get_templates);

    deploy_route.or(templates_route)
}

/// Helper to pass state to handlers
fn with_state(
    state: AppState,
) -> impl Filter<Extract = (AppState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Handle deployment request
async fn handle_deploy_request(
    request: DeploymentRequest,
    headers: warp::http::HeaderMap,
    state: AppState,
) -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("Received deployment request: {:?}", request);
    log::debug!("Request headers: {:?}", headers);

    let global = crate::GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = DeploymentService::new(global);

    // Generate deployment ID and start asynchronous deployment
    let deployment_id = uuid::Uuid::new_v4().to_string();
    log::info!("Generated deployment ID: {}", deployment_id);

    // Store initial deployment state
    {
        let mut deployments = state.deployments.lock().unwrap();
        deployments.insert(deployment_id.clone(), super::super::DeploymentState {
            id: deployment_id.clone(),
            template: request.template.clone(),
            status: "in_progress".to_string(),
            created_at: chrono::Utc::now(),
            config: request.config.clone(),
            progress: 0,
            step: "validate".to_string(),
            updated_at: chrono::Utc::now(),
        });
    }

    // Start async deployment task
    let deployment_config = request.config.clone();
    let deploy_headers = headers.clone();
    let deploy_state = state.clone();
    let deploy_id = deployment_id.clone();

    tokio::spawn(async move {
        // Run the actual deployment in background and send progress updates
        match run_async_deployment(&service, deployment_config, &deploy_headers, &deploy_state, &deploy_id).await {
            Ok(result) => {
                // Broadcast final success with subnet_id and contract addresses
                broadcast_progress_with_deployment_result(&deploy_state, &deploy_id, "verification", 100, "completed",
                    Some(format!("Subnet created successfully: {}", result.subnet_id)), &result).await;
            }
            Err(e) => {
                // Broadcast failure
                broadcast_progress(&deploy_state, &deploy_id, "error", 0, "failed",
                    Some(format!("Deployment failed: {}", e))).await;
            }
        }
    });

    // Return immediate response with deployment_id
    let response = DeploymentResponse {
        deployment_id: deployment_id.clone(),
        status: "in_progress".to_string(),
        message: "Deployment started successfully".to_string(),
    };

    log::info!("Deployment response created: {:?}", response);
    let api_response = ApiResponse::success(response);
    log::info!("About to return successful API response with deployment_id: {}", deployment_id);

    Ok(warp::reply::json(&api_response))
}

/// Handle get templates request
async fn handle_get_templates(
    state: AppState,
) -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("Received get templates request");

    let global = crate::GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = DeploymentService::new(global);

    match service.get_templates().await {
        Ok(templates) => {
            let response = ApiResponse::success(templates);
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            log::error!("Failed to get templates: {}", e);
            let response = ApiResponse::<Vec<serde_json::Value>>::error(format!("Failed to get templates: {}", e));
            Ok(warp::reply::json(&response))
        }
    }
}

/// Run deployment asynchronously with progress updates
async fn run_async_deployment(
    service: &DeploymentService,
    config: serde_json::Value,
    headers: &warp::http::HeaderMap,
    state: &AppState,
    deployment_id: &str,
) -> Result<SubnetDeploymentResult, anyhow::Error> {
    // Send progress updates for each step
    broadcast_progress(state, deployment_id, "validate", 10, "in_progress",
        Some("Validating configuration...".to_string())).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    broadcast_progress(state, deployment_id, "prepare", 20, "in_progress",
        Some("Preparing deployment files...".to_string())).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Check gateway mode to determine if we need to deploy contracts
    log::info!("=== GATEWAY MODE DEBUGGING ===");
    log::info!("Full config received: {}", serde_json::to_string_pretty(&config).unwrap_or_else(|_| "Failed to serialize config".to_string()));

    let gateway_mode = config.get("gatewayMode").and_then(|v| v.as_str()).unwrap_or("deploy");
    log::info!("Extracted gateway mode: '{}'", gateway_mode);
    log::info!("Gateway mode type: {:?}", config.get("gatewayMode"));

    if gateway_mode == "deployed" || gateway_mode == "l1-gateway" {
        log::info!("SHOULD SKIP CONTRACT DEPLOYMENT - Using {} mode", gateway_mode);
    } else {
        log::info!("WILL DEPLOY CONTRACTS - Gateway mode is '{}'", gateway_mode);
    }

    // Extract network configuration needed for all modes
    let rpc_url = headers
        .get("x-network-rpc-url")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing required header: X-Network-RPC-URL"))?;

    let chain_id = headers
        .get("x-network-chain-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid header: X-Network-Chain-ID"))?;

    let deployed_contracts = match gateway_mode {
        "deployed" => {
            // Use existing deployed gateway - get gateway info and skip contract deployment
            broadcast_progress(state, deployment_id, "contracts", 50, "in_progress",
                Some("Using existing deployed gateway...".to_string())).await;

            let selected_gateway_id = config.get("selectedDeployedGateway")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Selected deployed gateway ID is required when using deployed gateway mode"))?;

            log::info!("Using deployed gateway: {}", selected_gateway_id);

            // Get gateway information from the gateway service
            let gateway_service = super::super::services::GatewayService::new(crate::GlobalArguments {
                config_path: Some(state.config_path.clone()),
                _network: fvm_shared::address::Network::Testnet,
                __network: None,
            });

            let discovered_gateways = gateway_service.discover_gateways(Some(headers)).await
                .map_err(|e| anyhow::anyhow!("Failed to discover gateways: {}", e))?;

            let selected_gateway = discovered_gateways.iter()
                .find(|g| g.id == selected_gateway_id)
                .ok_or_else(|| anyhow::anyhow!("Selected gateway not found: {}", selected_gateway_id))?;

            log::info!("Found selected gateway: {} at address {}", selected_gateway.id, selected_gateway.address);

            // Convert Filecoin addresses to Ethereum format
            // Parse the Filecoin addresses first
            let gateway_fil_addr = FilecoinAddress::from_str(&selected_gateway.address)
                .map_err(|e| anyhow::anyhow!("Invalid Filecoin gateway address '{}': {}", selected_gateway.address, e))?;
            let registry_fil_addr = FilecoinAddress::from_str(&selected_gateway.registry_address)
                .map_err(|e| anyhow::anyhow!("Invalid Filecoin registry address '{}': {}", selected_gateway.registry_address, e))?;

            // Convert to Ethereum format
            let gateway_addr = ipc_api::evm::payload_to_evm_address(gateway_fil_addr.payload())
                .map_err(|e| anyhow::anyhow!("Failed to convert gateway address to Ethereum format: {}", e))?;
            let registry_addr = ipc_api::evm::payload_to_evm_address(registry_fil_addr.payload())
                .map_err(|e| anyhow::anyhow!("Failed to convert registry address to Ethereum format: {}", e))?;

            log::info!("Converted gateway address from {} to 0x{:x}", selected_gateway.address, gateway_addr);
            log::info!("Converted registry address from {} to 0x{:x}", selected_gateway.registry_address, registry_addr);

            fendermint_eth_deployer::DeployedContracts {
                gateway: gateway_addr,
                registry: registry_addr,
            }
        },
        "l1-gateway" => {
            // Use L1 gateway selected from the top menu
            broadcast_progress(state, deployment_id, "contracts", 50, "in_progress",
                Some("Using selected L1 gateway...".to_string())).await;

            let selected_gateway_id = config.get("selectedL1Gateway")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Selected L1 gateway ID is required when using l1-gateway mode"))?;

            log::info!("Using L1 gateway: {}", selected_gateway_id);

            // Get gateway information from the gateway service
            let gateway_service = super::super::services::GatewayService::new(crate::GlobalArguments {
                config_path: Some(state.config_path.clone()),
                _network: fvm_shared::address::Network::Testnet,
                __network: None,
            });

            let discovered_gateways = gateway_service.discover_gateways(Some(headers)).await
                .map_err(|e| anyhow::anyhow!("Failed to discover gateways: {}", e))?;

            let selected_gateway = discovered_gateways.iter()
                .find(|g| g.id == selected_gateway_id)
                .ok_or_else(|| anyhow::anyhow!("Selected L1 gateway not found: {}", selected_gateway_id))?;

            log::info!("Found selected L1 gateway: {} at address {}", selected_gateway.id, selected_gateway.address);

            // Convert Filecoin addresses to Ethereum format
            let gateway_fil_addr = FilecoinAddress::from_str(&selected_gateway.address)
                .map_err(|e| anyhow::anyhow!("Invalid Filecoin gateway address '{}': {}", selected_gateway.address, e))?;
            let registry_fil_addr = FilecoinAddress::from_str(&selected_gateway.registry_address)
                .map_err(|e| anyhow::anyhow!("Invalid Filecoin registry address '{}': {}", selected_gateway.registry_address, e))?;

            // Convert to Ethereum format
            let gateway_addr = ipc_api::evm::payload_to_evm_address(gateway_fil_addr.payload())
                .map_err(|e| anyhow::anyhow!("Failed to convert gateway address to Ethereum format: {}", e))?;
            let registry_addr = ipc_api::evm::payload_to_evm_address(registry_fil_addr.payload())
                .map_err(|e| anyhow::anyhow!("Failed to convert registry address to Ethereum format: {}", e))?;

            log::info!("Converted gateway address from {} to 0x{:x}", selected_gateway.address, gateway_addr);
            log::info!("Converted registry address from {} to 0x{:x}", selected_gateway.registry_address, registry_addr);

            fendermint_eth_deployer::DeployedContracts {
                gateway: gateway_addr,
                registry: registry_addr,
            }
        },
        "custom" => {
            // Use custom gateway addresses provided by user
            broadcast_progress(state, deployment_id, "contracts", 50, "in_progress",
                Some("Using custom gateway addresses...".to_string())).await;

            let gateway_address = config.get("customGatewayAddress")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Custom gateway address is required when using custom gateway mode"))?;

            let registry_address = config.get("customRegistryAddress")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Custom registry address is required when using custom gateway mode"))?;

            let gateway_addr = ethers::types::Address::from_str(gateway_address)
                .map_err(|e| anyhow::anyhow!("Invalid custom gateway address: {}", e))?;
            let registry_addr = ethers::types::Address::from_str(registry_address)
                .map_err(|e| anyhow::anyhow!("Invalid custom registry address: {}", e))?;

            fendermint_eth_deployer::DeployedContracts {
                gateway: gateway_addr,
                registry: registry_addr,
            }
        },
        _ => {
            // Deploy new gateway contracts (default behavior)
            broadcast_progress(state, deployment_id, "contracts", 30, "in_progress",
                Some("Deploying smart contracts...".to_string())).await;

            // Use real contract deployment progress tracking
            let state_clone = state.clone();
            let deployment_id_clone = deployment_id.to_string();

            // Create a progress callback that converts real deployment progress to UI updates
            let progress_callback = move |contract_name: &str, contract_type: &str, current_step: usize, total_steps: usize| {
                let state = state_clone.clone();
                let deployment_id = deployment_id_clone.clone();
                let contract_name = contract_name.to_string();
                let contract_type = contract_type.to_string();

                tokio::spawn(async move {
                    let progress_percent = ((current_step + 1) as f32 / total_steps as f32 * 40.0) + 30.0; // Scale to 30-70%

                    let contract_progress = ContractDeploymentProgress {
                        total_contracts: total_steps as u32,
                        completed_contracts: current_step as u32,
                        current_contract: Some(contract_name.clone()),
                        contracts: vec![], // We'll populate this with actual contract statuses
                    };

                    broadcast_progress_with_contracts(
                        &state,
                        &deployment_id,
                        "contracts",
                        progress_percent as u8,
                        if current_step + 1 == total_steps { "completed" } else { "in_progress" },
                        Some(format!("Deploying {} ({}/{})", contract_name, current_step + 1, total_steps)),
                        Some(contract_progress),
                    ).await;
                });
            };

            log::info!("=== DEBUGGING NETWORK CONFIGURATION ===");
            log::info!("Available headers: {:?}", headers.keys().collect::<Vec<_>>());
            log::info!("Config object: {}", serde_json::to_string_pretty(&config).unwrap_or_else(|_| "Failed to serialize config".to_string()));

            log::info!("Found RPC URL from header: {}", rpc_url);
            log::info!("Found Chain ID from header: {}", chain_id);

            // For deployment address, check config first, then fall back to a default
            let from_address_str = config["deployment"]["fromAddress"]
                .as_str()
                .or_else(|| config["from"].as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing required field: deployment.fromAddress or from"))?;

            let from_address = ethers::types::Address::from_str(from_address_str)
                .map_err(|e| anyhow::anyhow!("Invalid fromAddress '{}': {}", from_address_str, e))?;

            // Validate RPC URL format
            if rpc_url.is_empty() {
                return Err(anyhow::anyhow!("RPC URL cannot be empty"));
            }

            log::info!("Deploying contracts to RPC URL: {}, from address: {}, chain ID: {}",
                       rpc_url, from_address, chain_id);

            let deploy_config = DeployConfig {
                url: rpc_url.to_string(),
                from: from_address,
                chain_id,
                artifacts_path: None,
                subnet_creation_privilege: CliSubnetCreationPrivilege::Unrestricted,
            };

            // Deploy contracts with real progress tracking directly (without spawn)
            service.deploy_contracts_with_real_progress(&deploy_config, progress_callback).await?
        }
    };

    // Register the deployed contracts in the IPC configuration store (only for newly deployed contracts)
    if gateway_mode != "deployed" && gateway_mode != "custom" && gateway_mode != "l1-gateway" {
        log::info!("Registering newly deployed contracts in IPC config: gateway={:?}, registry={:?}",
                   deployed_contracts.gateway, deployed_contracts.registry);

        let ipc_config_store = service.get_config_store().await?;
        let subnet_id = ipc_api::subnet_id::SubnetID::new_root(chain_id);
        let rpc_url_parsed: url::Url = rpc_url.parse().context("invalid RPC URL")?;

        ipc_config_store
            .add_subnet(
                subnet_id.clone(),
                rpc_url_parsed,
                ipc_types::EthAddress::from(deployed_contracts.gateway).into(),
                ipc_types::EthAddress::from(deployed_contracts.registry).into(),
            )
            .await?;

        log::info!("Successfully registered subnet {} in IPC config", subnet_id);
    } else {
        log::info!("Skipping contract registration - using existing gateway (mode: {})", gateway_mode);
    }

    // Continue with the rest of the deployment
    broadcast_progress(state, deployment_id, "genesis", 70, "in_progress",
        Some("Creating genesis block...".to_string())).await;

    // Now continue with the subnet creation part, passing the selected gateway addresses
    let subnet_result = service.deploy_subnet_with_gateway(
        config.clone(),
        headers,
        Some(deployed_contracts.gateway),
        Some(deployed_contracts.registry)
    ).await?;

        // NOTE: Progress updates for validators, activate, and verification steps
    // should be broadcast from within deploy_subnet_with_gateway as they happen,
    // not here after the fact. These broadcasts were misleading.

    Ok(subnet_result)
}

/// Broadcast deployment progress to WebSocket clients
async fn broadcast_progress(
    state: &AppState,
    deployment_id: &str,
    step: &str,
    progress: u8,
    status: &str,
    message: Option<String>,
) {
    // Update deployment state
    {
        let mut deployments = state.deployments.lock().unwrap();
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.step = step.to_string();
            deployment.progress = progress;
            deployment.status = status.to_string();
            deployment.updated_at = chrono::Utc::now();
        }
    }

    // Create message in format frontend expects: { type: "deployment_progress", data: {...} }
    let progress_data = serde_json::json!({
        "deployment_id": deployment_id,
        "step": step,
        "progress": progress,
        "status": status,
        "message": message
    });

    let ws_message = serde_json::json!({
        "type": "deployment_progress",
        "data": progress_data
    });

    // Broadcast to all connected WebSocket clients
    let clients = {
        let clients_guard = state.websocket_clients.lock().unwrap();
        log::info!("Broadcasting progress to {} WebSocket clients", clients_guard.len());
        clients_guard.clone()  // Clone the Arc<Mutex<...>> handles
    };

    for client in clients.iter() {
        let mut sink = client.lock().await;
        if let Ok(json) = serde_json::to_string(&ws_message) {
            if let Err(e) = sink.send(Message::text(json)).await {
                log::error!("Failed to send WebSocket message: {}", e);
            }
        }
    }

    log::info!("Progress: {} - {} ({}%)", deployment_id, step, progress);
}

/// Broadcast deployment progress with subnet_id to WebSocket clients
async fn broadcast_progress_with_subnet_id(
    state: &AppState,
    deployment_id: &str,
    step: &str,
    progress: u8,
    status: &str,
    message: Option<String>,
    subnet_id: Option<String>,
) {
    // Update deployment state
    {
        let mut deployments = state.deployments.lock().unwrap();
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.step = step.to_string();
            deployment.progress = progress;
            deployment.status = status.to_string();
            deployment.updated_at = chrono::Utc::now();
        }
    }

    // Create message in format frontend expects: { type: "deployment_progress", data: {...} }
    let mut progress_data = serde_json::json!({
        "deployment_id": deployment_id,
        "step": step,
        "progress": progress,
        "status": status,
        "message": message
    });

    // Add subnet_id if provided
    if let Some(ref subnet_id_str) = subnet_id {
        progress_data["subnet_id"] = serde_json::Value::String(subnet_id_str.clone());

        // If this is a completion message, try to extract parent_id from subnet_id
        if status == "completed" {
            if let Ok(parsed_subnet_id) = ipc_api::subnet_id::SubnetID::from_str(subnet_id_str) {
                if let Some(parent_subnet) = parsed_subnet_id.parent() {
                    progress_data["parent_id"] = serde_json::Value::String(parent_subnet.to_string());
                }
            }
        }
    }

    let ws_message = serde_json::json!({
        "type": "deployment_progress",
        "data": progress_data
    });

    // Broadcast to all connected WebSocket clients
    let clients = {
        let clients_guard = state.websocket_clients.lock().unwrap();
        log::info!("Broadcasting progress with subnet_id to {} WebSocket clients", clients_guard.len());
        clients_guard.clone()  // Clone the Arc<Mutex<...>> handles
    };

    for client in clients.iter() {
        let mut sink = client.lock().await;
        if let Ok(json) = serde_json::to_string(&ws_message) {
            if let Err(e) = sink.send(Message::text(json)).await {
                log::error!("Failed to send WebSocket message: {}", e);
            }
        }
    }

    log::info!("Progress with subnet_id: {} - {} ({}%)", deployment_id, step, progress);
}

/// Broadcast deployment progress with full deployment result to WebSocket clients
async fn broadcast_progress_with_deployment_result(
    state: &AppState,
    deployment_id: &str,
    step: &str,
    progress: u8,
    status: &str,
    message: Option<String>,
    result: &SubnetDeploymentResult,
) {
    // Update deployment state
    {
        let mut deployments = state.deployments.lock().unwrap();
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.step = step.to_string();
            deployment.progress = progress;
            deployment.status = status.to_string();
            deployment.updated_at = chrono::Utc::now();
        }
    }

    // Create message in format frontend expects: { type: "deployment_progress", data: {...} }
    let mut progress_data = serde_json::json!({
        "deployment_id": deployment_id,
        "step": step,
        "progress": progress,
        "status": status,
        "message": message,
        "subnet_id": result.subnet_id,
        "parent_id": result.parent_id,
        "gateway_address": result.gateway_address,
        "registry_address": result.registry_address
    });

    let ws_message = serde_json::json!({
        "type": "deployment_progress",
        "data": progress_data
    });

    // Broadcast to all connected WebSocket clients
    let clients = {
        let clients_guard = state.websocket_clients.lock().unwrap();
        log::info!("Broadcasting progress with deployment result to {} WebSocket clients", clients_guard.len());
        clients_guard.clone()  // Clone the Arc<Mutex<...>> handles
    };

    for client in clients.iter() {
        let mut sink = client.lock().await;
        if let Ok(json) = serde_json::to_string(&ws_message) {
            if let Err(e) = sink.send(Message::text(json)).await {
                log::error!("Failed to send WebSocket message: {}", e);
            }
        }
    }

    log::info!("Progress with deployment result: {} - {} ({}%)", deployment_id, step, progress);
}

/// Broadcast deployment progress with contract details to WebSocket clients
async fn broadcast_progress_with_contracts(
    state: &AppState,
    deployment_id: &str,
    step: &str,
    progress: u8,
    status: &str,
    message: Option<String>,
    contract_progress: Option<ContractDeploymentProgress>,
) {
    // Update deployment state
    {
        let mut deployments = state.deployments.lock().unwrap();
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.step = step.to_string();
            deployment.progress = progress;
            deployment.status = status.to_string();
            deployment.updated_at = chrono::Utc::now();
        }
    }

    // Create progress data with contract details
    let mut progress_data = serde_json::json!({
        "deployment_id": deployment_id,
        "step": step,
        "progress": progress,
        "status": status,
        "message": message
    });

    // Add contract progress if available
    if let Some(contract_prog) = contract_progress {
        progress_data["contract_progress"] = serde_json::json!({
            "total_contracts": contract_prog.total_contracts,
            "completed_contracts": contract_prog.completed_contracts,
            "current_contract": contract_prog.current_contract,
            "contracts": contract_prog.contracts.iter().map(|c| serde_json::json!({
                "name": c.name,
                "type": c.contract_type,
                "status": c.status,
                "deployedAt": c.deployed_at
            })).collect::<Vec<_>>()
        });
    }

    let ws_message = serde_json::json!({
        "type": "deployment_progress",
        "data": progress_data
    });

    // Broadcast to all connected WebSocket clients
    let clients = {
        let clients_guard = state.websocket_clients.lock().unwrap();
        log::info!("Broadcasting progress with contracts to {} WebSocket clients", clients_guard.len());
        clients_guard.clone()
    };

    for client in clients.iter() {
        let mut sink = client.lock().await;
        if let Ok(json) = serde_json::to_string(&ws_message) {
            if let Err(e) = sink.send(Message::text(json)).await {
                log::error!("Failed to send WebSocket message: {}", e);
            }
        }
    }

    log::info!("Progress: {} - {} ({}%)", deployment_id, step, progress);
}