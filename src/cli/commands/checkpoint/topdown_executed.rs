// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Last top-down checkpoint executed in subnet

use std::fmt::Debug;

use async_trait::async_trait;
use clap::Args;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::sdk::IpcAgentClient;

/// The command to get the latest epoch executed for a top-down checkpoint
pub(crate) struct LastTopDownExec;

#[async_trait]
impl CommandLineHandler for LastTopDownExec {
    type Arguments = LastTopDownExecArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("last topdown exec with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let client = IpcAgentClient::default_from_url(url);
        let epoch = client.last_top_down_executed(&arguments.subnet).await?;

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
