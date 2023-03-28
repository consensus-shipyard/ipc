// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The command to set the validator net address

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::net_addr::SetValidatorNetAddrParams;
use async_trait::async_trait;
use clap::Args;

/// Setting the validator net address
pub(crate) struct SetValidatorNetAddr;

#[async_trait]
impl CommandLineHandler for SetValidatorNetAddr {
    type Arguments = SetValidatorNetAddrArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("set the validator net addr args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = SetValidatorNetAddrParams {
            subnet: arguments.subnet.clone(),
            from: arguments.from.clone(),
            validator_net_addr: arguments.validator_net_addr.clone(),
        };

        json_rpc_client
            .request::<()>(
                json_rpc_methods::SET_VALIDATOR_NET_ADDR,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!(
            "set the validator net addr to: {:} in subnet: {:}",
            arguments.validator_net_addr,
            arguments.subnet
        );

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Set the validator net address")]
pub(crate) struct SetValidatorNetAddrArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "Owner address of the validator being updated")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to set the validator")]
    pub subnet: String,
    #[arg(long, short, help = "New validator net address")]
    pub validator_net_addr: String,
}
