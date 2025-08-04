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
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    log::info!("Received deployment request: {:?}", request);

    let global = crate::GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = DeploymentService::new(global);

    match service.deploy_subnet(request.config.clone()).await {
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
    _state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    log::info!("Received get templates request");

    // Return available deployment templates
    let templates = vec![
        serde_json::json!({
            "id": "basic-subnet",
            "name": "Basic Subnet",
            "description": "A simple subnet configuration with default settings",
            "category": "basic",
            "config": {
                "min_validator_stake": 1.0,
                "min_validators": 1,
                "bottomup_check_period": 100,
                "permission_mode": "Collateral",
                "supply_source_kind": "Native"
            }
        }),
        serde_json::json!({
            "id": "advanced-subnet",
            "name": "Advanced Subnet",
            "description": "A subnet with advanced configuration options",
            "category": "advanced",
            "config": {
                "min_validator_stake": 10.0,
                "min_validators": 3,
                "bottomup_check_period": 50,
                "permission_mode": "Federated",
                "supply_source_kind": "ERC20"
            }
        }),
        serde_json::json!({
            "id": "enterprise-subnet",
            "name": "Enterprise Subnet",
            "description": "A high-performance subnet for enterprise use",
            "category": "enterprise",
            "config": {
                "min_validator_stake": 100.0,
                "min_validators": 5,
                "bottomup_check_period": 30,
                "permission_mode": "Collateral",
                "supply_source_kind": "Native"
            }
        })
    ];

    let response = super::types::ApiResponse::success(templates);
    Ok(warp::reply::json(&response))
}