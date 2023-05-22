// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet export cli handler
use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use clap::Args;
use fvm_shared::address::Address;
use ipc_identity::Wallet;
use std::fmt::Debug;
use std::io::Write;
use std::str::FromStr;

use crate::cli::commands::wallet::LotusJsonKeyType;
use crate::{
    cli::{get_keystore, CommandLineHandler, GlobalArguments},
    lotus::message::wallet::WalletKeyType,
};

pub(crate) struct WalletExport;

#[async_trait]
impl CommandLineHandler for WalletExport {
    type Arguments = WalletExportArgs;

    async fn handle(_global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("export wallet with args: {:?}", arguments);

        let mut wallet = Wallet::new(get_keystore(arguments.keystore.clone())?);

        let addr = Address::from_str(&arguments.address)?;
        let key_info = wallet.export(&addr)?;
        let ser_key = serde_json::to_string(&LotusJsonKeyType {
            r#type: WalletKeyType::try_from(*key_info.key_type())?.to_string(),
            private_key: BASE64_STANDARD.encode(key_info.private_key()),
        })?;

        match &arguments.output {
            Some(p) => {
                let mut file = std::fs::File::create(p)?;
                file.write_all(ser_key.as_bytes())?;
                log::info!(
                    "exported new wallet with address {:?} in file {:?}",
                    addr,
                    p
                );
            }
            None => {
                log::info!("exported new wallet with address {:?}", addr);
                log::info!("Key: {:?}", ser_key);
            }
        }

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
    pub keystore: Option<String>,
    #[arg(
        long,
        short,
        help = "Optional parameter that outputs the address key into the file specified"
    )]
    pub output: Option<String>,
}
