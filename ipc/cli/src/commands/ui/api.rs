// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! API endpoints for the UI service
//!
//! This module will contain specific API endpoint implementations
//! that can be shared between different server implementations.

use super::{AppState, SubnetInstance};
use anyhow::Result;
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

// Future API implementations will go here