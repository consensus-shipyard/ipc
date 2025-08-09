// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! UI Server implementation - Simplified to focus on routing and server management

use super::api::{
    deployment::deployment_routes,
    gateway::gateway_routes,
    network::network_routes,
    subnet::subnet_routes,
    transactions::transaction_routes,
    wallet::wallet_routes,
};
use super::{AppState, DeploymentMode};
use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use warp::Filter;

/// Create WebSocket routes (placeholder function)
fn websocket_routes(state: AppState) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let _state = state.clone();
            ws.on_upgrade(move |_socket| async move {
                // WebSocket handling placeholder
            })
        })
}

/// Start the UI server
pub async fn start_ui_server(
    config_path: String,
    addr: SocketAddr,
) -> Result<()> {
    log::info!("Starting IPC UI server on {}", addr);

    // Create shared state
    let state = AppState {
        config_path: config_path.clone(),
        mode: DeploymentMode::Development,
        deployments: Arc::new(Mutex::new(HashMap::new())),
        instances: Arc::new(Mutex::new(HashMap::new())),
        websocket_clients: Arc::new(Mutex::new(Vec::new())),
        deployed_gateways: Arc::new(Mutex::new(HashMap::new())),
        subnet_metadata: Arc::new(Mutex::new(HashMap::new())),
    };

    // Create API routes
    let api_routes = warp::path("api")
        .and(
            wallet_routes(state.clone())
                .or(subnet_routes(state.clone()))
                .or(gateway_routes(state.clone()))
                .or(deployment_routes(state.clone()))
                .or(transaction_routes(state.clone()))
                .or(network_routes(state.clone()))
        );

    // Create WebSocket routes
    let ws_routes = websocket_routes(state.clone());

    // Serve static files from the frontend build
    let static_files = warp::fs::dir("ipc-ui/frontend/dist/");

    // Combine all routes
    let routes = api_routes
        .or(ws_routes)
        .or(static_files)
        .with(warp::cors().allow_any_origin());

    // Start the server
    warp::serve(routes)
        .run(addr)
        .await;

    Ok(())
}
