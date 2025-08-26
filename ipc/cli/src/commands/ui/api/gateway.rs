// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Gateway API endpoints

use super::super::services::GatewayService;
use super::super::AppState;
use super::types::{ApiResponse, InvalidRequest, ServerError};
use crate::GlobalArguments;
use anyhow::Result;
use serde_json;
use std::convert::Infallible;
use warp::{Filter, Reply};

/// Create gateway API routes
pub fn gateway_routes(
    state: AppState,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let gateways_route = warp::path("gateways")
        .and(warp::get())
        .and(warp::header::headers_cloned())
        .and(with_state(state.clone()))
        .and_then(handle_get_gateways);

    let discover_route = warp::path!("gateways" / "discover")
        .and(warp::post())
        .and(warp::header::headers_cloned())
        .and(with_state(state.clone()))
        .and_then(handle_discover_gateways);

    let gateway_by_id_route = warp::path!("gateways" / String)
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_get_gateway_by_id);

    let update_gateway_route = warp::path!("gateways" / String)
        .and(warp::put())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_update_gateway);

    gateways_route
        .or(discover_route)
        .or(gateway_by_id_route)
        .or(update_gateway_route)
}

/// Helper to pass state to handlers
fn with_state(state: AppState) -> impl Filter<Extract = (AppState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Handle get gateways request
async fn handle_get_gateways(
    headers: warp::http::HeaderMap,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    log::debug!("Get gateways request with headers: {:?}", headers);

    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = GatewayService::new(global);

    match service.discover_gateways(Some(&headers)).await {
        Ok(gateways) => {
            log::info!("Found {} gateways for selected network", gateways.len());
            Ok(warp::reply::json(&ApiResponse::success(gateways)))
        }
        Err(e) => {
            log::error!("Get gateways failed: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle discover gateways request
async fn handle_discover_gateways(
    headers: warp::http::HeaderMap,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    log::debug!("Discover gateways request with headers: {:?}", headers);

    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = GatewayService::new(global);

    match service.discover_gateways(Some(&headers)).await {
        Ok(gateways) => {
            log::info!(
                "Discovered {} gateways for selected network",
                gateways.len()
            );
            Ok(warp::reply::json(&ApiResponse::success(gateways)))
        }
        Err(e) => {
            log::error!("Discover gateways failed: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle get gateway by ID request
async fn handle_get_gateway_by_id(
    gateway_id: String,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = GatewayService::new(global);

    match service.get_gateway_info(&gateway_id).await {
        Ok(Some(gateway)) => Ok(warp::reply::json(&ApiResponse::success(gateway))),
        Ok(None) => Err(warp::reject::custom(InvalidRequest(
            "Gateway not found".to_string(),
        ))),
        Err(e) => {
            log::error!("Get gateway by ID failed: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Handle update gateway request
async fn handle_update_gateway(
    gateway_id: String,
    updates: serde_json::Value,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let service = GatewayService::new(global);

    match service.update_gateway(&gateway_id, &updates).await {
        Ok(gateway) => Ok(warp::reply::json(&ApiResponse::success(gateway))),
        Err(e) => {
            log::error!("Update gateway failed: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}
