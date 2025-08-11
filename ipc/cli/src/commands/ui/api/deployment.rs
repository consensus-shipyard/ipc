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

    // Create deploy config - extract network info from headers instead of config
    log::info!("=== DEBUGGING NETWORK CONFIGURATION ===");
    log::info!("Available headers: {:?}", headers.keys().collect::<Vec<_>>());
    log::info!("Config object: {}", serde_json::to_string_pretty(&config).unwrap_or_else(|_| "Failed to serialize config".to_string()));

    let rpc_url = headers
        .get("x-network-rpc-url")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing required header: X-Network-RPC-URL"))?;

    log::info!("Found RPC URL from header: {}", rpc_url);

    let chain_id = headers
        .get("x-network-chain-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid header: X-Network-Chain-ID"))?;

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
    let deployed_contracts = service.deploy_contracts_with_real_progress(&deploy_config, progress_callback).await?;

    // Register the deployed contracts in the IPC configuration store
    log::info!("Registering deployed contracts in IPC config: gateway={:?}, registry={:?}",
               deployed_contracts.gateway, deployed_contracts.registry);

    let ipc_config_store = service.get_config_store().await?;
    let subnet_id = ipc_api::subnet_id::SubnetID::new_root(chain_id);
    let rpc_url: url::Url = rpc_url.parse().context("invalid RPC URL")?;

    ipc_config_store
        .add_subnet(
            subnet_id.clone(),
            rpc_url,
            ipc_types::EthAddress::from(deployed_contracts.gateway).into(),
            ipc_types::EthAddress::from(deployed_contracts.registry).into(),
        )
        .await?;

    log::info!("Successfully registered subnet {} in IPC config", subnet_id);

    // Continue with the rest of the deployment
    broadcast_progress(state, deployment_id, "genesis", 70, "in_progress",
        Some("Creating genesis block...".to_string())).await;

    // Now continue with the subnet creation part (without re-deploying contracts)
    let result = service.deploy_subnet(config.clone(), headers).await?;

    Ok(result)
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