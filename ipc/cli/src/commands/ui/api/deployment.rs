// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Deployment API endpoints

use super::super::AppState;
use super::super::services::{DeploymentService};
use super::types::{ApiResponse, DeploymentRequest, DeploymentResponse};
use anyhow::Result;
use std::convert::Infallible;
use std::sync::Mutex;
use uuid::Uuid;
use warp::{Filter, Reply};

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

    match service.deploy_subnet(request.config.clone(), &headers).await {
        Ok(result) => {
            let deployment_id = uuid::Uuid::new_v4().to_string();
            let response = DeploymentResponse {
                deployment_id: deployment_id.clone(),
                status: "completed".to_string(),
                message: format!("Subnet created successfully: {}", result.subnet_id),
            };

            // Store deployment state
            {
                let mut deployments = state.deployments.lock().unwrap();
                deployments.insert(deployment_id, super::super::DeploymentState {
                    id: response.deployment_id.clone(),
                    template: request.template,
                    status: "completed".to_string(),
                    created_at: chrono::Utc::now(),
                    config: request.config,
                    progress: 100,
                    step: "completed".to_string(),
                    updated_at: chrono::Utc::now(),
                });
            }

            Ok(warp::reply::json(&ApiResponse::success(response)))
        }
        Err(e) => {
            log::error!("Deployment failed: {}", e);
            let response = DeploymentResponse {
                deployment_id: uuid::Uuid::new_v4().to_string(),
                status: "failed".to_string(),
                message: format!("Deployment failed: {}", e),
            };
            Ok(warp::reply::json(&ApiResponse::success(response)))
        }
    }
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