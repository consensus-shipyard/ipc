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
use crate::server::wallet::balances::{WalletBalancesParams, WalletBalancesResponse};

pub(crate) struct WalletBalances;

#[async_trait]
impl CommandLineHandler for WalletBalances {
    type Arguments = WalletBalancesArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("list wallets with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = WalletBalancesParams {
            subnet: arguments.subnet.clone(),
        };

        let addrs = json_rpc_client
            .request::<WalletBalancesResponse>(
                json_rpc_methods::WALLET_BALANCES,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!("wallets in subnet {:} are {:?}", arguments.subnet, addrs);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "List balance of wallets in a subnet")]
pub(crate) struct WalletBalancesArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The subnet to list wallets from")]
    pub subnet: String,
}
