// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet new cli handler

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;
use std::str::FromStr;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::wallet::WalletType;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::wallet::new::{NewFvmWallet, WalletNewParams, WalletNewResponse};

pub(crate) struct WalletNew;

#[async_trait]
impl CommandLineHandler for WalletNew {
    type Arguments = WalletNewArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("create new wallet with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let wallet_type = WalletType::from_str(&arguments.wallet_type)?;
        let params = match wallet_type {
            WalletType::Evm => WalletNewParams::Evm,
            WalletType::Fvm => WalletNewParams::Fvm(NewFvmWallet {
                key_type: arguments.key_type.clone().expect("key type not specified"),
            }),
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
        help = "The fvm key type of the wallet (secp256k1, bls, secp256k1-ledger), only for fvm wallet type"
    )]
    pub key_type: Option<String>,
    #[arg(long, short, help = "The type of the wallet, i.e. fvm, evm")]
    pub wallet_type: String,
}
