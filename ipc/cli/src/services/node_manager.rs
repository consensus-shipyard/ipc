// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::services::comet_bft::CometBftService;
use crate::services::eth_api::EthApiService;
use crate::services::fendermint::FendermintService;
use crate::services::{run_services, Service};
use anyhow::Result;
use fendermint_app_settings::Settings;
use ipc_observability::traces::create_temporary_subscriber;
use std::path::PathBuf;
use std::time::Duration;
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

        // Use isolated tracing context like in single-binary-runner
        subscriber::with_default(create_temporary_subscriber(), || {
            tracing::info!("Node manager starting with home: {:?}", self.home);
        });

        // Create all services
        let services: Vec<Box<dyn Service>> = vec![
            // Fendermint ABCI application (in-process)
            Box::new(FendermintService::new(self.settings.clone())),
            // CometBFT consensus engine (external process)
            Box::new(CometBftService::new(self.home.join("cometbft"))),
            // ETH API facade (in-process)
            Box::new(EthApiService::new(
                self.settings.clone(),
                Duration::from_secs(5), // connect retry delay
            )),
        ];

        info!("All services created, starting orchestration");

        // Run all services with graceful shutdown
        run_services(services).await?;

        info!("All node services stopped");
        Ok(())
    }
}
