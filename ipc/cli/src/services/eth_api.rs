// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::services::Service;
use anyhow::{Context, Result};
use async_trait::async_trait;
use fendermint_app::service::eth_api::run as run_eth_api;
use fendermint_app_settings::Settings;
use fendermint_eth_api::HybridClient;
use ipc_observability::traces::create_temporary_subscriber;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{info, subscriber};

pub struct EthApiService {
    settings: Settings,
    connect_retry_delay: Duration,
}

impl EthApiService {
    pub fn new(settings: Settings, connect_retry_delay: Duration) -> Self {
        Self {
            settings,
            connect_retry_delay,
        }
    }
}

#[async_trait]
impl Service for EthApiService {
    fn name(&self) -> &'static str {
        "ETH API Service"
    }

    async fn run(&self, shutdown: CancellationToken) -> Result<()> {
        info!(target: "service.eth_api", "Starting ETH API service");

        // Use isolated tracing context like in single-binary-runner
        subscriber::with_default(create_temporary_subscriber(), || {
            tracing::info!(
                "ETH API service starting on: {:?}",
                self.settings.eth.listen
            );
        });

        // Create HybridClient for Tendermint connection
        let http_url = self.settings.tendermint_rpc_url()?;
        let ws_url = self.settings.tendermint_websocket_url()?;

        let (client, driver) = HybridClient::new(http_url, ws_url, self.connect_retry_delay)
            .context("failed to create HybridClient")?;

        let driver_handle = tokio::spawn(async move { driver.run().await });

        let result = run_eth_api(self.settings.eth.clone(), client, Some(shutdown)).await;

        // Await the driver's termination to ensure proper connection closure.
        let _ = driver_handle.await;
        result
    }
}
