// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::{CommandLineHandler, GlobalArguments};
use async_trait::async_trait;
use std::{env, path::PathBuf};

use fendermint_app_settings::Settings;
use ipc_observability::config::{
    default_log_level, FileLayerSettings, RotationKind, TracingSettings,
};
use ipc_observability::traces::set_global_tracing_subscriber;
use ipc_runner::services::comet_bft::CometBftService;
use ipc_runner::services::eth_api::EthApiService;
use ipc_runner::services::node::NodeService;
use ipc_runner::services::{run as run_services, Service};

use std::time::Duration;

use clap::Args;

pub(crate) struct StartNode;

#[async_trait]
impl CommandLineHandler for StartNode {
    type Arguments = StartNodeArgs;

    async fn handle(_global: &GlobalArguments, _arguments: &Self::Arguments) -> anyhow::Result<()> {
        // TODO Karel - generated the config - use default + override from the runner config
        let home_dir = expand_tilde("~/.fendermint");
        let config_dir = home_dir.join("config");
        let run_mode = "runner";
        let mut settings = Settings::new(&config_dir, &home_dir, run_mode).unwrap();

        settings.tracing = TracingSettings::default();
        settings.tracing.file = Some(FileLayerSettings {
            enabled: true,
            directory: Some(home_dir.join("logs")),
            level: default_log_level(),
            max_log_files: Some(5),
            rotation: Some(RotationKind::Daily),
            domain_filter: None,
            events_filter: None,
        });

        // TODO Karel - have a separate file for ETH API
        let _trace_file_guard = set_global_tracing_subscriber(&settings.tracing);

        let node_service = NodeService::new(settings.clone());
        let comet_bft_service = CometBftService::new(settings.abci.listen.clone(), 3);
        let eth_api_service = EthApiService::new(settings.clone(), Duration::from_secs(2));

        let services: Vec<Box<dyn Service>> = vec![
            Box::new(node_service),
            Box::new(comet_bft_service),
            Box::new(eth_api_service),
        ];

        run_services(services).await.unwrap();

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Arguments to start node")]
pub(crate) struct StartNodeArgs {}

// TODO Karel - use the existing method in fendermint app utils.
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~") {
        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .expect("Could not determine home directory");
        PathBuf::from(home).join(stripped.trim_start_matches(std::path::MAIN_SEPARATOR))
    } else {
        PathBuf::from(path)
    }
}
