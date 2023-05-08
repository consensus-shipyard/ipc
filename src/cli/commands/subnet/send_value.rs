// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! SendValue cli handler

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::send_value::SendValueParams;

pub(crate) struct SendValue;

#[async_trait]
impl CommandLineHandler for SendValue {
    type Arguments = SendValueArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("send value in subnet with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        // The json rpc server will handle directing the request to
        // the correct parent.
        let params = SendValueParams {
            subnet: arguments.subnet.clone(),
            from: arguments.from.clone(),
            to: arguments.to.clone(),
            amount: arguments.amount,
        };

        json_rpc_client
            .request::<()>(json_rpc_methods::SEND_VALUE, serde_json::to_value(params)?)
            .await?;

        log::info!("sending value in subnet: {:}", arguments.subnet);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Send value to an address within a subnet")]
pub(crate) struct SendValueArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The address to send value from")]
    pub from: Option<String>,
    #[arg(long, short, help = "The address to send value to")]
    pub to: String,
    #[arg(long, short, help = "The subnet of the addresses")]
    pub subnet: String,
    #[arg(help = "The amount to send (in whole FIL units)")]
    pub amount: f64,
}
