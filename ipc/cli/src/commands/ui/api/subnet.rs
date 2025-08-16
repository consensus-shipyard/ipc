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
use warp::reply;
use warp::hyper::HeaderMap;
use warp::Rejection;
use std::sync::Arc;

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

    let pending_approvals_route = warp::path!("gateways" / String / "pending-approvals")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_list_pending_approvals);

    let node_config_route = warp::path!("subnets" / String / "node-config")
        .and(warp::get())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_state(state.clone()))
        .and_then(handle_generate_node_config);

    let node_commands_route = warp::path!("subnets" / String / "node-commands")
        .and(warp::get())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(with_state(state.clone()))
        .and_then(handle_generate_node_commands);

    approve_route
        .or(add_validator_route)
        .or(remove_validator_route)
        .or(update_stake_route)
        .or(set_power_route)
        .or(stats_route)
        .or(status_route)
        .or(instances_route)
        .or(instance_route)
        .or(pending_approvals_route)
        .or(node_config_route)
        .or(node_commands_route)
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
    log::info!("=== SUBNET APPROVAL REQUEST ===");
    log::info!("Subnet ID (encoded): {}", subnet_id);
    log::info!("Approval data: {}", approval_data);

    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    // URL decode the subnet ID
    let decoded_subnet_id = urlencoding::decode(&subnet_id)
        .map_err(|e| {
            log::error!("Failed to decode subnet ID: {}", e);
            warp::reject::custom(InvalidRequest(format!("Invalid subnet ID encoding: {}", e)))
        })?;

    log::info!("Decoded subnet ID: {}", decoded_subnet_id);

    let from_address = approval_data.get("from")
        .and_then(|v| v.as_str());

    log::info!("From address: {:?}", from_address);
    log::info!("Calling service.approve_subnet...");

    match service.approve_subnet(&decoded_subnet_id, from_address).await {
        Ok(msg) => {
            log::info!("✓ APPROVAL SUCCESS: {}", msg);
            log::info!("=== END SUBNET APPROVAL (SUCCESS) ===");
            Ok(warp::reply::json(&ApiResponse::success(format!("Subnet {} approved successfully", decoded_subnet_id))))
        }
        Err(e) => {
            log::error!("✗ APPROVAL FAILED: {}", e);
            log::error!("Error details: {:?}", e);
            log::error!("=== END SUBNET APPROVAL (FAILED) ===");
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle list pending approvals request
async fn handle_list_pending_approvals(
    gateway_address: String,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    log::info!("=== LIST PENDING APPROVALS REQUEST ===");
    log::info!("Gateway address: {}", gateway_address);

    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    match service.list_pending_approvals(&gateway_address).await {
        Ok(pending_subnets) => {
            log::info!("Found {} pending subnets", pending_subnets.len());
            for subnet in &pending_subnets {
                if let Some(id) = subnet.get("subnet_id").and_then(|v| v.as_str()) {
                    log::info!("  - Pending subnet: {}", id);
                }
            }
            log::info!("=== END LIST PENDING APPROVALS ===");
            Ok(warp::reply::json(&ApiResponse::success(pending_subnets)))
        }
        Err(e) => {
            log::error!("Failed to list pending approvals: {}", e);
            log::error!("=== END LIST PENDING APPROVALS (ERROR) ===");
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

/// Approve a subnet
pub async fn approve_subnet(
    headers: Option<&HeaderMap>,
    subnet_service: Arc<SubnetService>,
    subnet_id: String,
    from_address: Option<String>,
) -> Result<impl Reply, Rejection> {
    log::info!("Approving subnet: {}", subnet_id);

    match subnet_service.approve_subnet(&subnet_id, from_address.as_deref()).await {
        Ok(_) => {
            log::info!("Successfully approved subnet: {}", subnet_id);
            Ok(reply::json(&serde_json::json!({
                "success": true,
                "message": format!("Subnet {} approved successfully", subnet_id)
            })))
        }
        Err(e) => {
            log::error!("Failed to approve subnet {}: {}", subnet_id, e);
            Ok(reply::json(&serde_json::json!({
                "success": false,
                "error": format!("Failed to approve subnet: {}", e)
            })))
        }
    }
}

/// List pending subnet approvals for a gateway
pub async fn list_pending_approvals(
    headers: Option<&HeaderMap>,
    subnet_service: Arc<SubnetService>,
    gateway_address: String,
) -> Result<impl Reply, Rejection> {
    log::info!("Listing pending approvals for gateway: {}", gateway_address);

    match subnet_service.list_pending_approvals(&gateway_address).await {
        Ok(pending_subnets) => {
            log::info!("Found {} pending approvals", pending_subnets.len());
            Ok(reply::json(&serde_json::json!({
                "success": true,
                "data": pending_subnets
            })))
        }
        Err(e) => {
            log::error!("Failed to list pending approvals: {}", e);
            Ok(reply::json(&serde_json::json!({
                "success": false,
                "error": format!("Failed to list pending approvals: {}", e)
            })))
        }
    }
}

/// Handle generate node config request
async fn handle_generate_node_config(
    subnet_id: String,
    query_params: HashMap<String, String>,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    // URL decode the subnet_id parameter
    let decoded_subnet_id = urlencoding::decode(&subnet_id)
        .map_err(|e| warp::reject::custom(ServerError(format!("Invalid subnet ID encoding: {}", e))))?
        .to_string();

    let validator_address = query_params.get("validator_address");

    log::info!("Generating node config for subnet: {}, validator: {:?}", decoded_subnet_id, validator_address);

    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    match service.generate_node_config(&decoded_subnet_id, validator_address.map(|s| s.as_str())).await {
        Ok(config_yaml) => {
            log::info!("Successfully generated node config for subnet: {}", decoded_subnet_id);
            Ok(warp::reply::json(&ApiResponse::success(serde_json::json!({
                "subnet_id": decoded_subnet_id,
                "config_yaml": config_yaml,
                "filename": format!("node_{}.yaml", decoded_subnet_id.replace('/', "_").replace(":", "_"))
            }))))
        }
        Err(e) => {
            log::error!("Failed to generate node config for subnet {}: {}", decoded_subnet_id, e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle generate node commands request
async fn handle_generate_node_commands(
    subnet_id: String,
    query_params: HashMap<String, String>,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    // URL decode the subnet_id parameter
    let decoded_subnet_id = urlencoding::decode(&subnet_id)
        .map_err(|e| warp::reject::custom(ServerError(format!("Invalid subnet ID encoding: {}", e))))?
        .to_string();

    let validator_address = query_params.get("validator_address");

    log::info!("Generating node commands for subnet: {}, validator: {:?}", decoded_subnet_id, validator_address);

    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = SubnetService::new(global);

    match service.generate_node_commands(&decoded_subnet_id, validator_address.map(|s| s.as_str())).await {
        Ok(commands_info) => {
            log::info!("Successfully generated node commands for subnet: {}", decoded_subnet_id);
            Ok(warp::reply::json(&ApiResponse::success(commands_info)))
        }
        Err(e) => {
            log::error!("Failed to generate node commands for subnet {}: {}", decoded_subnet_id, e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}