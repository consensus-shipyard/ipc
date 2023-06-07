// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! RPC subnet cli command handler.

use async_trait::async_trait;
use clap::Args;
use ipc_gateway::SubnetID;
use std::fmt::Debug;
use std::str::FromStr;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::rpc::RPCSubnetParams;

/// The command to get the RPC endpoint for a subnet
pub struct RPCSubnet;

#[async_trait]
impl CommandLineHandler for RPCSubnet {
    type Arguments = RPCSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("get rpc for subnet with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = RPCSubnetParams {
            subnet: arguments.subnet.clone(),
        };

        let rpc = json_rpc_client
            .request::<String>(json_rpc_methods::RPC_SUBNET, serde_json::to_value(params)?)
            .await?;

        let id = SubnetID::from_str(&arguments.subnet)?;

        log::info!("rpc endpoint for subnet {:}: {:}", arguments.subnet, rpc);
        // todo: We currently have the same ChainID for all subnet. This will be changed
        // once https://github.com/consensus-shipyard/lotus/issues/178 is implemented
        log::info!(
            "chainID for subnet {:}: {:}",
            arguments.subnet,
            id.chain_id(),
        );

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(name = "rpc", about = "RPC endpoint for a subnet")]
pub struct RPCSubnetArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The subnet to get the RPC from")]
    pub subnet: String,
}
