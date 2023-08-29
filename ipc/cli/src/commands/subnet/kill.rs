// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Kill a subnet cli command handler.

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::sdk::IpcAgentClient;
use crate::server::kill::KillSubnetParams;

/// The command to kill an existing subnet.
pub struct KillSubnet;

#[async_trait]
impl CommandLineHandler for KillSubnet {
    type Arguments = KillSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("kill subnet with args: {:?}", arguments);

        let params = KillSubnetParams {
            subnet: arguments.subnet.clone(),
            from: arguments.from.clone(),
        };

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let client = IpcAgentClient::default_from_url(url);
        client.kill_subnet(params).await?;

        log::info!("killed subnet: {:}", arguments.subnet);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(name = "kill", about = "Kill an existing subnet")]
pub struct KillSubnetArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The address that kills the subnet")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to kill")]
    pub subnet: String,
}
