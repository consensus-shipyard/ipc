// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Join subnet cli command handler.

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::JoinSubnetParams;

/// The command to join a subnet
pub(crate) struct JoinSubnet;

#[async_trait]
impl CommandLineHandler for JoinSubnet {
    type Arguments = JoinSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("join subnet with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        // The json rpc server will handle directing the request to
        // the correct parent.
        let params = JoinSubnetParams {
            subnet: arguments.subnet.clone(),
            from: arguments.from.clone(),
            collateral: arguments.collateral,
            validator_net_addr: arguments.validator_net_addr.clone(),
        };

        json_rpc_client
            .request::<()>(json_rpc_methods::JOIN_SUBNET, serde_json::to_value(params)?)
            .await?;

        log::info!("joined subnet: {:}", arguments.subnet);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Join a subnet")]
pub(crate) struct JoinSubnetArgs {
    #[arg(help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(help = "The address that joins the subnet")]
    pub from: Option<String>,
    #[arg(help = "The subnet to join")]
    pub subnet: String,
    #[arg(help = "The collateral to stake in the subnet")]
    pub collateral: u64,
    #[arg(help = "The validator net address")]
    pub validator_net_addr: String,
}
