// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Network API endpoints

use super::super::AppState;
use super::types::ApiResponse;
use anyhow::Result;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::Infallible;
use std::time::Duration;
use warp::{Filter, Reply};

/// Network connection status response
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkConnectionStatus {
    pub network_id: String,
    pub network_name: String,
    pub rpc_url: String,
    pub connected: bool,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
    pub last_checked: String,
}

/// Network connection test request
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct NetworkConnectionTestRequest {
    pub network_id: String,
    pub network_name: String,
    pub rpc_url: String,
    pub network_type: String,
}

/// Create network API routes
pub fn network_routes(
    state: AppState,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let test_connection_route = warp::path!("network" / "test-connection")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_test_network_connection);

    let health_route = warp::path("health")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_health_check);

    test_connection_route.or(health_route)
}

/// Helper to pass state to handlers
fn with_state(state: AppState) -> impl Filter<Extract = (AppState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Handle network connection test request
async fn handle_test_network_connection(
    request: NetworkConnectionTestRequest,
    _state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    log::info!(
        "Testing connection to network: {} ({})",
        request.network_name,
        request.rpc_url
    );

    let start_time = std::time::Instant::now();
    let mut status = NetworkConnectionStatus {
        network_id: request.network_id,
        network_name: request.network_name,
        rpc_url: request.rpc_url.clone(),
        connected: false,
        response_time_ms: None,
        error: None,
        last_checked: chrono::Utc::now().to_rfc3339(),
    };

    // Test the network connection
    match test_network_connection(&request.rpc_url).await {
        Ok(()) => {
            let response_time = start_time.elapsed().as_millis() as u64;
            status.connected = true;
            status.response_time_ms = Some(response_time);
            log::info!("Network connection successful ({}ms)", response_time);
        }
        Err(e) => {
            status.connected = false;
            status.error = Some(e.to_string());
            log::warn!("Network connection failed: {}", e);
        }
    }

    Ok(warp::reply::json(&ApiResponse::success(status)))
}

/// Handle health check request
async fn handle_health_check(_state: AppState) -> Result<impl Reply, warp::Rejection> {
    let health_status = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "ipc-cli-ui"
    });

    Ok(warp::reply::json(&ApiResponse::success(health_status)))
}

/// Test connection to a network RPC endpoint
async fn test_network_connection(rpc_url: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    // Try a basic JSON-RPC call to test connectivity
    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "eth_chainId",
        "params": [],
        "id": 1
    });

    let response = client
        .post(rpc_url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        // Additional validation - check if we get a valid JSON-RPC response
        let response_text = response.text().await?;
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|_| anyhow::anyhow!("Invalid JSON response from RPC endpoint"))?;

        // Check if response contains either result or error (valid JSON-RPC response structure)
        if response_json.get("result").is_some() || response_json.get("error").is_some() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid JSON-RPC response structure"))
        }
    } else {
        Err(anyhow::anyhow!(
            "HTTP error: {} - {}",
            response.status(),
            response.text().await.unwrap_or_default()
        ))
    }
}
