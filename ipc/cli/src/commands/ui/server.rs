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
use super::websocket::types::{IncomingMessage, OutgoingMessage};
use super::{AppState, DeploymentMode};
use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use warp::Filter;

/// Create WebSocket routes
fn websocket_routes(state: AppState) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let state = state.clone();
            ws.on_upgrade(move |socket| async move {
                handle_websocket_connection(socket, state).await;
            })
        })
}

/// Handle WebSocket connection
async fn handle_websocket_connection(websocket: warp::ws::WebSocket, state: AppState) {
    use futures_util::{SinkExt, StreamExt};
    use tokio::sync::Mutex;
    use std::sync::Arc;

    let (tx, mut rx) = websocket.split();
    let tx = Arc::new(Mutex::new(tx));

    // Add client to the list
    {
        let mut clients_guard = state.websocket_clients.lock().unwrap();
        clients_guard.push(tx.clone());
        log::info!("WebSocket client connected. Total clients: {}", clients_guard.len());
    }

    // Handle incoming messages
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    log::debug!("Received WebSocket message: {}", text);

                    // Try to parse as structured message first
                    match serde_json::from_str::<IncomingMessage>(&text) {
                        Ok(incoming_msg) => {
                            match incoming_msg {
                                IncomingMessage::Ping => {
                                    // Send structured pong response
                                    let pong_response = OutgoingMessage::Pong;
                                    if let Ok(pong_json) = serde_json::to_string(&pong_response) {
                                        let mut sink = tx.lock().await;
                                        if let Err(e) = sink.send(warp::ws::Message::text(pong_json)).await {
                                            log::error!("Failed to send pong: {}", e);
                                            break;
                                        }
                                    }
                                }
                                IncomingMessage::SubscribeDeployment { deployment_id } => {
                                    log::info!("Client subscribed to deployment: {}", deployment_id);
                                    // TODO: Handle deployment subscription
                                }
                                IncomingMessage::SubscribeInstance { instance_id } => {
                                    log::info!("Client subscribed to instance: {}", instance_id);
                                    // TODO: Handle instance subscription
                                }
                            }
                        }
                        Err(_) => {
                            // Fallback: handle legacy ping messages
                            if text.contains("ping") {
                                let pong_response = OutgoingMessage::Pong;
                                if let Ok(pong_json) = serde_json::to_string(&pong_response) {
                                    let mut sink = tx.lock().await;
                                    if let Err(e) = sink.send(warp::ws::Message::text(pong_json)).await {
                                        log::error!("Failed to send pong: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Remove client when connection closes
    {
        let mut clients_guard = state.websocket_clients.lock().unwrap();
        let initial_count = clients_guard.len();
        clients_guard.retain(|client| !Arc::ptr_eq(client, &tx));
        log::info!("WebSocket client disconnected. Clients: {} -> {}", initial_count, clients_guard.len());
    }
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
