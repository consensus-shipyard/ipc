// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::{CommandLineHandler, GlobalArguments};
use async_trait::async_trait;
use ipc_provider::config::Config;

use clap::Args;

/// The command to initialize a new config template in a specific path
pub(crate) struct InitConfig;

#[async_trait]
impl CommandLineHandler for InitConfig {
    type Arguments = InitConfigArgs;

    async fn handle(global: &GlobalArguments, _arguments: &Self::Arguments) -> anyhow::Result<()> {
        let path = global.config_path();
        log::debug!("initializing empty config file in {}", path);

        let config = Config::default();
        config.write_to_file_async(&path).await?;

        log::info!("Empty config populated successful in {}", &path);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Arguments to initialize a new empty config file")]
pub(crate) struct InitConfigArgs {}
