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
use crate::server::wallet::new::{WalletNewParams, WalletNewResponse};

pub(crate) struct WalletNew;

#[async_trait]
impl CommandLineHandler for WalletNew {
    type Arguments = WalletNewArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("create new wallet with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = WalletNewParams {
            key_type: arguments.key_type.clone(),
        };

        let addr = json_rpc_client
            .request::<WalletNewResponse>(
                json_rpc_methods::WALLET_NEW,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!("created new wallet with address {:?}", addr,);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Create new wallet in subnet")]
pub(crate) struct WalletNewArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(
        long,
        short,
        help = "Key type of the wallet (secp256k1, bls, secp256k1-ledger)"
    )]
    pub key_type: String,
}
