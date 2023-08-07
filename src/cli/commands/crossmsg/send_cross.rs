// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Send cross message cli command handler.

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::send_cross::SendCrossMsgParams;

/// The command to whitelist a propagator for a message in a postbox
pub(crate) struct SendCrossMsg;

#[async_trait]
impl CommandLineHandler for SendCrossMsg {
    type Arguments = SendCrossMsgsArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("send cross message with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let cross_message = serde_json::from_str(&arguments.cross_msg)?;
        let params = SendCrossMsgParams {
            subnet: arguments.subnet.clone(),
            from: arguments.from.clone(),
            cross_message,
        };
        json_rpc_client
            .request::<()>(
                json_rpc_methods::SEND_CROSS_MSG,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!("whitelisted subnet: {:}", arguments.subnet);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Whitelist propagators in the gateway actor")]
pub(crate) struct SendCrossMsgsArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The address that owns the message in the subnet")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to whitelist")]
    pub subnet: String,
    #[arg(help = "The cross network message to send")]
    pub cross_msg: String,
}
