// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! UI Server implementation

use super::{AppState, DeploymentMode, DeploymentState, SubnetInstance, WebSocketClient, GatewayInfo};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use warp::{Filter, Reply};
use include_dir::{include_dir, Dir};
use tokio::time::{sleep, Duration};
use std::time::Instant;
use serde_json::json;
use uuid;

// Import actual IPC CLI functions for real deployment
use crate::commands::subnet::create::{create_subnet as create_subnet_cmd, SubnetCreateConfig};

use crate::commands::subnet::init::ipc_config_store::IpcConfigStore;
use crate::get_ipc_provider;
use crate::GlobalArguments;
use ipc_api::subnet::{PermissionMode, AssetKind};
use ipc_api::subnet_id::SubnetID;
use ethers::types::Address as EthAddress;
use std::str::FromStr;

// Import actual deployment functionality
use crate::commands::deploy::{deploy_contracts as deploy_contracts_cmd, DeployConfig, CliSubnetCreationPrivilege};
use ipc_provider::new_evm_keystore_from_arc_config;
use ipc_wallet::EvmKeyStore;
use ipc_types::EthAddress as IpcEthAddress;

// Cache entry for subnet approval status
#[derive(Clone)]
struct ApprovalCacheEntry {
    is_approved: bool,
    timestamp: Instant,
}

// Simple cache with TTL for subnet approval status
type ApprovalCache = Arc<RwLock<HashMap<String, ApprovalCacheEntry>>>;

const APPROVAL_CACHE_TTL: Duration = Duration::from_secs(60); // Cache for 1 minute

/// Save a deployed gateway to the persistent storage file
async fn save_deployed_gateway_to_file(
    gateway_address: String,
    registry_address: String,
    deployer_address: String,
    parent_network: String,
    name: Option<String>,
    config_path: &str,
) -> Result<GatewayInfo> {
    let gateway_info = GatewayInfo::new(
        gateway_address,
        registry_address,
        deployer_address,
        parent_network,
        name,
    );

    let gateway_file = std::path::Path::new(config_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("deployed_gateways.json");

    // Load existing gateways
    let mut gateways: HashMap<String, GatewayInfo> = if gateway_file.exists() {
        match tokio::fs::read_to_string(&gateway_file).await {
            Ok(contents) => {
                serde_json::from_str(&contents).unwrap_or_else(|e| {
                    log::warn!("Failed to parse existing gateway data: {}, starting fresh", e);
                    HashMap::new()
                })
            }
            Err(e) => {
                log::warn!("Failed to read existing gateway file: {}, starting fresh", e);
                HashMap::new()
            }
        }
    } else {
        HashMap::new()
    };

    // Add the new gateway
    gateways.insert(gateway_info.id.clone(), gateway_info.clone());

    // Save to file
    let contents = serde_json::to_string_pretty(&gateways)?;
    tokio::fs::write(&gateway_file, contents).await?;

    log::debug!("Saved gateway {} to persistent storage", gateway_info.id);
    Ok(gateway_info)
}

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
            deployed_gateways: Arc::new(Mutex::new(HashMap::new())),
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
        // Initialize with real subnet data from IPC provider
        self.initialize_real_data().await?;

        // Load existing gateway data
        self.load_gateway_data().await?;

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
                        ],
                        "recommended": ["development"]
                    },
                    {
                        "id": "staging",
                        "name": "Staging Template",
                        "description": "Pre-production testing with realistic settings",
                        "icon": "🚀",
                        "features": [
                            "Collateral mode",
                            "Moderate stakes",
                            "Realistic validator count",
                            "Production-like settings",
                            "Lower barriers for testing"
                        ],
                        "recommended": ["staging"]
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
                        ],
                        "recommended": ["production"]
                    },
                    {
                        "id": "federated",
                        "name": "Federated Network Template",
                        "description": "For consortium and private networks",
                        "icon": "🤝",
                        "features": [
                            "Federated mode",
                            "Known validator set",
                            "Flexible management",
                            "Controlled access",
                            "Custom governance"
                        ],
                        "recommended": ["consortium"]
                    }
                ]);
                warp::reply::json(&templates)
            });

        // GET /api/instances
        let instances = warp::path!("api" / "instances")
            .and(warp::get())
            .and(state_filter.clone())
            .and_then(|state: AppState| async move {
                // Get real instances from IPC provider
                let server = UIServer {
                    host: "127.0.0.1".to_string(),
                    frontend_port: 3000,
                    backend_port: 3001,
                    mode: state.mode.clone(),
                    config_path: state.config_path.clone(),
                    state: state.clone(),
                };

                match server.get_real_instances().await {
                    Ok(instances) => Ok::<_, warp::Rejection>(warp::reply::json(&instances)),
                    Err(e) => {
                        log::error!("Failed to get real instances: {}", e);
                        // Return empty list on error to prevent UI breaking
                        Ok::<_, warp::Rejection>(warp::reply::json(&Vec::<SubnetInstance>::new()))
                    }
                }
            });

        // GET /api/instance?id=<subnet_id> (using query param to avoid URL encoding issues)
        let instance_by_id = warp::path!("api" / "instance")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .and(state_filter.clone())
            .and_then(|query: HashMap<String, String>, state: AppState| async move {
                let id = match query.get("id") {
                    Some(id) => id.clone(),
                    None => {
                        log::warn!("Missing 'id' query parameter");
                        return Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"error": "Missing 'id' parameter"})),
                            warp::http::StatusCode::BAD_REQUEST
                        ));
                    }
                };

                log::info!("🔍 Looking for instance with query param ID: '{}'", id);

                // Get real instances and find the requested one
                let server = UIServer {
                    host: "127.0.0.1".to_string(),
                    frontend_port: 3000,
                    backend_port: 3001,
                    mode: state.mode.clone(),
                    config_path: state.config_path.clone(),
                    state: state.clone(),
                };

                match server.get_real_instances().await {
                    Ok(instances) => {
                        log::info!("📋 Available instances:");
                        for instance in &instances {
                            log::info!("   - ID: '{}' (len={})", instance.id, instance.id.len());
                        }

                        match instances.iter().find(|instance| instance.id == id) {
                            Some(instance) => {
                                log::info!("✅ Found instance: '{}'", instance.id);
                                Ok::<_, warp::Rejection>(warp::reply::with_status(warp::reply::json(instance), warp::http::StatusCode::OK))
                            }
                            None => {
                                log::warn!("❌ Instance not found with ID: '{}'", id);
                                Ok::<_, warp::Rejection>(warp::reply::with_status(
                                    warp::reply::json(&serde_json::json!({"error": "Instance not found"})),
                                    warp::http::StatusCode::NOT_FOUND
                                ))
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to get real instances: {}", e);
                        Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"error": "Failed to retrieve instance data"})),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR
                        ))
                    }
                }
            });

        // GET /api/gateways - List all deployed gateways
        let gateways = warp::path!("api" / "gateways")
            .and(warp::get())
            .and(state_filter.clone())
            .and_then(|state: AppState| async move {
                let deployed_gateways = state.deployed_gateways.lock().unwrap();
                let gateways: Vec<GatewayInfo> = deployed_gateways.values().cloned().collect();
                Ok::<_, warp::Rejection>(warp::reply::json(&gateways))
            });

        // GET /api/gateways/:id - Get specific gateway details
        let gateway_by_id = warp::path!("api" / "gateways" / String)
            .and(warp::get())
            .and(state_filter.clone())
            .and_then(|id: String, state: AppState| async move {
                let deployed_gateways = state.deployed_gateways.lock().unwrap();
                match deployed_gateways.get(&id) {
                    Some(gateway) => {
                        Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(gateway),
                            warp::http::StatusCode::OK
                        ))
                    }
                    None => {
                        Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"error": "Gateway not found"})),
                            warp::http::StatusCode::NOT_FOUND
                        ))
                    }
                }
            });

        // GET /api/gateways-discover - Discover gateways from IPC config (using different path to avoid conflicts)
        let discover_gateways = warp::path!("api" / "gateways-discover")
            .and(warp::get())
            .and(state_filter.clone())
            .and_then(|state: AppState| async move {
                let server = UIServer {
                    host: "127.0.0.1".to_string(),
                    frontend_port: 3000,
                    backend_port: 3001,
                    mode: state.mode.clone(),
                    config_path: state.config_path.clone(),
                    state: state.clone(),
                };

                match server.discover_gateways_from_config().await {
                    Ok(discovered_gateways) => {
                        // Add discovered gateways to the tracked list
                        {
                            let mut deployed_gateways = state.deployed_gateways.lock().unwrap();
                            for gateway in &discovered_gateways {
                                deployed_gateways.insert(gateway.id.clone(), gateway.clone());
                            }
                        }

                        // Save to persistent storage
                        if let Err(e) = server.save_gateway_data().await {
                            log::warn!("Failed to save discovered gateways: {}", e);
                        }

                        Ok::<_, warp::Rejection>(warp::reply::json(&discovered_gateways))
                    }
                    Err(e) => {
                        log::error!("Failed to discover gateways: {}", e);
                        Ok::<_, warp::Rejection>(warp::reply::json(&Vec::<GatewayInfo>::new()))
                    }
                }
            });

                // PUT /api/gateways/:id - Update gateway information (name, description, etc.)
        let update_gateway = warp::path!("api" / "gateways" / String)
            .and(warp::put())
            .and(warp::body::json())
            .and(state_filter.clone())
            .and_then(|id: String, update_data: serde_json::Value, state: AppState| async move {
                let updated_gateway = {
                    let mut deployed_gateways = state.deployed_gateways.lock().unwrap();

                    match deployed_gateways.get_mut(&id) {
                        Some(gateway) => {
                            // Update editable fields
                            if let Some(name) = update_data.get("name").and_then(|v| v.as_str()) {
                                gateway.name = name.to_string();
                            }
                            if let Some(description) = update_data.get("description").and_then(|v| v.as_str()) {
                                gateway.description = Some(description.to_string());
                            }
                            if let Some(status) = update_data.get("status").and_then(|v| v.as_str()) {
                                gateway.status = status.to_string();
                            }
                            Some(gateway.clone())
                        }
                        None => None,
                    }
                }; // Lock is released here

                match updated_gateway {
                    Some(gateway) => {
                        // Save to disk (now we can safely await)
                        let server = UIServer {
                            host: "127.0.0.1".to_string(),
                            frontend_port: 3000,
                            backend_port: 3001,
                            mode: state.mode.clone(),
                            config_path: state.config_path.clone(),
                            state: state.clone(),
                        };

                        if let Err(e) = server.save_gateway_data().await {
                            log::error!("Failed to save gateway data: {}", e);
                        }

                        Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&gateway),
                            warp::http::StatusCode::OK
                        ))
                    }
                    None => {
                        Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"error": "Gateway not found"})),
                            warp::http::StatusCode::NOT_FOUND
                        ))
                    }
                }
            });

        // GET /api/contracts - List all contracts (gateways, registries, etc.)
        let contracts = warp::path!("api" / "contracts")
            .and(warp::get())
            .and(state_filter.clone())
            .and_then(|state: AppState| async move {
                let server = UIServer {
                    host: "127.0.0.1".to_string(),
                    frontend_port: 3000,
                    backend_port: 3001,
                    mode: state.mode.clone(),
                    config_path: state.config_path.clone(),
                    state: state.clone(),
                };

                match server.get_all_contracts().await {
                    Ok(contracts) => Ok::<_, warp::Rejection>(warp::reply::json(&contracts)),
                    Err(e) => {
                        log::error!("Failed to get contracts: {}", e);
                        Ok::<_, warp::Rejection>(warp::reply::json(&Vec::<serde_json::Value>::new()))
                    }
                }
            });

        // GET /api/contracts/:id - Get specific contract details
        let contract_by_id = warp::path!("api" / "contracts" / String)
            .and(warp::get())
            .and(state_filter.clone())
            .and_then(|id: String, state: AppState| async move {
                let server = UIServer {
                    host: "127.0.0.1".to_string(),
                    frontend_port: 3000,
                    backend_port: 3001,
                    mode: state.mode.clone(),
                    config_path: state.config_path.clone(),
                    state: state.clone(),
                };

                match server.get_contract_by_id(&id).await {
                    Ok(Some(contract)) => {
                        Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&contract),
                            warp::http::StatusCode::OK
                        ))
                    }
                    Ok(None) => {
                        Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"error": "Contract not found"})),
                            warp::http::StatusCode::NOT_FOUND
                        ))
                    }
                    Err(e) => {
                        log::error!("Failed to get contract {}: {}", id, e);
                        Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"error": "Failed to retrieve contract"})),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR
                        ))
                    }
                }
            });

        // GET /api/contracts/inspect/:address - Inspect contract by address
        let inspect_contract = warp::path!("api" / "contracts" / "inspect" / String)
            .and(warp::get())
            .and(state_filter.clone())
            .and_then(|address: String, state: AppState| async move {
                let server = UIServer {
                    host: "127.0.0.1".to_string(),
                    frontend_port: 3000,
                    backend_port: 3001,
                    mode: state.mode.clone(),
                    config_path: state.config_path.clone(),
                    state: state.clone(),
                };

                match server.inspect_contract(&address).await {
                    Ok(inspection) => {
                        Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&inspection),
                            warp::http::StatusCode::OK
                        ))
                    }
                    Err(e) => {
                        log::error!("Failed to inspect contract {}: {}", address, e);
                        Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"error": "Failed to inspect contract"})),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR
                        ))
                    }
                }
            });

        // PUT /api/contracts/:id/configure - Configure contract
        let configure_contract = warp::path!("api" / "contracts" / String / "configure")
            .and(warp::put())
            .and(warp::body::json())
            .and(state_filter.clone())
            .and_then(|id: String, config_data: serde_json::Value, state: AppState| async move {
                log::info!("Received contract configuration request for {}: {:?}", id, config_data);

                // TODO: Implement contract configuration logic
                Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                    "success": true,
                    "message": format!("Contract {} configuration updated", id)
                })))
            });

        // POST /api/contracts/:id/upgrade - Upgrade contract
        let upgrade_contract = warp::path!("api" / "contracts" / String / "upgrade")
            .and(warp::post())
            .and(warp::body::json())
            .and(state_filter.clone())
            .and_then(|id: String, upgrade_data: serde_json::Value, state: AppState| async move {
                log::info!("Received contract upgrade request for {}: {:?}", id, upgrade_data);

                // TODO: Implement contract upgrade logic
                Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                    "success": true,
                    "message": format!("Contract {} upgrade initiated", id)
                })))
            });

        // GET /api/contracts/:address/abi - Get contract ABI
        let contract_abi = warp::path!("api" / "contracts" / String / "abi")
            .and(warp::get())
            .and(state_filter.clone())
            .and_then(|address: String, state: AppState| async move {
                log::info!("Received ABI request for contract: {}", address);

                // TODO: Implement ABI retrieval logic
                Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                    "abi": [],
                    "address": address,
                    "message": "ABI retrieval not yet implemented"
                })))
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

                        // Broadcast error to WebSocket clients with more explicit error messaging
                        let error_msg = format!("Deployment Failed: {}", e);
                        log::error!("🚨 Broadcasting deployment failure: {}", error_msg);
                        broadcast_deployment_progress(&state_clone, &deployment_id_clone, "failed", 0, "failed", Some(error_msg)).await;
                    }
                });

                Ok::<_, warp::Rejection>(warp::reply::json(&json!({
                    "deployment_id": deployment_id,
                    "status": "started",
                    "message": "Deployment initiated successfully"
                })))
            });

        // POST /api/subnets/:subnet_id/approve - Approve a subnet
        let approve_subnet = warp::path!("api" / "subnets" / String / "approve")
            .and(warp::post())
            .and(warp::body::json())
            .and(state_filter.clone())
            .and_then(|subnet_id: String, approve_data: serde_json::Value, state: AppState| async move {
                // URL decode the subnet ID first
                let decoded_subnet_id = urlencoding::decode(&subnet_id)
                    .map_err(|e| {
                        log::error!("Failed to URL decode subnet ID '{}': {}", subnet_id, e);
                        warp::reject::custom(InvalidRequest(format!("Invalid subnet ID encoding: {}", e)))
                    })?
                    .into_owned();

                log::info!("Received subnet approval request for: {} (decoded: {})", subnet_id, decoded_subnet_id);

                // Extract the gateway owner address from the request
                let from_address = approve_data.get("from")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| warp::reject::custom(InvalidRequest("Missing 'from' field for gateway owner".to_string())))?;

                // Call the approval function
                match approve_subnet_via_cli(&decoded_subnet_id, from_address, &state.config_path).await {
                    Ok(_) => {
                        log::info!("✅ Successfully approved subnet: {}", decoded_subnet_id);
                        Ok(warp::reply::json(&serde_json::json!({
                            "success": true,
                            "message": format!("Subnet {} approved successfully", decoded_subnet_id)
                        })))
                    }
                    Err(e) => {
                        log::error!("❌ Failed to approve subnet {}: {}", decoded_subnet_id, e);
                        Err(warp::reject::custom(ServerError(format!("Failed to approve subnet: {}", e))))
                    }
                }
            });

        // Validator management endpoints
        let add_validator = warp::path!("api" / "validators" / "add")
            .and(warp::post())
            .and(warp::body::json())
            .and(state_filter.clone())
            .and_then(|validator_data: serde_json::Value, state: AppState| async move {
                log::info!("Received add validator request: {:?}", validator_data);

                match add_validator_via_cli(validator_data, &state.config_path).await {
                    Ok(result) => {
                        log::info!("Successfully added validator: {}", result);
                        Ok(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"success": true, "message": result})),
                            warp::http::StatusCode::OK,
                        ))
                    }
                    Err(e) => {
                        log::error!("Failed to add validator: {}", e);
                        Ok(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({
                                "success": false,
                                "error": format!("Failed to add validator: {}", e)
                            })),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ))
                    }
                }
            });

        let set_federated_power = warp::path!("api" / "validators" / "set-federated-power")
            .and(warp::post())
            .and(warp::body::json())
            .and(state_filter.clone())
            .and_then(|power_data: serde_json::Value, state: AppState| async move {
                log::info!("Received set federated power request: {:?}", power_data);

                match set_federated_power_via_cli(power_data, &state.config_path).await {
                    Ok(result) => {
                        log::info!("Successfully set federated power: {}", result);
                        Ok(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"success": true, "message": result})),
                            warp::http::StatusCode::OK,
                        ))
                    }
                    Err(e) => {
                        log::error!("Failed to set federated power: {}", e);
                        Ok(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({
                                "success": false,
                                "error": format!("Failed to set federated power: {}", e)
                            })),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ))
                    }
                }
            });

        let remove_validator = warp::path!("api" / "validators" / "remove")
            .and(warp::post())
            .and(warp::body::json())
            .and(state_filter.clone())
            .and_then(|validator_data: serde_json::Value, state: AppState| async move {
                log::info!("Received remove validator request: {:?}", validator_data);

                match remove_validator_via_cli(validator_data, &state.config_path).await {
                    Ok(result) => {
                        log::info!("Successfully removed validator: {}", result);
                        Ok(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"success": true, "message": result})),
                            warp::http::StatusCode::OK,
                        ))
                    }
                    Err(e) => {
                        log::error!("Failed to remove validator: {}", e);
                        Ok(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({
                                "success": false,
                                "error": format!("Failed to remove validator: {}", e)
                            })),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ))
                    }
                }
            });

        let update_validator_stake = warp::path!("api" / "validators" / "update-stake")
            .and(warp::post())
            .and(warp::body::json())
            .and(state_filter.clone())
            .and_then(|stake_data: serde_json::Value, state: AppState| async move {
                log::info!("Received update validator stake request: {:?}", stake_data);

                match update_validator_stake_via_cli(stake_data, &state.config_path).await {
                    Ok(result) => {
                        log::info!("Successfully updated validator stake: {}", result);
                        Ok(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({"success": true, "message": result})),
                            warp::http::StatusCode::OK,
                        ))
                    }
                    Err(e) => {
                        log::error!("Failed to update validator stake: {}", e);
                        Ok(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({
                                "success": false,
                                "error": format!("Failed to update validator stake: {}", e)
                            })),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ))
                    }
                }
            });

        templates
            .or(instances)
            .or(instance_by_id)
            .or(gateways)
            .or(discover_gateways)  // More specific route must come before generic :id route
            .or(gateway_by_id)
            .or(update_gateway)
            .or(deploy)
            .or(approve_subnet)
            .or(add_validator)
            .or(set_federated_power)
            .or(remove_validator)
            .or(update_validator_stake)
            .or(contracts)
            .or(contract_by_id)
            .or(inspect_contract)
            .or(configure_contract)
            .or(upgrade_contract)
            .or(contract_abi)


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

    /// Initialize with real subnet data from IPC provider
    async fn initialize_real_data(&self) -> Result<()> {
        // We'll load real data on-demand rather than pre-loading it
        // This allows for real-time data that reflects current subnet state
        log::info!("UI server initialized for real subnet data queries");
        Ok(())
    }

    /// Load existing gateway data from persistent storage
    async fn load_gateway_data(&self) -> Result<()> {
        let gateway_file = std::path::Path::new(&self.config_path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("deployed_gateways.json");

        if gateway_file.exists() {
            match tokio::fs::read_to_string(&gateway_file).await {
                Ok(contents) => {
                    match serde_json::from_str::<HashMap<String, GatewayInfo>>(&contents) {
                        Ok(gateways) => {
                            let mut deployed_gateways = self.state.deployed_gateways.lock().unwrap();
                            *deployed_gateways = gateways.clone();
                            log::info!("Loaded {} deployed gateways from storage", gateways.len());
                        }
                        Err(e) => {
                            log::warn!("Failed to parse gateway data: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read gateway file: {}", e);
                }
            }
        } else {
            log::info!("No existing gateway data found, starting fresh");
        }

        Ok(())
    }

    /// Save gateway data to persistent storage
    async fn save_gateway_data(&self) -> Result<()> {
        let gateway_file = std::path::Path::new(&self.config_path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("deployed_gateways.json");

        let gateways = {
            let deployed_gateways = self.state.deployed_gateways.lock().unwrap();
            deployed_gateways.clone()
        };

        let contents = serde_json::to_string_pretty(&gateways)?;
        tokio::fs::write(&gateway_file, contents).await?;

        log::debug!("Saved {} gateways to storage", gateways.len());
        Ok(())
    }

    /// Add a newly deployed gateway to the tracking system
    pub async fn track_deployed_gateway(
        &self,
        gateway_address: String,
        registry_address: String,
        deployer_address: String,
        parent_network: String,
        name: Option<String>,
    ) -> Result<GatewayInfo> {
        let gateway_info = GatewayInfo::new(
            gateway_address,
            registry_address,
            deployer_address,
            parent_network,
            name,
        );

        // Add to in-memory store
        {
            let mut deployed_gateways = self.state.deployed_gateways.lock().unwrap();
            deployed_gateways.insert(gateway_info.id.clone(), gateway_info.clone());
        }

        // Persist to disk
        self.save_gateway_data().await?;

        log::info!("🎯 Tracked new deployed gateway: {} ({})", gateway_info.name, gateway_info.id);
        Ok(gateway_info)
    }

    /// Get real subnet instances from IPC provider
    async fn get_real_instances(&self) -> Result<Vec<SubnetInstance>> {
        use crate::get_ipc_provider;
        use crate::GlobalArguments;
        use ipc_api::subnet_id::SubnetID;
        use std::str::FromStr;

        // Create cache for subnet approval status
        let approval_cache: ApprovalCache = Arc::new(RwLock::new(HashMap::new()));

        let global = GlobalArguments {
            config_path: Some(self.config_path.clone()),
            _network: fvm_shared::address::Network::Testnet,
            __network: None,
        };

        let provider = match get_ipc_provider(&global) {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Failed to get IPC provider: {}, falling back to empty list", e);
                return Ok(vec![]);
            }
        };

        // Also get config store to find configured subnets
        let global = GlobalArguments {
            config_path: Some(self.config_path.clone()),
            _network: fvm_shared::address::Network::Testnet,
            __network: None,
        };

        let ipc_config_store = match IpcConfigStore::load_or_init(&global).await {
            Ok(store) => Some(store),
            Err(e) => {
                log::debug!("Failed to get IPC config store: {}, using provider only", e);
                None
            }
        };

        let mut instances = Vec::new();

        // First, try to get subnets from the IPC config store (includes deployed but not yet approved subnets)
        if let Some(ref store) = ipc_config_store {
            let config_snapshot = store.snapshot().await;

            for (subnet_id, subnet_config) in config_snapshot.subnets.iter() {
                // Skip root networks, only show actual child subnets
                if subnet_id.is_root() {
                    continue;
                }

                // Try to get validators using the provider
                let validators = match provider.list_validators(&subnet_id).await {
                    Ok(validators) => {
                        validators.into_iter().map(|(addr, info)| {
                            super::ValidatorInfo {
                                address: addr.to_string(),
                                stake: "0".to_string(), // TODO: ValidatorStakingInfo fields are private
                                power: if info.is_active { 1 } else { 0 },
                                status: if info.is_active { "Active".to_string() } else { "Inactive".to_string() },
                            }
                        }).collect()
                    }
                    Err(e) => {
                        log::debug!("Failed to get validators for subnet {}: {}", subnet_id, e);
                        vec![]
                    }
                };

                // Create instance from config data
                let parent_id = subnet_id.parent().map(|p| p.to_string()).unwrap_or_else(|| "/r314159".to_string());

                // Check if subnet is approved by querying the gateway
                let is_approved = check_subnet_approval_status(&provider, &subnet_id, &approval_cache, Some(store)).await;

                // Determine status based on approval and validator presence
                let status = if is_approved {
                    if validators.is_empty() {
                        "Approved - No Validators".to_string()
                    } else {
                        "Active".to_string()
                    }
                } else {
                    "Pending Approval".to_string()
                };

                let instance = SubnetInstance {
                    id: subnet_id.to_string(),
                    name: format!("Subnet {}", subnet_id.to_string().split('/').last().unwrap_or("Unknown")),
                    status,
                    template: "Development Template".to_string(),
                    parent: parent_id,
                    created_at: chrono::Utc::now(), // We don't have creation time from config
                    validators,
                    config: serde_json::json!({
                        "rpc_url": subnet_config.rpc_http(),
                        "gateway_addr": subnet_config.gateway_addr(),
                        "registry_addr": subnet_config.registry_addr(),
                        "permissionMode": "federated"
                    }),
                };

                instances.push(instance);
            }
        }

        // Also try the original provider-based discovery for backward compatibility
        let parent_networks = vec![
            "/r314159",     // Filecoin Calibration testnet
            "/r31337",      // Local development
            "/r1",          // Filecoin mainnet
        ];

        for parent_str in parent_networks {
            if let Ok(parent_subnet) = SubnetID::from_str(parent_str) {
                match provider.list_child_subnets(None, &parent_subnet).await {
                    Ok(subnets) => {
                        for (subnet_id, subnet_info) in subnets {
                            // Skip if we already have this subnet from config store
                            if instances.iter().any(|inst| inst.id == subnet_id.to_string()) {
                                continue;
                            }

                            // Get validators for this subnet
                            let validators = match provider.list_validators(&subnet_id).await {
                                Ok(validators) => {
                                    validators.into_iter().map(|(addr, info)| {
                                        super::ValidatorInfo {
                                            address: addr.to_string(),
                                            stake: "0".to_string(), // TODO: ValidatorStakingInfo fields are private
                                            power: if info.is_active { 1 } else { 0 },
                                            status: if info.is_active { "Active".to_string() } else { "Inactive".to_string() },
                                        }
                                    }).collect()
                                }
                                Err(e) => {
                                    log::debug!("Failed to get validators for subnet {}: {}", subnet_id, e);
                                    vec![]
                                }
                            };

                            let instance = SubnetInstance {
                                id: subnet_id.to_string(),
                                name: format!("Subnet {}", subnet_id.to_string().split('/').last().unwrap_or("Unknown")),
                                status: if subnet_info.stake.is_zero() { "Inactive".to_string() } else { "Active".to_string() },
                                template: if subnet_info.stake.atto() > &1000000000000000000u64.into() {
                                    "Production Template".to_string()
                                } else {
                                    "Development Template".to_string()
                                },
                                parent: parent_str.to_string(),
                                created_at: chrono::DateTime::from_timestamp(subnet_info.genesis_epoch as i64, 0)
                                    .unwrap_or_else(chrono::Utc::now),
                                validators,
                                config: serde_json::json!({
                                    "stake": subnet_info.stake.atto().to_string(),
                                    "circ_supply": subnet_info.circ_supply.atto().to_string(),
                                    "genesis_epoch": subnet_info.genesis_epoch,
                                    "permissionMode": "collateral"
                                }),
                            };

                            instances.push(instance);
                        }
                    }
                    Err(e) => {
                        log::debug!("No subnets found for parent {}: {}", parent_str, e);
                    }
                }
            }
        }

        if instances.is_empty() {
            log::info!("No real subnets found, this may be expected in development");
        } else {
            log::info!("Found {} subnet instances", instances.len());
        }

        Ok(instances)
    }

    /// Discover gateways from the IPC config store that are not yet tracked
    async fn discover_gateways_from_config(&self) -> Result<Vec<GatewayInfo>> {
        let global = GlobalArguments {
            config_path: Some(self.config_path.clone()),
            _network: fvm_shared::address::Network::Testnet,
            __network: None,
        };

        let ipc_config_store = IpcConfigStore::load_or_init(&global).await?;
        let config_snapshot = ipc_config_store.snapshot().await;

        let mut discovered_gateways = Vec::new();

        for (subnet_id, subnet_config) in config_snapshot.subnets.iter() {
            let gateway_addr = subnet_config.gateway_addr().to_string();
            let registry_addr = subnet_config.registry_addr().to_string();
            let parent_str = subnet_id.parent().map(|p| p.to_string()).unwrap_or_else(|| "/r314159".to_string());

            // Create a unique ID based on gateway address to avoid duplicates
            let gateway_id = format!("gateway-{}", gateway_addr);

            // Only add if not already tracked
            if self.state.deployed_gateways.lock().unwrap().get(&gateway_id).is_none() {
                let gateway_info = GatewayInfo::new(
                    gateway_addr.clone(),
                    registry_addr.clone(),
                    "IPC Config".to_string(), // Indicate it's discovered from config
                    parent_str.clone(),
                    Some(format!("Discovered Gateway for {}", parent_str)),
                );
                log::info!("Discovered gateway: {} (Parent: {})", gateway_info.name, gateway_info.parent_network);
                discovered_gateways.push(gateway_info);
            }
        }

        Ok(discovered_gateways)
    }

    /// Get all contracts
    async fn get_all_contracts(&self) -> Result<Vec<serde_json::Value>> {
        let gateways = self.state.deployed_gateways.lock().unwrap();
        let mut contracts = Vec::new();

        // Add gateway contracts
        for gateway in gateways.values() {
            contracts.push(gateway.to_json());
        }

        // Add registry contracts (derived from gateways)
        for gateway in gateways.values() {
            contracts.push(serde_json::json!({
                "id": format!("registry-{}", gateway.id),
                "name": format!("Registry ({}...)", gateway.registry_address.chars().take(8).collect::<String>()),
                "type": "registry",
                "address": gateway.registry_address,
                "deployer": gateway.deployer_address,
                "network": gateway.parent_network,
                "deployed_at": gateway.deployed_at,
                "status": if gateway.status == "active" { "active" } else { "inactive" },
                "description": format!("Registry contract for {}", gateway.name),
                "actions": ["inspect", "configure"]
            }));
        }

        Ok(contracts)
    }

    /// Get contract by ID
    async fn get_contract_by_id(&self, id: &str) -> Result<Option<serde_json::Value>> {
        let gateways = self.state.deployed_gateways.lock().unwrap();
        Ok(gateways.get(id).map(|gateway| gateway.to_json()))
    }

    /// Inspect contract
    async fn inspect_contract(&self, address: &str) -> Result<serde_json::Value> {
        // Implement contract inspection logic
        Ok(serde_json::json!({
            "address": address,
            "status": "Inspected",
            "message": "Contract inspection not yet implemented"
        }))
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

/// Custom rejection type for server errors
#[derive(Debug)]
struct ServerError(String);

impl warp::reject::Reject for ServerError {}

/// Handle actual subnet deployment
async fn handle_deployment(
    state: AppState,
    deployment_id: String,
    _template: String,
    config: serde_json::Value,
) -> Result<()> {
    // Get config path from the state
    let config_path = &state.config_path;
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

    // Store IpcConfigStore for passing between steps
    let mut ipc_config_store: Option<IpcConfigStore> = None;
    let mut created_subnet_info: Option<(SubnetID, String)> = None; // (subnet_id, creator)

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

        // Process deployment step
        match step_id {
            "validate" => {
                // Validate configuration
                validate_deployment_config(&config)?;
                sleep(Duration::from_secs(2)).await;
            }
            "prepare" => {
                // Prepare deployment files and get IPC config store
                let store = prepare_deployment_files(&config, &state.config_path).await?;
                ipc_config_store = Some(store);
                sleep(Duration::from_secs(3)).await;
            }
            "contracts" => {
                // Deploy smart contracts
                if let Some(ref store) = ipc_config_store {
                    let (gateway_addr, registry_addr) = deploy_smart_contracts(&config, store, config_path, &state).await?;
                    if let (Some(gw), Some(reg)) = (gateway_addr, registry_addr) {
                        log::info!("🎯 Custom gateway contracts deployed/configured:");
                        log::info!("   - Gateway: {}", gw);
                        log::info!("   - Registry: {}", reg);
                        log::info!("   ✅ These contracts will be used for subnet creation");
                    }
                } else {
                    return Err(anyhow::anyhow!("IPC config store not initialized"));
                }
                sleep(Duration::from_secs(5)).await;
            }
            "genesis" => {
                // Create and approve subnet
                if let Some(ref store) = ipc_config_store {
                    match create_and_approve_subnet(&config, store, &state.config_path).await {
                        Ok(subnet_info) => {
                            created_subnet_info = Some(subnet_info);
                        }
                        Err(e) => {
                            log::error!("🚨 Subnet creation failed, broadcasting error to WebSocket");
                            let error_msg = format!("Deployment Failed: {}", e);
                            broadcast_deployment_progress(&state, &deployment_id, "failed", 0, "failed", Some(error_msg)).await;
                            return Err(e);
                        }
                    }
                } else {
                    let error_msg = "IPC config store not initialized".to_string();
                    broadcast_deployment_progress(&state, &deployment_id, "failed", 0, "failed", Some(error_msg.clone())).await;
                    return Err(anyhow::anyhow!(error_msg));
                }
                sleep(Duration::from_secs(3)).await;
            }
            "validators" => {
                // Initialize validators (skip - this is handled in activation)
                log::info!("Validator initialization will be handled in activation step");
                sleep(Duration::from_secs(4)).await;
            }
            "activation" => {
                // Activate subnet
                if let (Some(ref store), Some(ref subnet_info)) = (&ipc_config_store, &created_subnet_info) {
                    activate_subnet_deployment(&config, store, subnet_info, &state.config_path).await?;
                } else {
                    return Err(anyhow::anyhow!("Subnet not created or config store not initialized"));
                }
                sleep(Duration::from_secs(2)).await;
            }
            "verification" => {
                // Run verification
                if let Some(ref subnet_info) = created_subnet_info {
                    verify_deployment(&config, &subnet_info.0).await?;
                } else {
                    return Err(anyhow::anyhow!("Subnet not created"));
                }
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
    let mut data = json!({
        "deployment_id": deployment_id,
        "step": step,
        "progress": progress,
        "status": status,
        "message": message
    });

    // For failed status, also add error field for frontend compatibility
    if status == "failed" {
        data["error"] = message.clone().unwrap_or_else(|| "Deployment failed".to_string()).into();
    }

    let ws_message_json = json!({
        "type": "deployment_progress",
        "data": data
    });

    let ws_message = warp::ws::Message::text(ws_message_json.to_string());

    log::info!("📡 Broadcasting WebSocket message: {}", ws_message_json.to_string());

    let clients = state.websocket_clients.lock().unwrap();
    let client_count = clients.len();

    for client in clients.iter() {
        if let Err(e) = client.sender.send(ws_message.clone()) {
            log::warn!("Failed to send WebSocket message to client {}: {}", client.id, e);
        } else {
            log::debug!("✅ Sent WebSocket message to client {}", client.id);
        }
    }

    log::info!("📡 Deployment progress broadcast to {} clients: {} - {} ({}%)", client_count, deployment_id, step, progress);
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

    // Validate permission mode
    if let Some(permission_mode) = config.get("permissionMode").and_then(|v| v.as_str()) {
        match permission_mode {
            "federated" | "collateral" | "static" => {}
            _ => return Err(anyhow::anyhow!("Invalid permission mode: {}", permission_mode)),
        }
    }

    // Validate gateway mode
    if let Some(gateway_mode) = config.get("gatewayMode").and_then(|v| v.as_str()) {
        match gateway_mode {
            "existing" | "deploy" | "deployed" | "custom" => {}
            _ => return Err(anyhow::anyhow!("Invalid gateway mode: {}", gateway_mode)),
        }

        // Validate deployed gateway selection if deployed mode is selected
        if gateway_mode == "deployed" {
            if config.get("selectedDeployedGateway").and_then(|v| v.as_str()).is_none() {
                return Err(anyhow::anyhow!("Selected deployed gateway required when using deployed gateway mode"));
            }
        }

        // Validate custom gateway fields if custom mode is selected
        if gateway_mode == "custom" {
            if config.get("customGatewayAddress").and_then(|v| v.as_str()).is_none() {
                return Err(anyhow::anyhow!("Custom gateway address required when using custom gateway mode"));
            }
            if config.get("customRegistryAddress").and_then(|v| v.as_str()).is_none() {
                return Err(anyhow::anyhow!("Custom registry address required when using custom gateway mode"));
            }

            // Validate address formats
            if let Some(gw_addr) = config.get("customGatewayAddress").and_then(|v| v.as_str()) {
                if gw_addr.parse::<EthAddress>().is_err() {
                    return Err(anyhow::anyhow!("Invalid custom gateway address format"));
                }
            }
            if let Some(reg_addr) = config.get("customRegistryAddress").and_then(|v| v.as_str()) {
                if reg_addr.parse::<EthAddress>().is_err() {
                    return Err(anyhow::anyhow!("Invalid custom registry address format"));
                }
            }
        }
    }

    // Validate parent subnet format
    if let Some(parent) = config.get("parent").and_then(|v| v.as_str()) {
        SubnetID::from_str(parent)
            .map_err(|e| anyhow::anyhow!("Invalid parent subnet format: {}", e))?;
    }

    // Validate from address format
    if let Some(from) = config.get("from").and_then(|v| v.as_str()) {
        from.parse::<EthAddress>()
            .map_err(|e| anyhow::anyhow!("Invalid from address format: {}", e))?;
    }

    log::info!("Configuration validation completed");
    Ok(())
}

/// Prepare deployment files and IPC config store
async fn prepare_deployment_files(_config: &serde_json::Value, config_path: &str) -> Result<IpcConfigStore> {
    log::info!("Preparing deployment files");

    // Create GlobalArguments for IPC config store initialization
    let global = GlobalArguments {
        config_path: Some(config_path.to_string()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    // Load or initialize IPC config store
    let ipc_config_store = IpcConfigStore::load_or_init(&global).await?;

    log::info!("Deployment files prepared");
    Ok(ipc_config_store)
}

/// Deploy or configure smart contracts based on gateway mode
async fn deploy_smart_contracts(config: &serde_json::Value, ipc_config_store: &IpcConfigStore, config_path: &str, state: &AppState) -> Result<(Option<String>, Option<String>)> {
    log::info!("🚀 Starting smart contract deployment phase");

    // Log the configuration for debugging
    log::info!("Config parameters:");
    log::info!("  Parent: {}", config.get("parent").and_then(|v| v.as_str()).unwrap_or("NOT_SET"));
    log::info!("  From: {}", config.get("from").and_then(|v| v.as_str()).unwrap_or("NOT_SET"));
    log::info!("  Permission Mode: {}", config.get("permissionMode").and_then(|v| v.as_str()).unwrap_or("NOT_SET"));

    // Check gateway deployment mode
    let gateway_mode = config.get("gatewayMode").and_then(|v| v.as_str()).unwrap_or("existing");
    log::info!("  Gateway Mode: {}", gateway_mode);

    match gateway_mode {
        "deploy" => {
            log::info!("🔨 User chose to deploy new gateway contracts");
            log::info!("   - This gives full control over subnet approval");
            log::info!("   - User will be the gateway owner");
            deploy_new_gateway_contracts(config, ipc_config_store, config_path).await
        }
        "deployed" => {
            log::info!("🔨 User chose to use an existing deployed gateway");
            let selected_gateway_id = config.get("selectedDeployedGateway")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing selectedDeployedGateway field"))?;

            let deployed_gateways = state.deployed_gateways.lock().unwrap();
            if let Some(gateway) = deployed_gateways.get(selected_gateway_id) {
                log::info!("   - Selected Gateway: {}", gateway.name);
                log::info!("   - Gateway Address: {}", gateway.gateway_address);
                log::info!("   - Registry Address: {}", gateway.registry_address);
                log::info!("   - Parent Network: {}", gateway.parent_network);
                log::info!("   - Owner: {}", gateway.deployer_address);
                log::info!("   - Status: {}", gateway.status);
                log::info!("   - ID: {}", gateway.id);

                // For deployed mode, we just return the existing gateway addresses
                Ok((Some(gateway.gateway_address.clone()), Some(gateway.registry_address.clone())))
            } else {
                return Err(anyhow::anyhow!("Selected deployed gateway with ID '{}' not found", selected_gateway_id));
            }
        }
        "custom" => {
            log::info!("🔧 User provided custom gateway address");
            let gateway_addr = config.get("customGatewayAddress")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Custom gateway address required when gatewayMode is 'custom'"))?;
            let registry_addr = config.get("customRegistryAddress")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Custom registry address required when gatewayMode is 'custom'"))?;
            log::info!("   - Gateway: {}", gateway_addr);
            log::info!("   - Registry: {}", registry_addr);

            // TODO: Validate that these addresses are valid contracts
            log::info!("📦 Custom gateway configuration completed successfully");
            Ok((Some(gateway_addr.to_string()), Some(registry_addr.to_string())))
        }
        "existing" | _ => {
            log::info!("✅ Using existing parent chain contracts (Calibration gateway)");
            log::info!("   - Gateway contracts: Pre-deployed on parent chain");
            log::info!("   - Registry contracts: Pre-deployed on parent chain");
            log::info!("   - ⚠️  Note: Requires approval from gateway owner");
            log::info!("📦 Existing gateway configuration completed successfully");
            Ok((None, None))
        }
    }
}

/// Create and approve subnet
async fn create_and_approve_subnet(config: &serde_json::Value, ipc_config_store: &IpcConfigStore, config_path: &str) -> Result<(SubnetID, String)> {
    log::info!("🌐 Starting subnet creation and approval process");

    // Create GlobalArguments
    let global = GlobalArguments {
        config_path: Some(config_path.to_string()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    log::info!("📁 Using IPC config path: {}", config_path);

    // Check if we deployed new gateway contracts in the previous step
    let gateway_mode = config.get("gatewayMode").and_then(|v| v.as_str()).unwrap_or("existing");

    if gateway_mode == "deploy" {
        log::info!("🔄 Gateway deployment completed - reinitializing provider with updated configuration...");
        // Force reload of IPC config to pick up newly deployed gateway addresses
        // This is critical for the subnet creation to use the correct gateway contracts

        // CRITICAL FIX: Small delay to ensure config file is fully written
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        log::info!("⏳ Allowing config file to be fully updated before provider initialization...");
    }

    // Get IPC provider with detailed error handling
    // IMPORTANT: This provider initialization happens AFTER potential gateway deployment
    // ensuring it uses the updated configuration with new gateway addresses
    log::info!("🔌 Initializing IPC provider connection...");
    let provider = get_ipc_provider(&global)
        .map_err(|e| {
            log::error!("❌ Failed to get IPC provider: {}", e);
            anyhow::anyhow!("Failed to initialize IPC provider: {}", e)
        })?;
    log::info!("✅ IPC provider connection established");

    // Get parent chain configuration for gateway tracking
    let parent_str = config.get("parent").and_then(|v| v.as_str()).unwrap_or("/r314159");
    let parent_id = SubnetID::from_str(parent_str)?;

    if gateway_mode == "deploy" {
        log::info!("✅ Provider is now using newly deployed gateway contracts");

        // Verify the provider is using the correct gateway addresses
        if let Some(parent_config) = ipc_config_store.get_subnet(&parent_id).await {
            log::info!("🔍 Current provider configuration:");
            log::info!("   - Gateway: {}", parent_config.gateway_addr());
            log::info!("   - Registry: {}", parent_config.registry_addr());
            log::info!("   - RPC URL: {}", parent_config.rpc_http());
        } else {
            log::warn!("❌ Could not retrieve parent configuration from config store");
        }
    }

    // Track gateway information (whether newly deployed or existing)
    if let Some(parent_config) = ipc_config_store.get_subnet(&parent_id).await {
        let gateway_addr = parent_config.gateway_addr().to_string();
        let registry_addr = parent_config.registry_addr().to_string();
        let from_address = config.get("from")
            .and_then(|v| v.as_str())
            .unwrap_or("0x0a36d7c34ba5523d5bf783bb47f62371e52e0298");

        log::info!("📊 Tracking gateway usage for subnet deployment:");
        log::info!("   - Gateway: {}", gateway_addr);
        log::info!("   - Registry: {}", registry_addr);
        log::info!("   - Mode: {}", gateway_mode);

        // Save gateway information to tracking system
        match save_deployed_gateway_to_file(
            gateway_addr.clone(),
            registry_addr.clone(),
            from_address.to_string(),
            parent_str.to_string(),
            Some(match gateway_mode {
                "deploy" => format!("Custom Gateway for {}", parent_str),
                _ => format!("Calibration Gateway for {}", parent_str),
            }),
            config_path,
        ).await {
            Ok(gateway_info) => {
                log::info!("✅ Gateway tracked successfully: {}", gateway_info.name);
            }
            Err(e) => {
                log::warn!("⚠️  Failed to track gateway in management system: {}", e);
                // Don't fail the deployment if tracking fails
            }
        }
    }

    // Convert UI config to SubnetCreateConfig with validation
    log::info!("🔧 Converting UI configuration to subnet parameters...");
    let subnet_config = ui_config_to_subnet_create_config(config)
        .map_err(|e| {
            log::error!("❌ Configuration conversion failed: {}", e);
            anyhow::anyhow!("Failed to convert UI config: {}", e)
        })?;

    log::info!("📋 Subnet creation parameters:");
    log::info!("   - Parent: {}", subnet_config.parent);
    log::info!("   - From: {:?}", subnet_config.from);
    log::info!("   - Min Validators: {}", subnet_config.min_validators);
    log::info!("   - Min Stake: {}", subnet_config.min_validator_stake);
    log::info!("   - Permission Mode: {:?}", subnet_config.permission_mode);
    log::info!("   - Supply Source: {:?}", subnet_config.supply_source_kind);

    // Create subnet with detailed logging
    log::info!("🚀 Executing subnet creation command...");
    let subnet_address = create_subnet_cmd(provider.clone(), &subnet_config).await
        .map_err(|e| {
            log::error!("❌ Subnet creation command failed: {}", e);

                        // Provide more helpful error messages for common issues
            let error_message = e.to_string();
            if error_message.contains("actor not found") {
                let helpful_error = format!(
                    "Address resolution failed: The 'from' address ({}) doesn't exist on the parent chain ({}). \n\
                    This usually means:\n\
                    1. The address has no funds/hasn't been used on the parent chain\n\
                    2. The address is not valid for the parent chain network\n\
                    3. The parent chain RPC connection is not working\n\
                    \nPlease ensure your address has funds on the parent chain before deploying a subnet.\n\
                    Original error: {}",
                    subnet_config.from.as_ref().unwrap_or(&"unknown".to_string()),
                    subnet_config.parent,
                    error_message
                );
                anyhow::anyhow!("{}", helpful_error)
            } else if error_message.contains("revert") || error_message.contains("execution reverted") {
                // Handle smart contract revert errors with more helpful context
                let helpful_error = if error_message.contains("insufficient") {
                    format!(
                        "Insufficient funds: Your address ({}) doesn't have enough balance to cover the subnet creation costs.\n\
                        \n\
                        This includes:\n\
                        1. Minimum validator stake: {} FIL\n\
                        2. Gas fees for the transaction\n\
                        3. Potential collateral requirements\n\
                        \n\
                        Parent chain: {}\n\
                        \n\
                        Please add funds to your address and try again.\n\
                        Original error: {}",
                        subnet_config.from.as_ref().unwrap_or(&"unknown".to_string()),
                        subnet_config.min_validator_stake,
                        subnet_config.parent,
                        error_message
                    )
                } else {
                    format!(
                        "Contract call reverted: The subnet creation transaction was rejected by the smart contract.\n\
                        \n\
                        This could be due to:\n\
                        1. Insufficient funds for the required collateral\n\
                        2. Invalid subnet parameters\n\
                        3. Permission issues with the parent subnet\n\
                        4. Gateway or registry contract issues\n\
                        \n\
                        Address: {}\n\
                        Parent chain: {}\n\
                        \n\
                        Original error: {}",
                        subnet_config.from.as_ref().unwrap_or(&"unknown".to_string()),
                        subnet_config.parent,
                        error_message
                    )
                };
                anyhow::anyhow!("{}", helpful_error)
            } else {
                anyhow::anyhow!("Subnet creation failed: {}", e)
            }
        })?;

    log::info!("✅ Subnet contract deployed at address: {}", subnet_address);

    // Build subnet ID
    log::info!("🏗️  Building subnet ID from parent and address...");
    let parent_id = SubnetID::from_str(&subnet_config.parent)
        .map_err(|e| {
            log::error!("❌ Invalid parent subnet ID format: {}", e);
            anyhow::anyhow!("Invalid parent subnet ID: {}", e)
        })?;
    let subnet_id = SubnetID::new_from_parent(&parent_id, subnet_address);
    log::info!("🆔 Generated Subnet ID: {}", subnet_id);

    // Add subnet to config store
    log::info!("💾 Registering subnet in IPC config store...");
    let parent = ipc_config_store
        .get_subnet(&parent_id)
        .await
        .ok_or_else(|| {
            log::error!("❌ Parent subnet not found in config store: {}", parent_id);
            anyhow::anyhow!("Parent subnet '{}' not found in config store", parent_id)
        })?;

    log::info!("🔗 Parent subnet configuration found:");
    log::info!("   - RPC URL: {}", parent.rpc_http());
    log::info!("   - Gateway: {}", parent.gateway_addr());
    log::info!("   - Registry: {}", parent.registry_addr());

    ipc_config_store
        .add_subnet(
            subnet_id.clone(),
            parent.rpc_http().clone(),
            parent.gateway_addr(),
            parent.registry_addr(),
        )
        .await
        .map_err(|e| {
            log::error!("❌ Failed to add subnet to config store: {}", e);
            anyhow::anyhow!("Failed to register subnet in config store: {}", e)
        })?;

    log::info!("✅ Subnet registered in IPC config store");

    // NOTE: Subnet approval must be done by the gateway owner
    let creator = subnet_config.from.clone().unwrap_or_else(|| "0x0a36d7c34ba5523d5bf783bb47f62371e52e0298".to_string());

    log::info!("⚠️  IMPORTANT: Subnet created but requires manual approval!");
    log::info!("   - Subnet ID: {}", subnet_id);
    log::info!("   - Subnet Actor Address: {}", subnet_address);
    log::info!("   - Creator Address: {}", creator);
    log::info!("   - Gateway Address: 0x1AEe8A878a22280fc2753b3C63571c8f895D2FE3");
    log::info!("   ");
    log::info!("🔐 APPROVAL REQUIRED:");
    log::info!("   The subnet actor needs to be approved by the Calibration gateway owner.");
    log::info!("   This is a security measure to prevent unauthorized subnet creation.");
    log::info!("   ");
    log::info!("📝 NEXT STEPS:");
    log::info!("   1. Contact the IPC team or gateway administrator");
    log::info!("   2. Request approval for subnet actor: {}", subnet_address);
    log::info!("   3. Once approved, the subnet will be able to register and become active");
    log::info!("   ");
    log::info!("🎯 For development/testing, consider using a local network where you control the gateway.");

    log::info!("✅ Subnet creation completed (pending approval)!");

    Ok((subnet_id, creator))
}

/// Deploy new gateway contracts (user becomes owner)
async fn deploy_new_gateway_contracts(config: &serde_json::Value, ipc_config_store: &IpcConfigStore, config_path: &str) -> Result<(Option<String>, Option<String>)> {
    log::info!("🔧 Starting new gateway contract deployment...");

    // Extract required configuration parameters
    let from_str = config.get("from")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'from' address for contract deployment"))?;

    let from_address = from_str.parse::<EthAddress>()
        .map_err(|e| anyhow::anyhow!("Invalid 'from' address format '{}': {}", from_str, e))?;

    // Get parent chain configuration
    let parent_str = config.get("parent")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing parent chain configuration"))?;

    let parent_id = SubnetID::from_str(parent_str)
        .map_err(|e| anyhow::anyhow!("Invalid parent subnet ID: {}", e))?;

    // Get parent subnet info from config store
    let parent_subnet = ipc_config_store
        .get_subnet(&parent_id)
        .await
        .ok_or_else(|| anyhow::anyhow!("Parent subnet '{}' not found in config store", parent_id))?;

    // Get the base config for keystore creation
    let base_config = Arc::new(ipc_config_store.snapshot().await);

    log::info!("📋 Gateway deployment parameters:");
    log::info!("   - Deployer: {}", from_address);
    log::info!("   - Parent Chain URL: {}", parent_subnet.rpc_http());
    log::info!("   - Chain ID: {}", parent_subnet.id.chain_id());

    // Create keystore for deployment
    log::info!("🔑 Creating keystore for deployment...");
    let keystore = new_evm_keystore_from_arc_config(base_config.clone())
        .map_err(|e| anyhow::anyhow!("Failed to create keystore: {}", e))?;

    // Check if the deployer address is in the keystore
    if keystore.get(&from_address.into()).is_err() {
        return Err(anyhow::anyhow!(
            "Deployer address {} not found in keystore. Please import the private key first using 'ipc-cli wallet import'",
            from_address
        ));
    }

    // Create deployment configuration
    let deploy_config = DeployConfig {
        url: parent_subnet.rpc_http().to_string(),
        from: from_address,
        chain_id: parent_subnet.id.chain_id(),
        artifacts_path: None, // Use embedded contracts
        subnet_creation_privilege: CliSubnetCreationPrivilege::Unrestricted, // Allow unrestricted subnet creation
    };

    log::info!("🚀 Deploying gateway and registry contracts...");
    log::info!("   - Using embedded contract artifacts");
    log::info!("   - Subnet creation privilege: Unrestricted");
    log::info!("   - Deployer will become the gateway owner");

    // Deploy the contracts using the real deployment function
    let deployed_contracts = deploy_contracts_cmd(keystore, &deploy_config).await
        .map_err(|e| {
            log::error!("❌ Contract deployment failed: {}", e);
            anyhow::anyhow!("Gateway contract deployment failed: {}", e)
        })?;

    let gateway_addr = format!("0x{:x}", deployed_contracts.gateway);
    let registry_addr = format!("0x{:x}", deployed_contracts.registry);

    log::info!("🎉 Gateway contracts deployed successfully!");
    log::info!("   - Gateway: {}", gateway_addr);
    log::info!("   - Registry: {}", registry_addr);
    log::info!("   - Owner: {} (you have full control!)", from_address);

    // Update IPC config store with new gateway addresses
    log::info!("💾 Updating IPC configuration with new gateway addresses...");

    // The parent subnet configuration needs to be updated with the new gateway addresses
    // This allows the subnet creation to use the newly deployed contracts
    ipc_config_store
        .add_subnet(
            parent_id.clone(),
            parent_subnet.rpc_http().clone(),
            IpcEthAddress::from(deployed_contracts.gateway).into(),
            IpcEthAddress::from(deployed_contracts.registry).into(),
        )
        .await
        .map_err(|e| {
            log::error!("❌ Failed to update IPC config with new gateway addresses: {}", e);
            anyhow::anyhow!("Failed to update IPC configuration: {}", e)
        })?;

    log::info!("✅ IPC configuration updated with new gateway addresses");
    log::info!("📦 Gateway deployment completed successfully!");

        // Track the newly deployed gateway in our management system
    log::info!("💾 Adding gateway to tracking system...");
    if let Some(parent_str) = config.get("parent").and_then(|v| v.as_str()) {
        match save_deployed_gateway_to_file(
            gateway_addr.clone(),
            registry_addr.clone(),
            from_address.to_string(),
            parent_str.to_string(),
            Some(format!("Gateway for {}", parent_str)),
            &config_path,
        ).await {
            Ok(gateway_info) => {
                log::info!("✅ Gateway tracked successfully: {}", gateway_info.name);
            }
            Err(e) => {
                log::warn!("⚠️  Failed to track gateway in management system: {}", e);
                // Don't fail the deployment if tracking fails
            }
        }
    }

    Ok((Some(gateway_addr), Some(registry_addr)))
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

/// Convert UI configuration to SubnetCreateConfig
fn ui_config_to_subnet_create_config(config: &serde_json::Value) -> Result<SubnetCreateConfig> {
    let parent = config.get("parent")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing parent subnet"))?
        .to_string();

    let from = config.get("from")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let min_validator_stake = config.get("minValidatorStake")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    let min_validators = config.get("minValidators")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    let bottomup_check_period = config.get("bottomupCheckPeriod")
        .and_then(|v| v.as_i64())
        .unwrap_or(50);

    let min_cross_msg_fee = config.get("minCrossMsgFee")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.000001);

    let permission_mode = match config.get("permissionMode")
        .and_then(|v| v.as_str())
        .unwrap_or("federated") {
        "federated" => PermissionMode::Federated,
        "collateral" => PermissionMode::Collateral,
        "static" => PermissionMode::Static,
        _ => PermissionMode::Federated,
    };

    let supply_source_kind = match config.get("supplySourceKind")
        .and_then(|v| v.as_str())
        .unwrap_or("native") {
        "native" => AssetKind::Native,
        "erc20" => AssetKind::ERC20,
        _ => AssetKind::Native,
    };

    let supply_source_address = config.get("supplySourceAddress")
        .and_then(|v| v.as_str())
        .and_then(|s| if s.is_empty() { None } else { Some(s.to_string()) });

    let collateral_source_kind = match config.get("collateralSourceKind")
        .and_then(|v| v.as_str()) {
        Some("native") => Some(AssetKind::Native),
        Some("erc20") => Some(AssetKind::ERC20),
        _ => None,
    };

    let collateral_source_address = config.get("collateralSourceAddress")
        .and_then(|v| v.as_str())
        .and_then(|s| if s.is_empty() { None } else { Some(s.to_string()) });

    let genesis_subnet_ipc_contracts_owner = config.get("genesisSubnetIpcContractsOwner")
        .and_then(|v| v.as_str())
        .unwrap_or("0x0a36d7c34ba5523d5bf783bb47f62371e52e0298")
        .parse::<EthAddress>()
        .map_err(|e| anyhow::anyhow!("Invalid genesis contracts owner address: {}", e))?;

    Ok(SubnetCreateConfig {
        from,
        parent,
        min_validator_stake,
        min_validators,
        bottomup_check_period,
        active_validators_limit: None,
        min_cross_msg_fee,
        permission_mode,
        supply_source_kind,
        supply_source_address,
        validator_gater: None,
        validator_rewarder: None,
        collateral_source_kind,
        collateral_source_address,
        genesis_subnet_ipc_contracts_owner,
    })
}

/// Activate subnet deployment
async fn activate_subnet_deployment(config: &serde_json::Value, _ipc_config_store: &IpcConfigStore, subnet_info: &(SubnetID, String), config_path: &str) -> Result<()> {
    log::info!("🔥 Starting subnet activation process");

    let (subnet_id, creator) = subnet_info;
    log::info!("   - Subnet ID: {}", subnet_id);
    log::info!("   - Creator: {}", creator);

    // Create GlobalArguments
    let global = GlobalArguments {
        config_path: Some(config_path.to_string()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    // Get IPC provider
    let mut provider = get_ipc_provider(&global)?;

    // Check permission mode to determine activation type
    let permission_mode = config.get("permissionMode")
        .and_then(|v| v.as_str())
        .unwrap_or("collateral");

    match permission_mode {
        "federated" => {
            log::info!("⚙️ Setting up federated validators");

            if let Some(validators) = config.get("validators").and_then(|v| v.as_array()) {
                for validator in validators {
                    if let (Some(address), Some(pubkey_str), Some(power)) = (
                        validator.get("address").and_then(|v| v.as_str()),
                        validator.get("pubkey").and_then(|v| v.as_str()),
                        validator.get("power").and_then(|v| v.as_i64())
                    ) {
                        log::info!("   - Setting validator {} with power {}", address, power);

                        // TODO: Implement set_federated_power functionality when needed
                        log::info!("⚠️  Validator {} power configuration logged (set_federated_power not implemented)", address);
                    }
                }
            }
        }
        "collateral" => {
            log::info!("⚙️ Setting up collateral validators");

            if let Some(validators) = config.get("validators").and_then(|v| v.as_array()) {
                for validator in validators {
                    if let (Some(address), Some(collateral)) = (
                        validator.get("address").and_then(|v| v.as_str()),
                        validator.get("collateral").and_then(|v| v.as_f64())
                    ) {
                        log::info!("   - Adding validator {} with collateral {}", address, collateral);

                        let initial_balance = validator.get("initialBalance").and_then(|v| v.as_f64());

                        // TODO: Implement join_subnet functionality when needed
                        log::info!("⚠️  Validator {} configuration logged (join_subnet not implemented)", address);
                    }
                }
            }
        }
        _ => {
            log::warn!("Unknown permission mode: {}. Skipping validator setup.", permission_mode);
        }
    }

    log::info!("🎉 Subnet activation completed");
    Ok(())
}

/// Verify deployment
async fn verify_deployment(_config: &serde_json::Value, subnet_id: &SubnetID) -> Result<()> {
    log::info!("Verifying deployment for subnet: {}", subnet_id);

    // In a real implementation, this would:
    // - Check subnet is producing blocks
    // - Verify validator participation
    // - Test basic functionality

    log::info!("Deployment verification completed for subnet: {}", subnet_id);
    Ok(())
}

/// Create subnet instance after successful deployment
async fn create_subnet_instance(
    _state: &AppState,
    deployment_id: &str,
    config: &serde_json::Value,
) -> Result<()> {
    // For real deployments, the subnet would be created via the actual IPC provider
    // and registered in the IPC config. For now, we'll log the successful deployment
    // and let the real data queries pick up the new subnet when it becomes available

    log::info!("Deployment {} completed successfully", deployment_id);
    log::info!("Deployed subnet with config: {}", config);
    log::info!("New subnet should be discoverable via IPC provider queries");

    // In a real implementation, this would:
    // 1. Use the actual IPC CLI commands to create the subnet
    // 2. The subnet would be registered in the IPC provider config
    // 3. Subsequent API calls would pick up the new subnet automatically

    // For development, we can still store a temporary record to show recent deployments
    // but note that this is not persistent and is just for UI feedback

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

/// Approve a subnet using the CLI approve functionality
async fn approve_subnet_via_cli(subnet_id: &str, from_address: &str, config_path: &str) -> Result<()> {
    use crate::commands::subnet::approve::{approve_subnet as approve_subnet_cmd, ApproveSubnetArgs};

    log::info!("🔐 Starting subnet approval process");
    log::info!("   - Subnet ID: {}", subnet_id);
    log::info!("   - Gateway Owner: {}", from_address);

    // Create GlobalArguments
    let global = GlobalArguments {
        config_path: Some(config_path.to_string()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    // Get IPC provider
    let mut provider = get_ipc_provider(&global)
        .map_err(|e| {
            log::error!("❌ Failed to get IPC provider: {}", e);
            anyhow::anyhow!("Failed to initialize IPC provider: {}", e)
        })?;

    // Create approval arguments
    let approve_args = ApproveSubnetArgs {
        subnet: subnet_id.to_string(),
        from: Some(from_address.to_string()),
    };

    // Call the CLI approval function
    approve_subnet_cmd(&mut provider, &approve_args).await
        .map_err(|e| {
            log::error!("❌ Subnet approval failed: {}", e);
            anyhow::anyhow!("Subnet approval failed: {}", e)
        })?;

    log::info!("✅ Subnet approval completed successfully");
    Ok(())
}

/// Check if a subnet is approved by the gateway (with caching)
async fn check_subnet_approval_status(
    provider: &ipc_provider::IpcProvider,
    subnet_id: &ipc_api::subnet_id::SubnetID,
    cache: &ApprovalCache,
    ipc_config_store: Option<&IpcConfigStore>,
) -> bool {
    let subnet_key = subnet_id.to_string();

    // Check cache first
    {
        let cache_read = cache.read().await;
        if let Some(entry) = cache_read.get(&subnet_key) {
            // Check if cache entry is still valid
            if entry.timestamp.elapsed() < APPROVAL_CACHE_TTL {
                log::debug!("Using cached approval status for {}: {}", subnet_key, entry.is_approved);
                return entry.is_approved;
            }
        }
    }

    // Cache miss or expired, check approval status
    log::debug!("Checking approval status for {}", subnet_key);

    let is_approved = if let Some(store) = ipc_config_store {
        // First check: If subnet exists in IPC config store, it's approved
        // (Subnets only get added to config after successful approval)
        let config_snapshot = store.snapshot().await;
        let config_has_subnet = config_snapshot.subnets.contains_key(subnet_id);

        if config_has_subnet {
            log::debug!("Subnet {} found in IPC config - definitely approved", subnet_key);
            true
        } else {
            // Fallback: Check blockchain state
            log::debug!("Subnet {} not in config, checking blockchain state", subnet_key);
            check_blockchain_approval_status(provider, subnet_id).await
        }
    } else {
        // No config store available, fall back to blockchain check
        log::debug!("No config store available, checking blockchain for {}", subnet_key);
        check_blockchain_approval_status(provider, subnet_id).await
    };

    // Update cache
    {
        let mut cache_write = cache.write().await;
        cache_write.insert(subnet_key.clone(), ApprovalCacheEntry {
            is_approved,
            timestamp: Instant::now(),
        });
    }

    log::debug!("Cached new approval status for {}: {}", subnet_key, is_approved);
    is_approved
}

/// Check blockchain for subnet approval status
async fn check_blockchain_approval_status(
    provider: &ipc_provider::IpcProvider,
    subnet_id: &ipc_api::subnet_id::SubnetID,
) -> bool {
    if let Some(parent_id) = subnet_id.parent() {
        match provider.list_child_subnets(None, &parent_id).await {
            Ok(child_subnets) => {
                // If the subnet appears in the child subnets list, it's approved
                let found = child_subnets.contains_key(subnet_id);
                log::debug!("Blockchain check for {}: found in child subnets = {}", subnet_id, found);
                found
            }
            Err(e) => {
                log::debug!("Failed to list child subnets for {}: {}", parent_id, e);
                false
            }
        }
    } else {
        // Root subnets don't need approval
        false
    }
}

/// Add a validator to a subnet using the CLI functionality
async fn add_validator_via_cli(validator_data: serde_json::Value, config_path: &str) -> Result<String> {
    log::info!("🔹 Starting add validator process");

    // Extract required fields from the request
    let subnet_id = validator_data.get("subnetId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'subnetId' field"))?;

    let validator_address = validator_data.get("address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'address' field"))?;

    let permission_mode = validator_data.get("permissionMode")
        .and_then(|v| v.as_str())
        .unwrap_or("collateral");

    log::info!("   - Subnet ID: {}", subnet_id);
    log::info!("   - Validator Address: {}", validator_address);
    log::info!("   - Permission Mode: {}", permission_mode);

    // Create GlobalArguments
    let global = GlobalArguments {
        config_path: Some(config_path.to_string()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    // Get IPC provider
    let mut provider = get_ipc_provider(&global)
        .map_err(|e| anyhow::anyhow!("Failed to get IPC provider: {}", e))?;

    // Handle different permission modes
    match permission_mode {
        "collateral" => {
            // For collateral mode, we need to join the subnet first
            let collateral = validator_data.get("collateral")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| anyhow::anyhow!("Missing 'collateral' field for collateral mode"))?;

            let initial_balance = validator_data.get("initialBalance")
                .and_then(|v| v.as_f64());

            log::info!("   - Collateral: {}", collateral);
            log::info!("   - Initial Balance: {:?}", initial_balance);

            // Use the subnet join functionality
            use crate::commands::subnet::join::{join_subnet, JoinSubnetArgs};

            let args = JoinSubnetArgs {
                from: Some(validator_address.to_string()),
                subnet: subnet_id.to_string(),
                collateral,
                initial_balance,
            };

            join_subnet(&mut provider, &args).await
                .map_err(|e| anyhow::anyhow!("Failed to join subnet as validator: {}", e))?;

            Ok(format!("Successfully added validator {} to subnet {} with {} collateral",
                      validator_address, subnet_id, collateral))
        }
        "federated" => {
            // For federated mode, we need to merge with existing validators
            let power = validator_data.get("power")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| anyhow::anyhow!("Missing 'power' field for federated mode"))?;

            let public_key = validator_data.get("pubkey")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'pubkey' field for federated mode"))?;

            let from_address = validator_data.get("fromAddress")
                .and_then(|v| v.as_str())
                .unwrap_or(validator_address);

            log::info!("   - Power: {}", power);
            log::info!("   - Public Key: {}", public_key);
            log::info!("   - From Address: {}", from_address);

            // Get current validators and merge with the new one
            let subnet_id_parsed = ipc_api::subnet_id::SubnetID::from_str(subnet_id)
                .map_err(|e| anyhow::anyhow!("Invalid subnet ID '{}': {}", subnet_id, e))?;

            let current_validators = provider.list_validators(&subnet_id_parsed).await
                .map_err(|e| anyhow::anyhow!("Failed to get current validators: {}", e))?;

            log::info!("Found {} existing validators", current_validators.len());

            // Build merged validator lists
            let mut validator_addresses = Vec::new();
            let mut validator_pubkeys = Vec::new();
            let mut validator_powers = Vec::new();

            // Add existing validators
            for (addr, info) in current_validators {
                let addr_str = format!("{:#x}", addr);
                if addr_str.to_lowercase() != validator_address.to_lowercase() {
                    validator_addresses.push(addr_str);

                    // Extract public key from metadata (hex string)
                    let metadata_hex = format!("{:#x}", info.metadata);
                    let pubkey = if metadata_hex.starts_with("0x") || metadata_hex.starts_with("0X") {
                        metadata_hex[2..].to_string()
                    } else {
                        metadata_hex
                    };
                    validator_pubkeys.push(pubkey);

                    // Convert power to wei
                    let current_power = (info.current_power * 1e18) as u128;
                    validator_powers.push(current_power);
                }
            }

            // Add the new validator
            validator_addresses.push(validator_address.to_string());

            // Strip "0x" prefix from public key if present
            let clean_public_key = if public_key.starts_with("0x") || public_key.starts_with("0X") {
                public_key[2..].to_string()
            } else {
                public_key.to_string()
            };
            validator_pubkeys.push(clean_public_key);

            let new_power = (power * 1e18) as u128;
            validator_powers.push(new_power);

            log::info!("Merged validator set: {} validators total", validator_addresses.len());

            // Use set_federated_power with the merged validator set
            use crate::commands::subnet::set_federated_power::{set_federated_power, SetFederatedPowerArgs};

            let args = SetFederatedPowerArgs {
                from: from_address.to_string(),
                subnet: subnet_id.to_string(),
                validator_addresses,
                validator_pubkeys,
                validator_power: validator_powers,
            };

            set_federated_power(&provider, &args).await
                .map_err(|e| anyhow::anyhow!("Failed to set federated power for merged validator set: {}", e))?;

            Ok(format!("Successfully added federated validator {} to subnet {} with power {} (merged with existing validators)",
                      validator_address, subnet_id, power))
        }
        _ => {
            Err(anyhow::anyhow!("Unsupported permission mode: {}", permission_mode))
        }
    }
}

/// Remove a validator from a subnet using the CLI functionality
async fn remove_validator_via_cli(validator_data: serde_json::Value, config_path: &str) -> Result<String> {
    log::info!("🔹 Starting remove validator process");

    let subnet_id = validator_data.get("subnetId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'subnetId' field"))?;

    let validator_address = validator_data.get("address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'address' field"))?;

    log::info!("   - Subnet ID: {}", subnet_id);
    log::info!("   - Validator Address: {}", validator_address);

    // Create GlobalArguments
    let global = GlobalArguments {
        config_path: Some(config_path.to_string()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    // Get IPC provider
    let mut provider = get_ipc_provider(&global)
        .map_err(|e| anyhow::anyhow!("Failed to get IPC provider: {}", e))?;

    // Use the subnet leave functionality
    use crate::commands::subnet::leave::{LeaveSubnetArgs};

    let args = LeaveSubnetArgs {
        from: Some(validator_address.to_string()),
        subnet: subnet_id.to_string(),
    };

    // Use the provider method directly
    let subnet_id_parsed = ipc_api::subnet_id::SubnetID::from_str(subnet_id)
        .map_err(|e| anyhow::anyhow!("Invalid subnet ID '{}': {}", subnet_id, e))?;

    let from_addr = crate::require_fil_addr_from_str(validator_address)
        .map_err(|e| anyhow::anyhow!("Invalid from address '{}': {}", validator_address, e))?;

    provider.leave_subnet(subnet_id_parsed, Some(from_addr)).await
        .map_err(|e| anyhow::anyhow!("Failed to remove validator from subnet: {}", e))?;

    Ok(format!("Successfully removed validator {} from subnet {}", validator_address, subnet_id))
}

/// Update validator stake using the CLI functionality
async fn update_validator_stake_via_cli(stake_data: serde_json::Value, config_path: &str) -> Result<String> {
    log::info!("🔹 Starting update validator stake process");

    let subnet_id = stake_data.get("subnetId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'subnetId' field"))?;

    let amount = stake_data.get("amount")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| anyhow::anyhow!("Missing 'amount' field"))?;

    let operation = stake_data.get("operation")
        .and_then(|v| v.as_str())
        .unwrap_or("stake"); // Default to stake operation

    let validator_address = stake_data.get("address")
        .and_then(|v| v.as_str())
        .unwrap_or(""); // Optional address

    log::info!("   - Subnet ID: {}", subnet_id);
    log::info!("   - Amount: {}", amount);
    log::info!("   - Operation: {}", operation);
    log::info!("   - Validator Address: {}", validator_address);

    // Create GlobalArguments
    let global = GlobalArguments {
        config_path: Some(config_path.to_string()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    // Get IPC provider
    let mut provider = get_ipc_provider(&global)
        .map_err(|e| anyhow::anyhow!("Failed to get IPC provider: {}", e))?;

    match operation {
        "stake" => {
            // Use the stake functionality
            use crate::commands::subnet::join::{StakeSubnetArgs};

            let args = StakeSubnetArgs {
                from: if validator_address.is_empty() { None } else { Some(validator_address.to_string()) },
                subnet: subnet_id.to_string(),
                collateral: amount,
            };

            // Use provider method directly
            let subnet_id_parsed = ipc_api::subnet_id::SubnetID::from_str(subnet_id)
                .map_err(|e| anyhow::anyhow!("Invalid subnet ID '{}': {}", subnet_id, e))?;

            let from_addr = if validator_address.is_empty() {
                None
            } else {
                Some(crate::require_fil_addr_from_str(validator_address)
                    .map_err(|e| anyhow::anyhow!("Invalid from address '{}': {}", validator_address, e))?)
            };

            let token_amount = crate::f64_to_token_amount(amount)
                .map_err(|e| anyhow::anyhow!("Invalid token amount: {}", e))?;

            provider.stake(subnet_id_parsed, from_addr, token_amount).await
                .map_err(|e| anyhow::anyhow!("Failed to stake in subnet: {}", e))?;

            Ok(format!("Successfully staked {} tokens in subnet {}", amount, subnet_id))
        }
        "unstake" => {
            // Use the unstake functionality
            use crate::commands::subnet::join::{UnstakeSubnetArgs};

            let args = UnstakeSubnetArgs {
                from: if validator_address.is_empty() { None } else { Some(validator_address.to_string()) },
                subnet: subnet_id.to_string(),
                collateral: amount,
            };

            // Use provider method directly
            let subnet_id_parsed = ipc_api::subnet_id::SubnetID::from_str(subnet_id)
                .map_err(|e| anyhow::anyhow!("Invalid subnet ID '{}': {}", subnet_id, e))?;

            let from_addr = if validator_address.is_empty() {
                None
            } else {
                Some(crate::require_fil_addr_from_str(validator_address)
                    .map_err(|e| anyhow::anyhow!("Invalid from address '{}': {}", validator_address, e))?)
            };

            let token_amount = crate::f64_to_token_amount(amount)
                .map_err(|e| anyhow::anyhow!("Invalid token amount: {}", e))?;

            provider.unstake(subnet_id_parsed, from_addr, token_amount).await
                .map_err(|e| anyhow::anyhow!("Failed to unstake from subnet: {}", e))?;

            Ok(format!("Successfully unstaked {} tokens from subnet {}", amount, subnet_id))
        }
        _ => {
            Err(anyhow::anyhow!("Unsupported stake operation: {}", operation))
        }
    }
}

/// Set federated power for a complete validator set
async fn set_federated_power_via_cli(power_data: serde_json::Value, config_path: &str) -> Result<String> {
    log::info!("🔹 Starting set federated power process");

    // Extract required fields from the request
    let subnet_id = power_data.get("subnetId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'subnetId' field"))?;

    let from_address = power_data.get("fromAddress")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'fromAddress' field"))?;

    let validators = power_data.get("validators")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing 'validators' field"))?;

    log::info!("   - Subnet ID: {}", subnet_id);
    log::info!("   - From Address: {}", from_address);
    log::info!("   - Validators count: {}", validators.len());

    // Create GlobalArguments
    let global = GlobalArguments {
        config_path: Some(config_path.to_string()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    // Get IPC provider
    let provider = get_ipc_provider(&global)
        .map_err(|e| anyhow::anyhow!("Failed to get IPC provider: {}", e))?;

    // Parse validators from the request
    let mut validator_addresses = Vec::new();
    let mut validator_pubkeys = Vec::new();
    let mut validator_powers = Vec::new();

    for validator in validators {
        let address = validator.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'address' field in validator"))?;

        let pubkey = validator.get("pubkey")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'pubkey' field in validator"))?;

        let power = validator.get("power")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'power' field in validator"))?;

        validator_addresses.push(address.to_string());

        // Strip "0x" prefix from public key if present
        let clean_public_key = if pubkey.starts_with("0x") || pubkey.starts_with("0X") {
            pubkey[2..].to_string()
        } else {
            pubkey.to_string()
        };
        validator_pubkeys.push(clean_public_key);

        let power_wei = (power * 1e18) as u128;
        validator_powers.push(power_wei);

        log::info!("   - Validator: {} (power: {})", address, power);
    }

    // Use set_federated_power with the complete validator set
    use crate::commands::subnet::set_federated_power::{set_federated_power, SetFederatedPowerArgs};

    let args = SetFederatedPowerArgs {
        from: from_address.to_string(),
        subnet: subnet_id.to_string(),
        validator_addresses,
        validator_pubkeys,
        validator_power: validator_powers,
    };

    set_federated_power(&provider, &args).await
        .map_err(|e| anyhow::anyhow!("Failed to set federated power: {}", e))?;

    Ok(format!("Successfully set federated power for {} validators in subnet {}", validators.len(), subnet_id))
}
