// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! UI module for the IPC CLI
//!
//! This module provides a web-based user interface for managing IPC operations.

use anyhow::Result;
use clap::{Args, Subcommand};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub mod api;
pub mod server;
pub mod services;
pub mod websocket;

/// Import the simplified server function
pub use server::start_ui_server;

/// WebSocket client handle
pub type WebSocketClient = std::sync::Arc<tokio::sync::Mutex<futures_util::stream::SplitSink<warp::ws::WebSocket, warp::ws::Message>>>;

/// Deployment state tracking
#[derive(Debug, Clone)]
pub struct DeploymentState {
    pub id: String,
    pub template: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub config: serde_json::Value,
    pub progress: u8,
    pub step: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Shared application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub config_path: String,
    pub mode: DeploymentMode,
    pub deployments: Arc<Mutex<HashMap<String, DeploymentState>>>,
    pub instances: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    pub websocket_clients: Arc<Mutex<Vec<WebSocketClient>>>,
    pub deployed_gateways: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    pub subnet_metadata: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

/// Deployment mode for the UI
#[derive(Debug, Clone)]
pub enum DeploymentMode {
    Development,
    Production,
}

/// UI command arguments
#[derive(Debug, Args)]
pub struct UICommandArgs {
    /// Address to bind the UI server to (default: 127.0.0.1)
    #[clap(long, default_value = "127.0.0.1")]
    pub address: String,

    /// Port to bind the UI server to (default: 3000)
    #[clap(long, default_value = "3000")]
    pub port: u16,

    /// Configuration file path
    #[clap(long)]
    pub config_path: Option<String>,
}

/// UI subcommand
#[derive(Debug, Subcommand)]
pub enum UICommand {
    /// Start the UI server
    Start,
}

/// Run the UI command
pub async fn run_ui_command(
    global: crate::GlobalArguments,
    command: UICommand,
    args: UICommandArgs,
) -> Result<()> {
    match command {
        UICommand::Start => {
            let ip: IpAddr = args.address.parse()
                .unwrap_or_else(|_| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
            let addr = SocketAddr::new(ip, args.port);

            // Fix config path resolution - use global.config_path() which handles defaults properly
            let config_path = args.config_path
                .unwrap_or_else(|| global.config_path());

            start_ui_server(config_path, addr).await
        }
    }
}