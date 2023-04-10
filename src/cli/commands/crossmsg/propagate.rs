// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Propagate cli command handler.

use async_trait::async_trait;
use cid::Cid;
use clap::Args;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::propagate::PropagateParams;

/// The command to propagate a message in the postbox.
pub(crate) struct Propagate;

#[async_trait]
impl CommandLineHandler for Propagate {
    type Arguments = PropagateArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("propagate operation with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = PropagateParams {
            subnet: arguments.subnet.clone(),
            from: arguments.from.clone(),
            postbox_msg_cid: arguments.postbox_msg_cid,
        };
        json_rpc_client
            .request::<()>(json_rpc_methods::PROPAGATE, serde_json::to_value(params)?)
            .await?;

        log::info!("propagated subnet: {:}", arguments.subnet);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Propagate operation in the gateway actor")]
pub(crate) struct PropagateArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The address that pays for the propagation gas")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet of the message to propagate")]
    pub subnet: String,
    #[arg(help = "The message cid to propagate")]
    pub postbox_msg_cid: Cid,
}
