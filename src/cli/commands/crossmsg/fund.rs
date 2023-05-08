// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Fund cli command handler.

use async_trait::async_trait;
use clap::Args;
use fvm_shared::clock::ChainEpoch;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::fund::FundParams;

/// The command to send funds to a subnet from parent
pub(crate) struct Fund;

#[async_trait]
impl CommandLineHandler for Fund {
    type Arguments = FundArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("fund operation with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = FundParams {
            subnet: arguments.subnet.clone(),
            from: arguments.from.clone(),
            amount: arguments.amount,
        };
        let epoch = json_rpc_client
            .request::<ChainEpoch>(json_rpc_methods::FUND, serde_json::to_value(params)?)
            .await?;

        log::info!("funded subnet: {:} at epoch: {epoch:?}", arguments.subnet);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Send funds from a parent to a child subnet")]
pub(crate) struct FundArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The address to send funds from and to")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to fund")]
    pub subnet: String,
    #[arg(help = "The amount to fund in FIL, in whole FIL")]
    pub amount: f64,
}
