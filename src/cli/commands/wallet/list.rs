// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet new cli handler

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::wallet::list::{WalletListParams, WalletListResponse};

pub(crate) struct WalletList;

#[async_trait]
impl CommandLineHandler for WalletList {
    type Arguments = WalletListArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("list wallets with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = WalletListParams {
            subnet: arguments.subnet.clone(),
        };

        let addrs = json_rpc_client
            .request::<WalletListResponse>(
                json_rpc_methods::WALLET_LIST,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!("wallets in subnet {:} are {:?}", arguments.subnet, addrs);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "List wallets in a subnet")]
pub(crate) struct WalletListArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The subnet to list wallets from")]
    pub subnet: String,
}
