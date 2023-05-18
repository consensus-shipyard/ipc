// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet import cli handler

use async_trait::async_trait;
use clap::Args;
use fvm_shared::crypto::signature::SignatureType;
use serde::Deserialize;
use std::fmt::Debug;
use std::str::FromStr;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::lotus::message::wallet::WalletKeyType;
use crate::server::wallet::import::{WalletImportParams, WalletImportResponse};

pub(crate) struct WalletImport;

#[async_trait]
impl CommandLineHandler for WalletImport {
    type Arguments = WalletImportArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("import wallet with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        // Get keyinfo from file or stdin
        let keyinfo = if arguments.path.is_some() {
            std::fs::read_to_string(arguments.path.as_ref().unwrap())?
        } else {
            // FIXME: Accept keyinfo from stdin
            return Err(anyhow::anyhow!("stdin not supported yet"));
        };

        let params: LotusJsonKeyType = serde_json::from_str(&keyinfo)?;
        let addr = json_rpc_client
            .request::<WalletImportResponse>(
                json_rpc_methods::WALLET_IMPORT,
                serde_json::to_value(WalletImportParams {
                    key_type: SignatureType::try_from(WalletKeyType::from_str(&params.r#type)?)?
                        as u8,
                    private_key: params.private_key,
                })?,
            )
            .await?;

        log::info!("imported wallet with address {:?}", addr);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Import a key into the agent's wallet")]
pub(crate) struct WalletImportArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "Path of keyinfo file for the key to import")]
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LotusJsonKeyType {
    r#type: String,
    private_key: String,
}
