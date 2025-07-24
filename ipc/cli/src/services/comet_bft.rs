// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::comet_runner::CometDaemon;
use crate::services::Service;
use anyhow::Result;
use async_trait::async_trait;
use ipc_observability::traces::create_temporary_subscriber;
use std::path::PathBuf;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{info, subscriber};

pub struct CometBftService {
    home: PathBuf,
}

impl CometBftService {
    pub fn new(home: PathBuf) -> Self {
        Self { home }
    }
}

#[async_trait]
impl Service for CometBftService {
    fn name(&self) -> &'static str {
        "CometBFT Service"
    }

    async fn run(&self, shutdown: CancellationToken) -> Result<()> {
        info!(target: "service.cometbft", "Starting CometBFT consensus service");

        // Use isolated tracing context like in single-binary-runner
        subscriber::with_default(create_temporary_subscriber(), || {
            tracing::info!("CometBFT service starting with home: {:?}", self.home);
        });

        let daemon = CometDaemon::new(vec![
            "start".to_string(),
            "--home".to_string(),
            self.home.to_string_lossy().to_string(),
        ])
        .with_max_restarts(5)
        .with_restart_delay(Duration::from_secs(3));

        daemon.run_with_restart(shutdown).await
    }
}
