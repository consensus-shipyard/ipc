//! Deployment service for subnet creation through the UI
//!
//! This service wraps existing CLI handlers for deployment operations.

use crate::commands::deploy::DeployConfig;
use crate::commands::subnet::create::SubnetCreateConfig;
use crate::{GlobalArguments};
use anyhow::Result;
use fvm_shared::address::Address;
use ipc_api::subnet_id::SubnetID;
use ipc_api::subnet::{PermissionMode, AssetKind};
use std::str::FromStr;

/// Result of a subnet deployment operation
#[derive(Debug, Clone)]
pub struct SubnetDeploymentResult {
    pub subnet_id: String,
    pub parent_id: String,
    pub gateway_address: Option<String>,
    pub registry_address: Option<String>,
}

/// Progress callback type for deployment operations
pub type ProgressCallback = Box<dyn Fn(&str, u8, &str, Option<String>) + Send + Sync>;

/// Service for handling subnet deployments via the UI
pub struct DeploymentService {
    global: GlobalArguments,
    progress_callback: Option<ProgressCallback>,
}

impl DeploymentService {
    /// Create a new deployment service
    pub fn new(global: GlobalArguments) -> Self {
        Self {
            global,
            progress_callback: None,
        }
    }

    /// Set a progress callback for deployment operations
    pub fn with_progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    /// Report progress to the callback if set
    fn report_progress(&self, step: &str, progress: u8, status: &str, message: Option<String>) {
        if let Some(callback) = &self.progress_callback {
            callback(step, progress, status, message);
        }
    }

    /// Deploy a subnet with the provided configuration
    pub async fn deploy_subnet(&self, ui_config: serde_json::Value) -> Result<SubnetDeploymentResult> {
        // Extract parent subnet ID
        let parent_str = ui_config.get("parent")
            .and_then(|v| v.as_str())
            .unwrap_or("/r314159");
        let parent = SubnetID::from_str(parent_str)?;

        // Create a default address for missing fields
        let default_address = Address::new_id(0);

        // Convert UI config to SubnetCreateConfig - using only the fields that actually exist
        let subnet_config = SubnetCreateConfig {
            from: ui_config.get("from").and_then(|v| v.as_str()).map(|s| s.to_string()),
            parent: parent.to_string(),
            min_validator_stake: ui_config.get("min_validator_stake")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(1.0),
            min_validators: ui_config.get("min_validators")
                .and_then(|v| v.as_u64())
                .unwrap_or(1),
            bottomup_check_period: ui_config.get("bottomup_check_period")
                .and_then(|v| v.as_i64())
                .unwrap_or(100),
            active_validators_limit: ui_config.get("active_validators_limit")
                .and_then(|v| v.as_u64())
                .map(|v| v as u16),
            min_cross_msg_fee: ui_config.get("min_cross_msg_fee")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.000001),
            permission_mode: PermissionMode::Collateral,
            supply_source_kind: AssetKind::Native,
            supply_source_address: ui_config.get("supply_source_address")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            validator_gater: ui_config.get("validator_gater")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            // Add missing required fields with defaults
            collateral_source_kind: Some(AssetKind::Native),
            collateral_source_address: None,
            genesis_subnet_ipc_contracts_owner: ethers::types::Address::zero(),
            validator_rewarder: None,
        };

        // Create deployment configuration if needed
        let deploy_config = if ui_config.get("deploy_gateway").and_then(|v| v.as_bool()).unwrap_or(false) {
            let from_str = ui_config.get("from")
                .and_then(|v| v.as_str())
                .unwrap_or("0x0000000000000000000000000000000000000000");
            let from_address = ethers::types::Address::from_str(from_str)
                .unwrap_or(ethers::types::Address::zero());

            Some(DeployConfig {
                url: ui_config.get("rpc_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("http://localhost:8545")
                    .to_string(),
                from: from_address,
                chain_id: ui_config.get("chain_id")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(31415926),
                artifacts_path: None, // Use default builtin contracts
                subnet_creation_privilege: crate::commands::deploy::CliSubnetCreationPrivilege::Unrestricted,
            })
        } else {
            None
        };

        // For now, return a placeholder result
        // TODO: Implement actual subnet creation using the CLI handlers
        Ok(SubnetDeploymentResult {
            subnet_id: format!("{}/r{}", parent, chrono::Utc::now().timestamp()),
            parent_id: parent.to_string(),
            gateway_address: deploy_config.as_ref().map(|_| "0x1234567890123456789012345678901234567890".to_string()),
            registry_address: deploy_config.as_ref().map(|_| "0x0987654321098765432109876543210987654321".to_string()),
        })
    }
}