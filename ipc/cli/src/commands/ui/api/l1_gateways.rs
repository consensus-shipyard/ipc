// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! L1 Gateway configuration API endpoints

use super::super::services::gateway_service::GatewayService;
use super::super::AppState;
use super::types::ApiResponse;
use crate::GlobalArguments;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use warp::{Filter, Reply};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1GatewayConfig {
    pub id: String,
    pub name: String,
    pub address: String,
    pub registry_address: String,
    pub network_id: String,
    pub network_name: String,
    pub chain_id: u64,
    pub deployed_at: String,
    pub deployer_address: String,
    pub is_default: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1GatewayConfigFile {
    pub default_gateway: Option<String>,
    pub gateways: Vec<L1GatewayConfig>,
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGatewaySelectionRequest {
    pub gateway_id: String,
}

impl Default for L1GatewayConfigFile {
    fn default() -> Self {
        L1GatewayConfigFile {
            default_gateway: None,
            gateways: Vec::new(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Get L1 gateway configuration
pub async fn get_l1_gateway_config(state: AppState) -> Result<impl Reply, warp::Rejection> {
    log::info!("Getting L1 gateway configuration from discovered gateways");

    // Get discovered gateways and convert them to L1Gateway format
    let global = crate::GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let gateway_service = crate::commands::ui::services::GatewayService::new(global);

    match gateway_service.discover_gateways(None).await {
        Ok(discovered_gateways) => {
            log::info!(
                "Discovered {} gateways for L1 configuration",
                discovered_gateways.len()
            );

            // Convert discovered gateways to L1GatewayConfig format
            let l1_gateways: Vec<L1GatewayConfig> = discovered_gateways
                .iter()
                .filter(|gw| {
                    // Only include L1 gateways (root network gateways)
                    gw.parent_network.starts_with("/r")
                        && gw.parent_network.matches('/').count() == 1 // e.g., "/r31337"
                })
                .enumerate()
                .map(|(index, gw)| {
                    L1GatewayConfig {
                        id: gw.id.clone(),
                        name: gw
                            .name
                            .clone()
                            .unwrap_or_else(|| format!("Gateway {}", index + 1)),
                        address: gw.address.clone(),
                        registry_address: gw.registry_address.clone(),
                        network_id: gw.parent_network.clone(),
                        network_name: format!(
                            "Local Anvil {}",
                            gw.parent_network.replace("/r", "")
                        ),
                        chain_id: 31337, // Default for local anvil, could be parsed from network_id
                        deployed_at: gw.deployed_at.to_rfc3339(),
                        deployer_address: "unknown".to_string(), // Not available in GatewayInfo
                        is_default: index == 0,                  // Make first gateway default
                        description: Some(format!(
                            "Gateway contract serving network {}",
                            gw.parent_network
                        )),
                    }
                })
                .collect();

            let config = L1GatewayConfigFile {
                default_gateway: l1_gateways.first().map(|gw| gw.id.clone()),
                gateways: l1_gateways,
                last_updated: chrono::Utc::now().to_rfc3339(),
            };

            log::info!(
                "Returning {} L1 gateways in configuration",
                config.gateways.len()
            );

            let response = ApiResponse {
                success: true,
                data: Some(serde_json::to_value(config).unwrap()),
                error: None,
            };

            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            log::error!("Failed to discover gateways for L1 configuration: {}", e);

            // Return empty config on error
            let config = L1GatewayConfigFile::default();
            let response = ApiResponse {
                success: true,
                data: Some(serde_json::to_value(config).unwrap()),
                error: None,
            };

            Ok(warp::reply::json(&response))
        }
    }
}

/// Update L1 gateway selection
pub async fn update_l1_gateway_selection(
    request: UpdateGatewaySelectionRequest,
    _state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    log::info!("Updating L1 gateway selection to: {}", request.gateway_id);

    // For now, just acknowledge the update
    // In a real implementation, this would update the config file
    let response = ApiResponse {
        success: true,
        data: Some(serde_json::json!({
            "selected_gateway": request.gateway_id,
            "updated_at": chrono::Utc::now().to_rfc3339()
        })),
        error: None,
    };

    Ok(warp::reply::json(&response))
}

/// Add a new L1 gateway
pub async fn add_l1_gateway(
    gateway: L1GatewayConfig,
    _state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    log::info!(
        "Adding new L1 gateway: {} ({})",
        gateway.name,
        gateway.address
    );

    // For now, just acknowledge the addition
    // In a real implementation, this would add to the config file
    let response = ApiResponse {
        success: true,
        data: Some(serde_json::to_value(&gateway).unwrap()),
        error: None,
    };

    Ok(warp::reply::json(&response))
}

/// Remove an L1 gateway
pub async fn remove_l1_gateway(
    gateway_id: String,
    _state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    log::info!("Removing L1 gateway: {}", gateway_id);

    // For now, just acknowledge the removal
    // In a real implementation, this would remove from the config file
    let response = ApiResponse {
        success: true,
        data: Some(serde_json::json!({
            "removed_gateway_id": gateway_id,
            "removed_at": chrono::Utc::now().to_rfc3339()
        })),
        error: None,
    };

    Ok(warp::reply::json(&response))
}

/// Helper to pass state to handlers
fn with_state(state: AppState) -> impl Filter<Extract = (AppState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Create L1 gateway API routes
pub fn l1_gateway_routes(
    state: AppState,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let get_config = warp::path!("l1-gateways" / "config")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_get_l1_gateway_config);

    let update_selection = warp::path!("l1-gateways" / "selection")
        .and(warp::put())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_update_l1_gateway_selection);

    let add_gateway = warp::path("l1-gateways")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_add_l1_gateway);

    let remove_gateway = warp::path!("l1-gateways" / String)
        .and(warp::delete())
        .and(with_state(state.clone()))
        .and_then(handle_remove_l1_gateway);

    get_config
        .or(update_selection)
        .or(add_gateway)
        .or(remove_gateway)
}

/// Handle get L1 gateway configuration request
async fn handle_get_l1_gateway_config(state: AppState) -> Result<impl Reply, warp::Rejection> {
    get_l1_gateway_config(state).await
}

/// Handle update L1 gateway selection request
async fn handle_update_l1_gateway_selection(
    request: UpdateGatewaySelectionRequest,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    update_l1_gateway_selection(request, state).await
}

/// Handle add L1 gateway request
async fn handle_add_l1_gateway(
    gateway: L1GatewayConfig,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    add_l1_gateway(gateway, state).await
}

/// Handle remove L1 gateway request
async fn handle_remove_l1_gateway(
    gateway_id: String,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    remove_l1_gateway(gateway_id, state).await
}
