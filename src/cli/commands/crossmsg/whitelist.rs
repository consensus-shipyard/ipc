// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Whitelist cli command handler.

use async_trait::async_trait;
use cid::Cid;
use clap::Args;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::whitelist::WhitelistPropagatorParams;

/// The command to whitelist a propagator for a message in a postbox
pub(crate) struct WhitelistPropagator;

#[async_trait]
impl CommandLineHandler for WhitelistPropagator {
    type Arguments = WhitelistPropagatorArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("whitelist operation with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = WhitelistPropagatorParams {
            subnet: arguments.subnet.clone(),
            from: arguments.from.clone(),
            postbox_msg_cid: arguments.postbox_msg_cid,
            to_add: arguments.to_add.clone(),
        };
        json_rpc_client
            .request::<()>(
                json_rpc_methods::WHITELIST_PROPAGATOR,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!("whitelisted subnet: {:}", arguments.subnet);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Whitelist propagators in the gateway actor")]
pub(crate) struct WhitelistPropagatorArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The address that owns the message in the subnet")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to whitelist")]
    pub subnet: String,
    #[arg(help = "The message cid to whitelist")]
    pub postbox_msg_cid: Cid,
    #[arg(help = "The addresses to whitelist")]
    pub to_add: Vec<String>,
}
