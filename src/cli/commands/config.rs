// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! This mod triggers a config reload in the IPC-Agent Json RPC server.

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;
use std::io::Write;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::{json_rpc_methods, DEFAULT_CONFIG_TEMPLATE};
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::ReloadConfigParams;

/// The command to reload the agent config after an update
pub(crate) struct ReloadConfig;

#[async_trait]
impl CommandLineHandler for ReloadConfig {
    type Arguments = ReloadConfigArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("reload config with args: {:?}", arguments);

        let params = ReloadConfigParams {
            path: arguments.path.clone(),
        };

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        json_rpc_client
            .request::<()>(
                json_rpc_methods::RELOAD_CONFIG,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!("Reload json rpc config successful");

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Reload config for IPC Agent JSON RPC server")]
pub(crate) struct ReloadConfigArgs {
    #[arg(
        short,
        long,
        help = "The path to ask json rpc server to load config from, optional"
    )]
    pub path: Option<String>,
    #[arg(short, long, help = "The JSON RPC server url for ipc agent, optional")]
    pub ipc_agent_url: Option<String>,
}

/// The command to initialize a new config template in a specific path
pub(crate) struct InitConfig;

#[async_trait]
impl CommandLineHandler for InitConfig {
    type Arguments = InitConfigArgs;

    async fn handle(global: &GlobalArguments, _arguments: &Self::Arguments) -> anyhow::Result<()> {
        let path = global.config_path();
        log::debug!("initializing empty config file in {}", path);

        let file_path = std::path::Path::new(&path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(&path).map_err(|e| {
            log::error!("couldn't create config file");
            e
        })?;
        file.write_all(DEFAULT_CONFIG_TEMPLATE.as_bytes())
            .map_err(|e| {
                log::error!("error populating empty config template");
                e
            })?;

        log::info!("Empty config populated successful in {}", &path);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Arguments to initialize a new empty config file")]
pub(crate) struct InitConfigArgs {}
