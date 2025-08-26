// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! WebSocket message types and connection state

use serde::{Deserialize, Serialize};

/// WebSocket message types from frontend to backend
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum IncomingMessage {
    /// Subscribe to deployment progress updates
    SubscribeDeployment { deployment_id: String },
    /// Subscribe to instance status updates
    SubscribeInstance { instance_id: String },
    /// Ping message for connection health check
    Ping,
}

/// WebSocket message types from backend to frontend
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum OutgoingMessage {
    /// Deployment progress update
    DeploymentProgress {
        deployment_id: String,
        step: String,
        status: String,
        progress: u8,
        message: Option<String>,
    },
    /// Instance status update
    InstanceStatus {
        instance_id: String,
        status: String,
        message: Option<String>,
    },
    /// Pong response to ping
    Pong,
    /// Error message
    Error { message: String },
}

/// WebSocket connection state
#[derive(Debug)]
#[allow(dead_code)]
pub struct ConnectionState {
    pub client_id: String,
    pub subscriptions: Vec<String>,
}

impl ConnectionState {
    #[allow(dead_code)]
    pub fn new(client_id: String) -> Self {
        Self {
            client_id,
            subscriptions: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn subscribe(&mut self, subscription: String) {
        if !self.subscriptions.contains(&subscription) {
            self.subscriptions.push(subscription);
        }
    }

    #[allow(dead_code)]
    pub fn unsubscribe(&mut self, subscription: &str) {
        self.subscriptions.retain(|s| s != subscription);
    }
}
