// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! UI Server implementation

use super::{AppState, DeploymentMode, SubnetInstance, WebSocketClient};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use warp::Filter;

/// UI Server that handles both frontend serving and backend API
pub struct UIServer {
    host: String,
    frontend_port: u16,
    backend_port: u16,
    mode: DeploymentMode,
    config_path: String,
    state: AppState,
}

impl UIServer {
    /// Create a new UI server instance
    pub fn new(
        host: String,
        frontend_port: u16,
        backend_port: u16,
        mode: String,
        config_path: String,
    ) -> Result<Self> {
        let mode = mode.parse::<DeploymentMode>()?;

        let state = AppState {
            config_path: config_path.clone(),
            mode: mode.clone(),
            instances: Arc::new(Mutex::new(HashMap::new())),
            websocket_clients: Arc::new(Mutex::new(Vec::new())),
        };

        Ok(UIServer {
            host,
            frontend_port,
            backend_port,
            mode,
            config_path,
            state,
        })
    }

    /// Start the UI server (both frontend and backend)
    pub async fn start(&mut self) -> Result<()> {
        // Start backend API server
        let backend_state = self.state.clone();
        let backend_addr = format!("{}:{}", self.host, self.backend_port);

        tokio::spawn(async move {
            if let Err(e) = start_backend_server(backend_addr, backend_state).await {
                log::error!("Backend server error: {}", e);
            }
        });

        // Start frontend server (for now, just log that it would serve static files)
        let frontend_addr = format!("{}:{}", self.host, self.frontend_port);
        log::info!("Frontend server would serve static files at {}", frontend_addr);
        log::info!("In production, this would serve the built Vue.js application");

        // Initialize with some mock data for testing
        self.initialize_mock_data().await?;

        Ok(())
    }

    /// Initialize with mock subnet instances for testing
    async fn initialize_mock_data(&self) -> Result<()> {
        let mut instances = self.state.instances.lock().unwrap();

        // Add some mock subnet instances
        let mock_instances = vec![
            SubnetInstance {
                id: "subnet-001".to_string(),
                name: "Development Test".to_string(),
                status: "Active".to_string(),
                template: "Development Template".to_string(),
                parent: "/r31337".to_string(),
                created_at: chrono::Utc::now() - chrono::Duration::hours(2),
                validators: vec![
                    super::ValidatorInfo {
                        address: "0x1234567890123456789012345678901234567890".to_string(),
                        stake: "1.0".to_string(),
                        power: 1,
                        status: "Active".to_string(),
                    }
                ],
                config: serde_json::json!({
                    "permissionMode": "federated",
                    "minValidators": 1,
                    "minValidatorStake": 1.0
                }),
            },
            SubnetInstance {
                id: "subnet-002".to_string(),
                name: "Production Subnet A".to_string(),
                status: "Active".to_string(),
                template: "Production Template".to_string(),
                parent: "/r31337".to_string(),
                created_at: chrono::Utc::now() - chrono::Duration::days(1),
                validators: vec![
                    super::ValidatorInfo {
                        address: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string(),
                        stake: "100.0".to_string(),
                        power: 10,
                        status: "Active".to_string(),
                    },
                    super::ValidatorInfo {
                        address: "0x1111222233334444555566667777888899990000".to_string(),
                        stake: "150.0".to_string(),
                        power: 15,
                        status: "Active".to_string(),
                    }
                ],
                config: serde_json::json!({
                    "permissionMode": "collateral",
                    "minValidators": 5,
                    "minValidatorStake": 100.0
                }),
            }
        ];

        for instance in mock_instances {
            instances.insert(instance.id.clone(), instance);
        }

        log::info!("Initialized with {} mock subnet instances", instances.len());
        Ok(())
    }
}

/// Start the backend API server
async fn start_backend_server(addr: String, state: AppState) -> Result<()> {
    log::info!("Starting backend API server on {}", addr);

    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    // API routes
    let api_routes = api_routes(state.clone());

    // WebSocket route
    let ws_route = websocket_route(state.clone());

    // Combine all routes
    let routes = api_routes
        .or(ws_route)
        .with(cors)
        .with(warp::log("api"));

    let socket_addr: SocketAddr = addr.parse()?;
    warp::serve(routes).run(socket_addr).await;

    Ok(())
}

/// Create API routes
fn api_routes(
    state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let state_filter = warp::any().map(move || state.clone());

    // GET /api/instances - List all subnet instances
    let list_instances = warp::path!("api" / "instances")
        .and(warp::get())
        .and(state_filter.clone())
        .map(|state: AppState| {
            let instances = state.instances.lock().unwrap();
            let instances_vec: Vec<_> = instances.values().cloned().collect();
            warp::reply::json(&instances_vec)
        });

    // GET /api/instances/{id} - Get specific subnet instance
    let get_instance = warp::path!("api" / "instances" / String)
        .and(warp::get())
        .and(state_filter.clone())
        .map(|id: String, state: AppState| {
            let instances = state.instances.lock().unwrap();
            match instances.get(&id) {
                Some(instance) => warp::reply::with_status(
                    warp::reply::json(instance),
                    warp::http::StatusCode::OK,
                ),
                None => warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({"error": "Instance not found"})),
                    warp::http::StatusCode::NOT_FOUND,
                ),
            }
        });

    // POST /api/deploy - Deploy new subnet
    let deploy_subnet = warp::path!("api" / "deploy")
        .and(warp::post())
        .and(warp::body::json())
        .and(state_filter.clone())
                .map(|config: serde_json::Value, _state: AppState| {
            log::info!("Received deployment request: {}", config);

            // For now, return a mock deployment ID
            let deployment_id = format!("deploy-{}", chrono::Utc::now().timestamp());

            // In a real implementation, this would:
            // 1. Validate the configuration
            // 2. Start the deployment process
            // 3. Return deployment status via WebSocket

            warp::reply::json(&serde_json::json!({
                "deployment_id": deployment_id,
                "status": "started",
                "message": "Deployment initiated successfully"
            }))
        });

    // GET /api/templates - Get available templates
    let get_templates = warp::path!("api" / "templates")
        .and(warp::get())
        .map(|| {
            // Mock template data matching our frontend templates
            let templates = serde_json::json!([
                {
                    "id": "development",
                    "name": "Development Template",
                    "description": "Perfect for local development and testing",
                    "icon": "🧪",
                    "features": [
                        "Federated mode for quick setup",
                        "Minimal validators (1-3)",
                        "Low stakes and barriers",
                        "Fast checkpoints",
                        "Local network compatible"
                    ]
                },
                {
                    "id": "production",
                    "name": "Production Template",
                    "description": "Battle-tested configuration for live deployments",
                    "icon": "🏭",
                    "features": [
                        "Collateral mode",
                        "High security settings",
                        "Robust validator requirements",
                        "Conservative parameters",
                        "High stakes protection"
                    ]
                }
            ]);
            warp::reply::json(&templates)
        });

    list_instances
        .or(get_instance)
        .or(deploy_subnet)
        .or(get_templates)
}

/// Create WebSocket route
fn websocket_route(
    state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || state.clone()))
        .map(|ws: warp::ws::Ws, state: AppState| {
            ws.on_upgrade(move |socket| handle_websocket(socket, state))
        })
}

/// Handle WebSocket connections
async fn handle_websocket(ws: warp::ws::WebSocket, state: AppState) {
    log::info!("New WebSocket connection established");

    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // Generate client ID
    let client_id = uuid::Uuid::new_v4().to_string();

    // Add client to state
    {
        let mut clients = state.websocket_clients.lock().unwrap();
        clients.push(WebSocketClient {
            id: client_id.clone(),
            sender: tx,
        });
    }

    // Handle outgoing messages
    let send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if ws_tx.send(message).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    if let Ok(text) = msg.to_str() {
                        log::debug!("Received WebSocket message: {}", text);

                        // Echo message back for now (in real implementation, handle different message types)
                        let response = serde_json::json!({
                            "type": "echo",
                            "data": text
                        });

                        let clients = state.websocket_clients.lock().unwrap();
                        for client in clients.iter() {
                            if client.id == client_id {
                                let _ = client.sender.send(warp::ws::Message::text(response.to_string()));
                                break;
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

    // Remove client from state
    {
        let mut clients = state.websocket_clients.lock().unwrap();
        clients.retain(|client| client.id != client_id);
    }

    send_task.abort();
    log::info!("WebSocket connection closed");
}