// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::services::Service;
use anyhow::Result;
use async_trait::async_trait;
use fendermint_app::service::node::run as run_fendermint;
use fendermint_app_settings::Settings;
use ipc_observability::traces::create_temporary_subscriber;
use tokio_util::sync::CancellationToken;
use tracing::{info, subscriber};

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

    async fn run(&self, shutdown: CancellationToken) -> Result<()> {
        info!(target: "service.fendermint", "Starting Fendermint ABCI application");

        // Use isolated tracing context like in single-binary-runner
        subscriber::with_default(create_temporary_subscriber(), || {
            tracing::info!(
                "Fendermint service starting with home: {:?}",
                self.settings.home_dir()
            );
        });

        run_fendermint(self.settings.clone(), Some(shutdown)).await
    }
}
