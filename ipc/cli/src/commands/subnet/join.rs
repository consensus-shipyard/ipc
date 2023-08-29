// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Join subnet cli command handler.

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::sdk::IpcAgentClient;
use crate::server::join::JoinSubnetParams;

/// The command to join a subnet
pub struct JoinSubnet;

#[async_trait]
impl CommandLineHandler for JoinSubnet {
    type Arguments = JoinSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("join subnet with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;

        // The json rpc server will handle directing the request to
        // the correct parent.
        let params = JoinSubnetParams {
            subnet: arguments.subnet.clone(),
            from: arguments.from.clone(),
            collateral: arguments.collateral,
            validator_net_addr: arguments.validator_net_addr.clone(),
            worker_addr: arguments.worker_addr.clone(),
        };

        let client = IpcAgentClient::default_from_url(url);
        client.join_subnet(params).await?;

        log::info!("joined subnet: {:}", arguments.subnet);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(name = "join", about = "Join a subnet")]
pub struct JoinSubnetArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The address that joins the subnet")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to join")]
    pub subnet: String,
    #[arg(
        long,
        short,
        help = "The collateral to stake in the subnet (in whole FIL units)"
    )]
    pub collateral: f64,
    #[arg(long, short, help = "The validator net address")]
    pub validator_net_addr: String,
    #[arg(
        long,
        short,
        help = "The validator worker address. If not set will be the same as `from`"
    )]
    pub worker_addr: Option<String>,
}
