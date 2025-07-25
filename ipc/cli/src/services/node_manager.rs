// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::services::comet_bft::CometBftService;
use crate::services::eth_api::EthApiService;
use crate::services::fendermint::FendermintService;
use crate::services::Service;
use anyhow::{Context, Result};
use fendermint_app_settings::Settings;
use ipc_observability::traces::create_temporary_subscriber;
use std::path::PathBuf;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{info, subscriber};

pub struct NodeManager {
    home: PathBuf,
    settings: Settings,
}

impl NodeManager {
    pub fn new(home: PathBuf, settings: Settings) -> Self {
        Self { home, settings }
    }

    /// Start all node services and run them until shutdown
    pub async fn start_all_services(&self) -> Result<()> {
        info!(
            target: "service.node_manager",
            "Starting IPC node services from home directory: {}",
            self.home.display()
        );

        subscriber::with_default(create_temporary_subscriber(), || {
            tracing::info!("Node manager starting with home: {:?}", self.home);
        });

        info!("Starting services in order: Fendermint -> CometBFT -> ETH API");

        // Create a shared cancellation token for all services
        let cancel_token = CancellationToken::new();

        // Start Fendermint first (it provides the ABCI server that CometBFT needs)
        let fendermint_service = FendermintService::new(self.settings.clone());
        let fendermint_handle = {
            let token = cancel_token.clone();
            tokio::spawn(async move { fendermint_service.run(token).await })
        };

        // Give Fendermint a moment to start the ABCI server
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Start CometBFT (it will connect to Fendermint's ABCI server)
        let cometbft_service = CometBftService::new(self.home.join("cometbft"));
        let cometbft_handle = {
            let token = cancel_token.clone();
            tokio::spawn(async move { cometbft_service.run(token).await })
        };

        // Wait for CometBFT RPC to be ready
        self.wait_for_comet_bft_ready().await?;
        info!(target: "service.node_manager", "CometBFT RPC is ready");

        // Start ETH API service
        let eth_api_service = EthApiService::new(
            self.settings.clone(),
            Duration::from_secs(5), // connect retry delay
        );
        let eth_api_handle = {
            let token = cancel_token.clone();
            tokio::spawn(async move { eth_api_service.run(token).await })
        };

        // Wait for all services
        let (fendermint_result, cometbft_result, eth_api_result) =
            tokio::join!(fendermint_handle, cometbft_handle, eth_api_handle);

        // Check results
        if let Err(e) = fendermint_result {
            anyhow::bail!("Fendermint service failed: {:?}", e);
        }
        if let Err(e) = cometbft_result {
            anyhow::bail!("CometBFT service failed: {:?}", e);
        }
        if let Err(e) = eth_api_result {
            anyhow::bail!("ETH API service failed: {:?}", e);
        }

        info!("All node services stopped");
        Ok(())
    }

    /// Wait for CometBFT RPC to be ready by polling the status endpoint
    async fn wait_for_comet_bft_ready(&self) -> Result<()> {
        use std::time::Duration;
        use tokio::time::{sleep, timeout};

        let max_wait = Duration::from_secs(30);
        let poll_interval = Duration::from_millis(500);

        info!(target: "service.node_manager", "Waiting for CometBFT RPC to be ready...");

        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > max_wait {
                anyhow::bail!(
                    "CometBFT RPC did not become ready within {} seconds",
                    max_wait.as_secs()
                );
            }

            // Try to connect to CometBFT RPC
            match timeout(Duration::from_secs(1), self.check_comet_bft_status()).await {
                Ok(Ok(_)) => {
                    info!(target: "service.node_manager", "CometBFT RPC is ready");
                    return Ok(());
                }
                Ok(Err(e)) => {
                    info!(target: "service.node_manager", "CometBFT RPC not ready yet: {}", e);
                }
                Err(_) => {
                    info!(target: "service.node_manager", "CometBFT RPC connection timeout");
                }
            }

            sleep(poll_interval).await;
        }
    }

    /// Check if CometBFT RPC is responding
    async fn check_comet_bft_status(&self) -> Result<()> {
        use reqwest::Client;

        let client = Client::new();
        let response = client
            .get("http://127.0.0.1:26657/status")
            .timeout(Duration::from_secs(1))
            .send()
            .await
            .context("Failed to connect to CometBFT RPC")?;

        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("CometBFT RPC returned status: {}", response.status())
        }
    }
}
