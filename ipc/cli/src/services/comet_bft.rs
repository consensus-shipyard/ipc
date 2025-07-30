// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::comet_runner::CometDaemon;
use crate::services::Service;
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

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
        "CometBFT"
    }

    async fn run(&self, shutdown: CancellationToken) -> Result<()> {
        let daemon = CometDaemon::new(vec![
            "start".to_string(),
            "--home".to_string(),
            self.home.to_string_lossy().to_string(),
        ])
        .with_max_restarts(5);

        daemon.run_with_restart(shutdown).await
    }
}
