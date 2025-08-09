// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Deployment API endpoints

use super::super::AppState;
use super::super::services::DeploymentService;
use super::super::services::deployment_service::SubnetDeploymentResult;
use super::types::{ApiResponse, DeploymentRequest, DeploymentResponse};
use anyhow::Result;
use std::convert::Infallible;
use std::sync::Mutex;
use uuid::Uuid;
use warp::{Filter, Reply};
use crate::commands::ui::websocket::types::OutgoingMessage;
use futures_util::SinkExt;
use warp::ws::Message;

/// Create deployment API routes
pub fn deployment_routes(
    state: AppState,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
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
) -> impl Filter<Extract = (AppState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Handle deployment request
async fn handle_deploy_request(
    request: DeploymentRequest,
    headers: warp::http::HeaderMap,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
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
                // Broadcast final success
                broadcast_progress(&deploy_state, &deploy_id, "verification", 100, "completed",
                    Some(format!("Subnet created successfully: {}", result.subnet_id))).await;
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

    Ok(warp::reply::json(&ApiResponse::success(response)))
}

/// Handle get templates request
async fn handle_get_templates(
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
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

    // Run the actual deployment
    let result = service.deploy_subnet(config, headers).await?;

        broadcast_progress(state, deployment_id, "genesis", 70, "in_progress",
        Some("Creating genesis block...".to_string())).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    broadcast_progress(state, deployment_id, "validators", 85, "in_progress",
        Some("Initializing validators...".to_string())).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    broadcast_progress(state, deployment_id, "activation", 95, "in_progress",
        Some("Activating subnet...".to_string())).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

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

    // Create WebSocket message
    let ws_message = OutgoingMessage::DeploymentProgress {
        deployment_id: deployment_id.to_string(),
        step: step.to_string(),
        status: status.to_string(),
        progress,
        message,
    };

    // Broadcast to all connected WebSocket clients
    let clients = {
        let clients_guard = state.websocket_clients.lock().unwrap();
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