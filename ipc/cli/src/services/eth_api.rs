// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::services::Service;
use anyhow::Result;
use async_trait::async_trait;
use fendermint_app_settings::Settings;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::info;

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

    async fn run(&self, _shutdown: CancellationToken) -> Result<()> {
        // TODO: Implement proper ETH API service startup
        // For now, just log that we would start ETH API
        info!(target: "eth_api", "ETH API would start here");
        Ok(())
    }
}
