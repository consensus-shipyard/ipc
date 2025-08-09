// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Subnet API endpoints

use super::types::{ApiResponse, InvalidRequest, ServerError};
use super::super::AppState;
use super::super::services::SubnetService;
use crate::GlobalArguments;
use anyhow::Result;
use serde_json;
use std::collections::HashMap;
use std::convert::Infallible;
use warp::{Filter, Reply};

/// Create subnet API routes
pub fn subnet_routes(
    state: AppState,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let approve_route = warp::path!("subnets" / String / "approve")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_approve_subnet);

    let add_validator_route = warp::path!("subnets" / String / "validators")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_add_validator);

    let remove_validator_route = warp::path!("subnets" / String / "validators" / "remove")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_remove_validator);

    let update_stake_route = warp::path!("subnets" / String / "validators" / "stake")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_update_validator_stake);

    let set_power_route = warp::path!("subnets" / String / "power")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_set_federated_power);

    let stats_route = warp::path!("subnets" / String / "stats")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_subnet_stats);

    let status_route = warp::path!("subnets" / String / "status")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_subnet_status);

    let instances_route = warp::path("instances")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_get_instances);

    let instance_route = warp::path("instance")
        .and(warp::get())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_state(state.clone()))
        .and_then(handle_get_instance);

    approve_route
        .or(add_validator_route)
        .or(remove_validator_route)
        .or(update_stake_route)
        .or(set_power_route)
        .or(stats_route)
        .or(status_route)
        .or(instances_route)
        .or(instance_route)
}

/// Helper to pass state to handlers
fn with_state(
    state: AppState,
) -> impl Filter<Extract = (AppState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Handle subnet approval request
async fn handle_approve_subnet(
    subnet_id: String,
    approval_data: serde_json::Value,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    let from_address = approval_data.get("from")
        .and_then(|v| v.as_str())
        .ok_or_else(|| warp::reject::custom(InvalidRequest("from address required".to_string())))?;

    match service.approve_subnet(&subnet_id, from_address).await {
        Ok(message) => Ok(warp::reply::json(&ApiResponse::success(message))),
        Err(e) => {
            log::error!("Subnet approval failed: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle add validator request
async fn handle_add_validator(
    subnet_id: String,
    validator_data: serde_json::Value,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    match service.add_validator(&subnet_id, &validator_data).await {
        Ok(message) => Ok(warp::reply::json(&ApiResponse::success(message))),
        Err(e) => {
            log::error!("Add validator failed: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle remove validator request
async fn handle_remove_validator(
    subnet_id: String,
    validator_data: serde_json::Value,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    match service.remove_validator(&subnet_id, &validator_data).await {
        Ok(message) => Ok(warp::reply::json(&ApiResponse::success(message))),
        Err(e) => {
            log::error!("Remove validator failed: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle update validator stake request
async fn handle_update_validator_stake(
    subnet_id: String,
    stake_data: serde_json::Value,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    match service.update_validator_stake(&subnet_id, &stake_data).await {
        Ok(message) => Ok(warp::reply::json(&ApiResponse::success(message))),
        Err(e) => {
            log::error!("Update validator stake failed: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle set federated power request
async fn handle_set_federated_power(
    subnet_id: String,
    power_data: serde_json::Value,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    match service.set_federated_power(&subnet_id, &power_data).await {
        Ok(message) => Ok(warp::reply::json(&ApiResponse::success(message))),
        Err(e) => {
            log::error!("Set federated power failed: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle subnet statistics request
async fn handle_subnet_stats(
    subnet_id: String,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    // URL decode the subnet_id parameter
    let decoded_subnet_id = urlencoding::decode(&subnet_id)
        .map_err(|e| warp::reject::custom(ServerError(format!("Invalid subnet ID encoding: {}", e))))?
        .to_string();

    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    match service.get_subnet_stats(&decoded_subnet_id).await {
        Ok(stats) => Ok(warp::reply::json(&ApiResponse::success(stats))),
        Err(e) => {
            log::error!("Get subnet stats failed: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle subnet status request
async fn handle_subnet_status(
    subnet_id: String,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    // URL decode the subnet_id parameter
    let decoded_subnet_id = urlencoding::decode(&subnet_id)
        .map_err(|e| warp::reject::custom(ServerError(format!("Invalid subnet ID encoding: {}", e))))?
        .to_string();

    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    match service.get_subnet_status(&decoded_subnet_id).await {
        Ok(status) => Ok(warp::reply::json(&ApiResponse::success(status))),
        Err(e) => {
            log::error!("Failed to get subnet status: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle get instances request
async fn handle_get_instances(
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    log::info!("Received get instances request");

    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    // Get subnet instances from the configuration
    match service.list_subnets().await {
        Ok(instances) => {
            let response = ApiResponse::success(instances);
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            log::error!("Failed to get instances: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle get instance request
async fn handle_get_instance(
    query_params: HashMap<String, String>,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let instance_id = match query_params.get("id") {
        Some(id) => id,
        None => {
            return Err(warp::reject::custom(InvalidRequest("Missing 'id' parameter".to_string())));
        }
    };

    log::info!("Received get instance request for ID: {}", instance_id);

    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    // Get specific subnet instance
    match service.get_subnet_info(instance_id).await {
        Ok(instance) => {
            let response = ApiResponse::success(instance);
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            log::error!("Failed to get instance {}: {}", instance_id, e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}