// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Last top-down checkpoint executed in subnet

use std::fmt::Debug;

use async_trait::async_trait;
use clap::Args;
use serde_json::Value;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::topdown_executed::LastTopDownExecParams;

/// The command to get the latest epoch executed for a top-down checkpoint
pub(crate) struct LastTopDownExec;

#[async_trait]
impl CommandLineHandler for LastTopDownExec {
    type Arguments = LastTopDownExecArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("last topdown exec with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = LastTopDownExecParams {
            subnet_id: arguments.subnet.clone(),
        };

        let epoch = json_rpc_client
            .request::<Value>(
                json_rpc_methods::LAST_TOPDOWN_EXECUTED,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!("Last top-down checkpoint executed in epoch: {epoch:}");

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Epoch of the last top-down checkpoint executed")]
pub(crate) struct LastTopDownExecArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The subnet id of the checkpointing subnet")]
    pub subnet: String,
}
