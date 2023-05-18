// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Release cli command handler.

use async_trait::async_trait;
use clap::Args;
use fvm_shared::clock::ChainEpoch;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::release::ReleaseParams;

/// The command to release funds from a child to a parent
pub(crate) struct Release;

#[async_trait]
impl CommandLineHandler for Release {
    type Arguments = ReleaseArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("release operation with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = ReleaseParams {
            subnet: arguments.subnet.clone(),
            from: arguments.from.clone(),
            amount: arguments.amount,
        };
        let epoch = json_rpc_client
            .request::<ChainEpoch>(json_rpc_methods::RELEASE, serde_json::to_value(params)?)
            .await?;

        log::info!("released subnet: {:} at epoch {epoch:}", arguments.subnet);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Release operation in the gateway actor")]
pub(crate) struct ReleaseArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The address that releases funds")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to release funds from")]
    pub subnet: String,
    #[arg(help = "The amount to release in FIL, in whole FIL")]
    pub amount: f64,
}
