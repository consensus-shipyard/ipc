// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet remove cli handler

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;
use std::str::FromStr;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::wallet::remove::WalletRemoveParams;
use crate::server::wallet::WalletType;

pub(crate) struct WalletRemove;

#[async_trait]
impl CommandLineHandler for WalletRemove {
    type Arguments = WalletRemoveArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("remove wallet with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let wallet_type = WalletType::from_str(&arguments.wallet_type)?;
        let params = WalletRemoveParams {
            wallet_type,
            address: arguments.address.clone(),
        };

        json_rpc_client
            .request::<()>(
                json_rpc_methods::WALLET_REMOVE,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!("remove wallet keys for address {:?}", arguments.address);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Create new wallet in subnet")]
pub(crate) struct WalletRemoveArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "Address of the key to remove")]
    pub address: String,
    #[arg(long, short, help = "The type of the wallet, i.e. fvm, evm")]
    pub wallet_type: String,
}
