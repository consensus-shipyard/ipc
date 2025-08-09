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

/// Subnet status response
#[derive(Debug, Serialize, Deserialize)]
pub struct SubnetStatus {
    pub is_active: bool,
    pub last_block_time: String,
    pub block_height: u64,
    pub validators_online: u32,
    pub consensus_status: String, // "healthy", "degraded", "offline"
    pub sync_status: String, // "synced", "syncing", "behind"
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