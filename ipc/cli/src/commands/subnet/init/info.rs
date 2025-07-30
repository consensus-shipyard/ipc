// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Subnet information that gets serialized to subnet-{SUBNET_ID}.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubnetInfo {
    /// General subnet information
    pub subnet_info: SubnetGeneralInfo,
    /// Contract addresses and deployment info
    pub contracts: SubnetContractInfo,
    /// Genesis information
    pub genesis: SubnetGenesisInfo,
    /// Activation information
    pub activation: Option<SubnetActivationInfo>,
}

/// General subnet information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubnetGeneralInfo {
    /// Subnet ID
    pub subnet_id: String,
    /// Parent subnet ID
    pub parent_id: String,
    /// Subnet name (if provided)
    pub name: Option<String>,
    /// Creation timestamp
    pub created_at: String,
    /// Network (mainnet/testnet)
    pub network: String,
}

/// Contract deployment information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubnetContractInfo {
    /// Gateway contract address
    pub gateway_address: String,
    /// Registry contract address
    pub registry_address: String,
    /// Parent gateway address
    pub parent_gateway: String,
    /// Parent registry address
    pub parent_registry: String,
}

/// Genesis file information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubnetGenesisInfo {
    /// Path to genesis file
    pub genesis_path: String,
    /// Path to sealed genesis file
    pub sealed_genesis_path: String,
    /// Network version
    pub network_version: u32,
    /// Base fee
    pub base_fee: String,
    /// Power scale
    pub power_scale: i8,
}

/// Subnet activation information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubnetActivationInfo {
    /// Activation mode (federated/static/collateral)
    pub mode: String,
    /// Validator addresses (if any)
    pub validators: Vec<String>,
    /// Stake amounts (if collateral mode)
    pub stakes: Option<Vec<String>>,
}
