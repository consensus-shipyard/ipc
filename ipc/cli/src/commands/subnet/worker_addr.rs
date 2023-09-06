// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The command to set the validator worker address

use crate::commands::get_ipc_agent_url;
use crate::config::json_rpc_methods;
use crate::server::worker_addr::SetValidatorWorkerAddrParams;
use crate::{CommandLineHandler, GlobalArguments};
use async_trait::async_trait;
use clap::Args;
use ipc_provider::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};

/// Setting the validator worker address
pub(crate) struct SetValidatorWorkerAddr;

#[async_trait]
impl CommandLineHandler for SetValidatorWorkerAddr {
    type Arguments = SetValidatorWorkerAddrArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("set the validator worker addr args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = SetValidatorWorkerAddrParams {
            subnet: arguments.subnet.clone(),
            from: arguments.from.clone(),
            validator_worker_addr: arguments.validator_worker_addr.clone(),
        };

        json_rpc_client
            .request::<()>(
                json_rpc_methods::SET_VALIDATOR_WORKER_ADDR,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!(
            "set the validator worker addr to: {:} in subnet: {:}",
            arguments.validator_worker_addr,
            arguments.subnet
        );

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Set the validator worker address")]
pub(crate) struct SetValidatorWorkerAddrArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "Owner address of the validator being updated")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to set the validator")]
    pub subnet: String,
    #[arg(long, short, help = "New validator worker address")]
    pub validator_worker_addr: String,
}
