// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! UI Server implementation

use super::{AppState, DeploymentMode, DeploymentState, SubnetInstance, WebSocketClient};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use warp::{Filter, Reply};
use include_dir::{include_dir, Dir};
use tokio::time::{sleep, Duration};
use serde_json::json;
use uuid;

// Include the built frontend files at compile time
static FRONTEND_DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../ipc-ui/frontend/dist");

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
            deployments: Arc::new(Mutex::new(HashMap::new())),
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
        // Initialize with mock data
        self.initialize_mock_data().await?;

        // Create the combined server with both static files and API
        let addr: SocketAddr = format!("{}:{}", self.host, self.frontend_port)
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

        let routes = self.create_combined_routes();

        log::info!("UI server starting at http://{}", addr);
        log::info!("Serving frontend static files and backend API from single port");
        log::info!("Frontend: http://{}/", addr);
        log::info!("Backend API: http://{}/api/", addr);
        log::info!("WebSocket: ws://{}/ws", addr);

        warp::serve(routes)
            .run(addr)
            .await;

        Ok(())
    }

    /// Create combined routes for both API and static file serving
    fn create_combined_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let api_routes = self.api_routes();
        let websocket_route = self.websocket_route();
        let static_routes = self.static_file_routes();

        api_routes
            .or(websocket_route)
            .or(static_routes)
            .with(warp::log("api"))
    }

    /// Create API routes
    fn api_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let state_filter = warp::any().map({
            let state = self.state.clone();
            move || state.clone()
        });

        // GET /api/templates
        let templates = warp::path!("api" / "templates")
            .and(warp::get())
            .and(state_filter.clone())
            .map(|_state: AppState| {
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

        // GET /api/instances
        let instances = warp::path!("api" / "instances")
            .and(warp::get())
            .and(state_filter.clone())
            .map(|state: AppState| {
                let instances = state.instances.lock().unwrap();
                let instances_vec: Vec<_> = instances.values().collect();
                warp::reply::json(&instances_vec)
            });

        // GET /api/instances/:id
        let instance_by_id = warp::path!("api" / "instances" / String)
            .and(warp::get())
            .and(state_filter.clone())
            .map(|id: String, state: AppState| {
                let instances = state.instances.lock().unwrap();
                match instances.get(&id) {
                    Some(instance) => {
                        warp::reply::with_status(warp::reply::json(instance), warp::http::StatusCode::OK)
                    }
                    None => {
                        warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"error": "Instance not found"})),
                            warp::http::StatusCode::NOT_FOUND
                        )
                    }
                }
            });

        // POST /api/deploy
        let deploy = warp::path!("api" / "deploy")
            .and(warp::post())
            .and(warp::body::json())
            .and(state_filter.clone())
            .and_then(|config: serde_json::Value, state: AppState| async move {
                log::info!("Received deployment request: {}", config);

                // Parse deployment request
                let template = config["template"].as_str()
                    .ok_or_else(|| warp::reject::custom(InvalidRequest("Missing template field".to_string())))?
                    .to_string();

                let deployment_config = config["config"].clone();

                // Generate deployment ID
                let deployment_id = format!("deploy-{}", chrono::Utc::now().timestamp());

                // Create deployment state
                let deployment_state = DeploymentState::new(
                    deployment_id.clone(),
                    template.clone(),
                    deployment_config.clone(),
                );

                // Store deployment state
                {
                    let mut deployments = state.deployments.lock().unwrap();
                    deployments.insert(deployment_id.clone(), deployment_state.clone());
                }

                // Start background deployment task
                let state_clone = state.clone();
                let deployment_id_clone = deployment_id.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_deployment(state_clone.clone(), deployment_id_clone.clone(), template, deployment_config).await {
                        log::error!("Deployment failed: {}", e);

                        // Update deployment state with error
                        {
                            let mut deployments = state_clone.deployments.lock().unwrap();
                            if let Some(deployment) = deployments.get_mut(&deployment_id_clone) {
                                deployment.set_error(e.to_string());
                            }
                        } // Drop the mutex guard before the await

                        // Broadcast error to WebSocket clients
                        broadcast_deployment_progress(&state_clone, &deployment_id_clone, "error", 0, "failed", Some(e.to_string())).await;
                    }
                });

                Ok::<_, warp::Rejection>(warp::reply::json(&json!({
                    "deployment_id": deployment_id,
                    "status": "started",
                    "message": "Deployment initiated successfully"
                })))
            });

        templates
            .or(instances)
            .or(instance_by_id)
            .or(deploy)
    }

    /// Create WebSocket route
    fn websocket_route(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        warp::path("ws")
            .and(warp::ws())
            .and(warp::any().map({
                let state = self.state.clone();
                move || state.clone()
            }))
            .map(|ws: warp::ws::Ws, state: AppState| {
                ws.on_upgrade(move |socket| handle_websocket(socket, state))
            })
    }

    /// Create static file serving routes
    fn static_file_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        // Serve specific files
        let files = warp::get()
            .and(warp::path::tail())
            .and_then(serve_static_file);

        // Serve index.html for all routes (SPA routing)
        let spa_fallback = warp::get()
            .and(warp::path::tail())
            .and_then(serve_spa_fallback);

        files.or(spa_fallback)
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

/// Serve static files from the embedded frontend
async fn serve_static_file(path: warp::path::Tail) -> Result<Box<dyn Reply>, warp::Rejection> {
    let path_str = path.as_str();

    // Handle root path
    if path_str.is_empty() || path_str == "/" {
        return serve_index_html().await;
    }

    // Try to find the file in the embedded directory
    if let Some(file) = FRONTEND_DIST.get_file(path_str) {
        let mime = mime_guess::from_path(path_str).first_or_octet_stream();

        let reply = warp::reply::with_header(
            file.contents(),
            "content-type",
            mime.as_ref(),
        );

        let final_reply = warp::reply::with_header(
            reply,
            "cache-control",
            if path_str.contains("assets/") {
                "public, max-age=31536000" // 1 year for assets
            } else {
                "public, max-age=3600" // 1 hour for other files
            },
        );

        Ok(Box::new(final_reply))
    } else {
        // File not found, reject to try SPA fallback
        Err(warp::reject::not_found())
    }
}

/// Serve SPA fallback (index.html for all routes)
async fn serve_spa_fallback(_path: warp::path::Tail) -> Result<Box<dyn Reply>, warp::Rejection> {
    serve_index_html().await
}

/// Serve the index.html file
async fn serve_index_html() -> Result<Box<dyn Reply>, warp::Rejection> {
    if let Some(index_file) = FRONTEND_DIST.get_file("index.html") {
        let reply = warp::reply::with_header(
            index_file.contents(),
            "content-type",
            "text/html",
        );

        let final_reply = warp::reply::with_header(
            reply,
            "cache-control",
            "no-cache", // Don't cache index.html for SPA routing
        );

        Ok(Box::new(final_reply))
    } else {
        log::error!("index.html not found in embedded frontend files");
        Err(warp::reject::not_found())
    }
}

/// Custom rejection type for invalid requests
#[derive(Debug)]
struct InvalidRequest(String);

impl warp::reject::Reject for InvalidRequest {}

/// Handle actual subnet deployment
async fn handle_deployment(
    state: AppState,
    deployment_id: String,
    _template: String,
    config: serde_json::Value,
) -> Result<()> {
    log::info!("Starting deployment process for {}", deployment_id);

    // Define deployment steps
    let steps = vec![
        ("validate", "Validating Configuration", 10),
        ("prepare", "Preparing Deployment Files", 20),
        ("contracts", "Deploying Smart Contracts", 40),
        ("genesis", "Creating Genesis Block", 60),
        ("validators", "Initializing Validators", 80),
        ("activation", "Activating Subnet", 90),
        ("verification", "Running Verification", 100),
    ];

    for (step_id, step_name, progress) in steps {
        log::info!("Deployment {}: {}", deployment_id, step_name);

        // Update deployment state
        {
            let mut deployments = state.deployments.lock().unwrap();
            if let Some(deployment) = deployments.get_mut(&deployment_id) {
                deployment.update_progress(
                    step_id.to_string(),
                    progress,
                    "in_progress".to_string(),
                    Some(step_name.to_string()),
                );
            }
        }

        // Broadcast progress to WebSocket clients
        broadcast_deployment_progress(
            &state,
            &deployment_id,
            step_id,
            progress,
            "in_progress",
            Some(step_name.to_string()),
        ).await;

        // Simulate deployment step processing
        match step_id {
            "validate" => {
                // Validate configuration
                validate_deployment_config(&config)?;
                sleep(Duration::from_secs(2)).await;
            }
            "prepare" => {
                // Prepare deployment files
                prepare_deployment_files(&config).await?;
                sleep(Duration::from_secs(3)).await;
            }
            "contracts" => {
                // Deploy smart contracts
                deploy_smart_contracts(&config).await?;
                sleep(Duration::from_secs(5)).await;
            }
            "genesis" => {
                // Create genesis block
                create_genesis_block(&config).await?;
                sleep(Duration::from_secs(3)).await;
            }
            "validators" => {
                // Initialize validators
                initialize_validators(&config).await?;
                sleep(Duration::from_secs(4)).await;
            }
            "activation" => {
                // Activate subnet
                activate_subnet_deployment(&config).await?;
                sleep(Duration::from_secs(2)).await;
            }
            "verification" => {
                // Run verification
                verify_deployment(&config).await?;
                sleep(Duration::from_secs(2)).await;
            }
            _ => {}
        }
    }

    // Mark deployment as completed
    {
        let mut deployments = state.deployments.lock().unwrap();
        if let Some(deployment) = deployments.get_mut(&deployment_id) {
            deployment.update_progress(
                "completed".to_string(),
                100,
                "completed".to_string(),
                Some("Deployment completed successfully".to_string()),
            );
        }
    }

    // Broadcast completion
    broadcast_deployment_progress(
        &state,
        &deployment_id,
        "completed",
        100,
        "completed",
        Some("Deployment completed successfully".to_string()),
    ).await;

    // Create subnet instance
    create_subnet_instance(&state, &deployment_id, &config).await?;

    log::info!("Deployment {} completed successfully", deployment_id);
    Ok(())
}

/// Broadcast deployment progress to WebSocket clients
async fn broadcast_deployment_progress(
    state: &AppState,
    deployment_id: &str,
    step: &str,
    progress: u8,
    status: &str,
    message: Option<String>,
) {
    let message = json!({
        "type": "deployment_progress",
        "data": {
            "deployment_id": deployment_id,
            "step": step,
            "progress": progress,
            "status": status,
            "message": message
        }
    });

    let ws_message = warp::ws::Message::text(message.to_string());

    let clients = state.websocket_clients.lock().unwrap();
    for client in clients.iter() {
        if let Err(e) = client.sender.send(ws_message.clone()) {
            log::warn!("Failed to send WebSocket message to client {}: {}", client.id, e);
        }
    }

    log::debug!("Broadcasted deployment progress: {} - {} ({}%)", deployment_id, step, progress);
}

/// Validate deployment configuration
fn validate_deployment_config(config: &serde_json::Value) -> Result<()> {
    log::info!("Validating deployment configuration");

    // Check required fields
    let required_fields = ["parent", "minValidatorStake", "minValidators", "permissionMode"];
    for field in required_fields {
        if config.get(field).is_none() {
            return Err(anyhow::anyhow!("Missing required field: {}", field));
        }
    }

    log::info!("Configuration validation completed");
    Ok(())
}

/// Prepare deployment files
async fn prepare_deployment_files(_config: &serde_json::Value) -> Result<()> {
    log::info!("Preparing deployment files");

    // In a real implementation, this would:
    // - Create subnet-init.yaml from UI config
    // - Set up temporary directories
    // - Prepare wallet configurations

    log::info!("Deployment files prepared");
    Ok(())
}

/// Deploy smart contracts
async fn deploy_smart_contracts(_config: &serde_json::Value) -> Result<()> {
    log::info!("Deploying smart contracts");

    // In a real implementation, this would:
    // - Deploy gateway and registry contracts
    // - Record contract addresses
    // - Set up subnet contracts

    log::info!("Smart contracts deployed");
    Ok(())
}

/// Create genesis block
async fn create_genesis_block(_config: &serde_json::Value) -> Result<()> {
    log::info!("Creating genesis block");

    // In a real implementation, this would:
    // - Generate genesis.json from parent chain
    // - Seal genesis with validators
    // - Set up initial state

    log::info!("Genesis block created");
    Ok(())
}

/// Initialize validators
async fn initialize_validators(_config: &serde_json::Value) -> Result<()> {
    log::info!("Initializing validators");

    // In a real implementation, this would:
    // - Set up validator nodes
    // - Configure validator power/stakes
    // - Initialize validator consensus

    log::info!("Validators initialized");
    Ok(())
}

/// Activate subnet deployment
async fn activate_subnet_deployment(_config: &serde_json::Value) -> Result<()> {
    log::info!("Activating subnet");

    // In a real implementation, this would:
    // - Join validators to subnet
    // - Start subnet consensus
    // - Begin block production

    log::info!("Subnet activated");
    Ok(())
}

/// Verify deployment
async fn verify_deployment(_config: &serde_json::Value) -> Result<()> {
    log::info!("Verifying deployment");

    // In a real implementation, this would:
    // - Check subnet is producing blocks
    // - Verify validator participation
    // - Test basic functionality

    log::info!("Deployment verified");
    Ok(())
}

/// Create subnet instance after successful deployment
async fn create_subnet_instance(
    state: &AppState,
    deployment_id: &str,
    config: &serde_json::Value,
) -> Result<()> {
    let instance = SubnetInstance {
        id: deployment_id.to_string(),
        name: config["name"].as_str().unwrap_or("Unnamed Subnet").to_string(),
        status: "active".to_string(),
        template: config.get("template").and_then(|t| t.as_str())
            .unwrap_or("development").to_string(),
        parent: config["parent"].as_str().unwrap_or("Unknown").to_string(),
        created_at: chrono::Utc::now(),
        validators: vec![], // TODO: Extract from config
        config: config.clone(),
    };

    let mut instances = state.instances.lock().unwrap();
    instances.insert(deployment_id.to_string(), instance);

    log::info!("Created subnet instance for deployment {}", deployment_id);
    Ok(())
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
                    let text = msg.to_str().unwrap_or("invalid");
                    log::info!("Received WebSocket message: {}", text);

                    // Parse incoming message
                    if let Ok(message) = serde_json::from_str::<serde_json::Value>(text) {
                        match message["type"].as_str() {
                            Some("subscribe_deployment") => {
                                if let Some(deployment_id) = message["data"]["deployment_id"].as_str() {
                                    log::info!("Client {} subscribing to deployment {}", client_id, deployment_id);
                                    // Client is now subscribed to this deployment
                                    // In a more complex implementation, we'd track subscriptions per client
                                }
                            }
                            Some("ping") => {
                                // Respond with pong
                                let pong_message = json!({
                                    "type": "pong"
                                });
                                let ws_message = warp::ws::Message::text(pong_message.to_string());

                                let clients = state.websocket_clients.lock().unwrap();
                                if let Some(client) = clients.iter().find(|c| c.id == client_id) {
                                    let _ = client.sender.send(ws_message);
                                }
                            }
                            _ => {
                                log::debug!("Unknown WebSocket message type: {}", text);
                            }
                        }
                    }
                } else if msg.is_close() {
                    log::info!("WebSocket connection closed");
                    break;
                }
            }
            Err(e) => {
                log::error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Clean up
    send_task.abort();

    // Remove client from state
    {
        let mut clients = state.websocket_clients.lock().unwrap();
        clients.retain(|client| client.id != client_id);
    }

    log::info!("WebSocket connection cleaned up");
}
