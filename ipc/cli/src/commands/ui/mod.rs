// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! UI service command for starting the web interface backend

use crate::{CommandLineHandler, GlobalArguments};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod server;
mod api;
mod websocket;

use server::UIServer;

/// The UI service command
pub struct UICommand;

#[async_trait]
impl CommandLineHandler for UICommand {
    type Arguments = UICommandArgs;

    async fn handle(global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        log::info!("Starting IPC UI service...");
        log::info!("Frontend: http://{}:{}", args.host, args.port);
        log::info!("Backend API: http://{}:{}", args.host, args.backend_port);
        log::info!("WebSocket: ws://{}:{}/ws", args.host, args.backend_port);

        // Create the UI server
        let mut server = UIServer::new(
            args.host.clone(),
            args.port,
            args.backend_port,
            args.mode.clone(),
            global.config_path(),
        )?;

        // Start the server
        server.start().await?;

        // Open browser if requested
        if !args.no_browser {
            let url = format!("http://{}:{}", args.host, args.port);
            if let Err(e) = open_browser(&url) {
                log::warn!("Failed to open browser: {}", e);
                log::info!("Please open {} in your browser", url);
            }
        }

        log::info!("UI service started successfully");
        log::info!("Press Ctrl+C to stop");

        // Keep the service running
        tokio::signal::ctrl_c().await?;
        log::info!("Shutting down UI service...");

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(name = "ui", about = "Start the IPC web UI service")]
pub struct UICommandArgs {
    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1", help = "Host address to bind to")]
    pub host: String,

    /// Port for the frontend server
    #[arg(long, default_value = "3000", help = "Port for the frontend server")]
    pub port: u16,

    /// Port for the backend API server
    #[arg(long, default_value = "3001", help = "Port for the backend API server")]
    pub backend_port: u16,

    /// Deployment mode
    #[arg(long, default_value = "development", help = "Deployment mode (development, testnet, mainnet)")]
    pub mode: String,

    /// Don't automatically open browser
    #[arg(long, help = "Don't automatically open browser")]
    pub no_browser: bool,
}

/// Deployment modes for different network configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentMode {
    Development,
    Testnet,
    Mainnet,
}

impl std::str::FromStr for DeploymentMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(DeploymentMode::Development),
            "testnet" | "test" => Ok(DeploymentMode::Testnet),
            "mainnet" | "main" => Ok(DeploymentMode::Mainnet),
            _ => Err(anyhow!("Invalid deployment mode: {}", s)),
        }
    }
}

/// Shared application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub config_path: String,
    pub mode: DeploymentMode,
    pub instances: Arc<Mutex<HashMap<String, SubnetInstance>>>,
    pub websocket_clients: Arc<Mutex<Vec<WebSocketClient>>>,
}

/// Subnet instance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetInstance {
    pub id: String,
    pub name: String,
    pub status: String,
    pub template: String,
    pub parent: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub validators: Vec<ValidatorInfo>,
    pub config: serde_json::Value,
}

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: String,
    pub stake: String,
    pub power: u64,
    pub status: String,
}

/// WebSocket client connection
#[derive(Debug)]
pub struct WebSocketClient {
    pub id: String,
    pub sender: tokio::sync::mpsc::UnboundedSender<warp::ws::Message>,
}

/// Open browser utility function
fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/c", "start", url])
            .spawn()?;
    }
    Ok(())
}