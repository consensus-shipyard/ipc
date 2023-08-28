// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet export cli handler
use anyhow::anyhow;
use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use clap::Args;
use fvm_shared::address::Address;
use ipc_identity::{EvmKeyStore, PersistentKeyInfo, Wallet};
use std::fmt::Debug;
use std::io::Write;
use std::str::FromStr;

use crate::cli::get_evm_keystore;
use crate::sdk::LotusJsonKeyType;
use crate::server::wallet::WalletType;
use crate::{
    cli::{get_fvm_store, CommandLineHandler, GlobalArguments},
    lotus::message::wallet::WalletKeyType,
};

pub(crate) struct WalletExport;

impl WalletExport {
    fn export_evm(arguments: &WalletExportArgs) -> anyhow::Result<String> {
        let keystore = get_evm_keystore(&arguments.keystore)?;
        let address = ethers::types::Address::from_str(&arguments.address)?;

        let key_info = keystore
            .get(&address)?
            .ok_or_else(|| anyhow!("key does not exists"))?;

        let info = PersistentKeyInfo::new(
            format!("{:?}", address),
            hex::encode(key_info.private_key()),
        );
        Ok(serde_json::to_string(&info)?)
    }

    fn export_fvm(arguments: &WalletExportArgs) -> anyhow::Result<String> {
        let mut wallet = Wallet::new(get_fvm_store(arguments.keystore.clone())?);

        let addr = Address::from_str(&arguments.address)?;
        let key_info = wallet.export(&addr)?;
        Ok(serde_json::to_string(&LotusJsonKeyType {
            r#type: WalletKeyType::try_from(*key_info.key_type())?.to_string(),
            private_key: BASE64_STANDARD.encode(key_info.private_key()),
        })?)
    }
}

#[async_trait]
impl CommandLineHandler for WalletExport {
    type Arguments = WalletExportArgs;

    async fn handle(_global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("export wallet with args: {:?}", arguments);

        let wallet_type = WalletType::from_str(&arguments.wallet_type)?;
        let v = match wallet_type {
            WalletType::Evm => WalletExport::export_evm(arguments),
            WalletType::Fvm => WalletExport::export_fvm(arguments),
        }?;

        match &arguments.output {
            Some(p) => {
                let mut file = std::fs::File::create(p)?;
                file.write_all(v.as_bytes())?;
                log::info!(
                    "exported new wallet with address {:?} in file {:?}",
                    arguments.address,
                    p
                );
            }
            None => {
                log::info!("exported new wallet with address {:?}", arguments.address);
                log::info!("Key: {:?}", v);
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
    #[arg(long, short, help = "The type of the wallet, i.e. fvm, evm")]
    pub wallet_type: String,
}
