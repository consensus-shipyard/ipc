// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet export cli handler

use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use ipc_identity::json::KeyInfoJson;
use ipc_identity::Wallet;
use std::fmt::Debug;
use std::str::FromStr;

use crate::cli::{get_keystore, CommandLineHandler, GlobalArguments};

pub(crate) struct WalletExport;

#[async_trait]
impl CommandLineHandler for WalletExport {
    type Arguments = WalletExportArgs;

    async fn handle(_global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("export wallet with args: {:?}", arguments);

        let mut wallet = Wallet::new(get_keystore(&arguments.path)?);

        let addr = Address::from_str(&arguments.address)?;
        let key_info = wallet.export(&addr)?;

        log::info!("exported new wallet with address {:?}", addr,);
        log::info!("{:?}", KeyInfoJson(key_info));

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Export the key from a wallet address")]
pub(crate) struct WalletExportArgs {
    #[arg(long, short, help = "Address of the key to export")]
    pub address: String,
    #[arg(
        long,
        short,
        help = "Keystore path (the default repo keystore is used if not specified)"
    )]
    pub path: Option<String>,
}
