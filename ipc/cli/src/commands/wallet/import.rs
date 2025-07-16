// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet import cli handler

use anyhow::bail;
use async_trait::async_trait;
use clap::{ArgGroup, Args};
use fs_err as fs;
use ipc_wallet::WalletType;
use serde::Deserialize;
use std::fmt::Debug;
use std::str::FromStr;

use crate::{get_ipc_provider, CommandLineHandler, GlobalArguments};

pub(crate) struct WalletImport;

#[async_trait]
impl CommandLineHandler for WalletImport {
    type Arguments = WalletImportArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("import wallet with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;
        let address = import_wallet(&provider, arguments)?;

        println!("{:?}", address);
        Ok(())
    }
}

/// Import a key into the IPC keystore and return the address
pub(crate) fn import_wallet(
    provider: &ipc_provider::IpcProvider,
    arguments: &WalletImportArgs,
) -> anyhow::Result<String> {
    let wallet_type = WalletType::from_str(&arguments.wallet_type)?;

    if let Some(key) = &arguments.private_key {
        if !matches!(wallet_type, WalletType::Evm) {
            bail!("--private-key only supported by --wallet-type=evm");
        }
        let address = provider.import_evm_key_from_privkey(key)?.to_string();
        println!("{:?}", address);
        Ok(address)
    } else {
        // Get keyinfo from file or stdin
        let keyinfo = if arguments.path.is_some() {
            fs::read_to_string(arguments.path.as_ref().unwrap())?
        } else {
            // FIXME: Accept keyinfo from stdin
            bail!("stdin not supported yet")
        };

        let address = match wallet_type {
            WalletType::Fvm => provider.import_fvm_key(&keyinfo)?.to_string(),
            WalletType::Evm => {
                let key = provider
                    .import_evm_key_from_privkey(&keyinfo)
                    .or_else(|_| provider.import_evm_key_from_json(&keyinfo))?;
                key.to_string()
            }
        };

        Ok(address)
    }
}

#[derive(Debug, Args, Deserialize)]
#[command(about = "Import a key into the agent's wallet")]
#[clap(group(ArgGroup::new("key_source")
.required(true)
.multiple(false)
.args(&["path", "private_key"]),
))]
#[serde(rename_all = "kebab-case")]
pub(crate) struct WalletImportArgs {
    #[arg(long, help = "The type of the wallet, i.e. fvm, evm")]
    pub wallet_type: String,
    #[arg(
        long,
        group = "key_source",
        help = "Path of key info file for the key to import"
    )]
    pub path: Option<String>,
    #[arg(
        long,
        group = "key_source",
        help = "The evm private key to import if path is not specified"
    )]
    pub private_key: Option<String>,
}
