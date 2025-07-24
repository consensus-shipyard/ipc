// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::services::Service;
use anyhow::Result;
use async_trait::async_trait;
use fendermint_app_settings::Settings;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct FendermintService {
    settings: Settings,
}

impl FendermintService {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }
}

#[async_trait]
impl Service for FendermintService {
    fn name(&self) -> &'static str {
        "Fendermint Service"
    }

    async fn run(&self, _shutdown: CancellationToken) -> Result<()> {
        // TODO: Implement proper Fendermint service startup
        // For now, just log that we would start Fendermint
        info!(target: "fendermint", "Fendermint ABCI application would start here");
        Ok(())
    }
}
