// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! API type definitions for the UI service

use serde::{Deserialize, Serialize};

/// API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Deployment request payload
#[derive(Debug, Deserialize)]
pub struct DeploymentRequest {
    pub template: String,
    pub config: serde_json::Value,
}

/// Deployment response
#[derive(Debug, Serialize)]
pub struct DeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub message: String,
}

/// Chain statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct ChainStats {
    pub block_height: u64,
    pub latest_block_time: String,
    pub transaction_count: u64,
    pub validator_count: u32,
    pub tps: f64,
    pub avg_block_time: f64,
    pub last_checkpoint: String,
    pub total_supply: String,
    pub circulating_supply: String,
    pub fees_collected: String,
    pub pending_transactions: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubnetLifecycleState {
    // Deployment states
    #[serde(rename = "deploying")]
    Deploying,
    #[serde(rename = "deployed")]
    Deployed,

    // Initialization states
    #[serde(rename = "initializing")]
    Initializing,
    #[serde(rename = "waiting_for_validators")]
    WaitingForValidators,

    // Active states
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "syncing")]
    Syncing,
    #[serde(rename = "healthy")]
    Healthy,

    // Problem states
    #[serde(rename = "degraded")]
    Degraded,
    #[serde(rename = "offline")]
    Offline,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "unknown")]
    Unknown,
}

impl Default for SubnetLifecycleState {
    fn default() -> Self {
        SubnetLifecycleState::Unknown
    }
}

impl std::fmt::Display for SubnetLifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubnetLifecycleState::Deploying => write!(f, "deploying"),
            SubnetLifecycleState::Deployed => write!(f, "deployed"),
            SubnetLifecycleState::Initializing => write!(f, "initializing"),
            SubnetLifecycleState::WaitingForValidators => write!(f, "waiting_for_validators"),
            SubnetLifecycleState::Active => write!(f, "active"),
            SubnetLifecycleState::Syncing => write!(f, "syncing"),
            SubnetLifecycleState::Healthy => write!(f, "healthy"),
            SubnetLifecycleState::Degraded => write!(f, "degraded"),
            SubnetLifecycleState::Offline => write!(f, "offline"),
            SubnetLifecycleState::Failed => write!(f, "failed"),
            SubnetLifecycleState::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetStatusInfo {
    pub lifecycle_state: SubnetLifecycleState,
    pub genesis_available: bool,
    pub validator_count: usize,
    pub active_validators: usize,
    pub permission_mode: Option<String>,
    pub deployment_time: Option<String>,
    pub last_block_time: Option<String>,
    pub error_message: Option<String>,
    pub next_action_required: Option<String>,
    // Additional fields for frontend compatibility
    pub is_active: bool,
    pub block_height: u64,
    pub validators_online: usize,
    pub consensus_status: String,
    pub sync_status: Option<String>,
    pub status: String,
    pub message: String,
    // New fields for detailed subnet setup status
    pub setup_checklist: SubnetSetupChecklist,
}

/// Detailed checklist of subnet setup steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetSetupChecklist {
    pub permission_mode: String, // "federated", "collateral", "static"
    pub steps: Vec<SetupStep>,
    pub next_required_action: Option<String>,
    pub all_complete: bool,
}

/// Individual setup step with status and action info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: StepStatus,
    pub required: bool,
    pub action_available: bool,
    pub action_button_text: Option<String>,
    pub action_type: Option<String>, // "set_federated_power", "approve_subnet", etc.
    pub details: Option<serde_json::Value>, // Additional data for the action
}

/// Status of an individual setup step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "not_applicable")]
    NotApplicable,
}

impl Default for SubnetStatusInfo {
    fn default() -> Self {
        SubnetStatusInfo {
            lifecycle_state: SubnetLifecycleState::Unknown,
            genesis_available: false,
            validator_count: 0,
            active_validators: 0,
            permission_mode: None,
            deployment_time: None,
            last_block_time: None,
            error_message: None,
            next_action_required: None,
            is_active: false,
            block_height: 0,
            validators_online: 0,
            consensus_status: String::new(),
            sync_status: None,
            status: String::new(),
            message: String::new(),
            setup_checklist: SubnetSetupChecklist::default(),
        }
    }
}

impl Default for SubnetSetupChecklist {
    fn default() -> Self {
        SubnetSetupChecklist {
            permission_mode: "unknown".to_string(),
            steps: Vec::new(),
            next_required_action: None,
            all_complete: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetInstanceResponse {
    pub id: String,
    pub name: String,
    pub parent: String,
    pub status: SubnetLifecycleState,
    pub status_info: SubnetStatusInfo,
    pub validators: Vec<serde_json::Value>,
    pub created_at: Option<String>,
    pub config: Option<serde_json::Value>,
}

/// Test transaction request
#[derive(Debug, Deserialize)]
pub struct TestTransactionRequest {
    pub tx_type: String, // "simple", "transfer", "contract_call"
    pub network: String, // "subnet", "l1"
    pub from: Option<String>,
    pub to: Option<String>,
    pub amount: Option<String>,
    pub data: Option<String>,
    pub gas_limit: Option<u64>,
}

/// Test transaction response
#[derive(Debug, Serialize)]
pub struct TestTransactionResponse {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub block_number: Option<u64>,
    pub gas_used: Option<u64>,
    pub error: Option<String>,
    pub network: String, // Which network the transaction was sent to
}

/// Wallet address information for UI selection
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalletAddress {
    pub address: String,
    pub wallet_type: String, // "evm" or "fvm"
    pub pubkey: Option<String>, // For EVM addresses, used in validator selection
    pub balance: Option<String>, // Balance in the current subnet context
    pub custom_label: Option<String>, // User-defined name for the address
    pub is_default: bool, // Whether this is the default address for this wallet type
}

/// Gateway information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GatewayInfo {
    pub id: String,
    pub address: String,
    pub registry_address: String,
    pub deployer_address: String,
    pub parent_network: String,
    pub name: Option<String>,
    pub subnet_count: u64,
    pub is_active: bool,
    pub deployed_at: chrono::DateTime<chrono::Utc>,
}

/// Subnet metadata for tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubnetMetadata {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub name: Option<String>,
    pub template: Option<String>,
}

/// Error types for better error handling
#[derive(Debug)]
pub struct InvalidRequest(pub String);

impl warp::reject::Reject for InvalidRequest {}

#[derive(Debug)]
pub struct ServerError(pub String);

impl warp::reject::Reject for ServerError {}