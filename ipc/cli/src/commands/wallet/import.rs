// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet import cli handler

use async_trait::async_trait;
use clap::{ArgGroup, Args};
use ipc_wallet::WalletType;
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
        let wallet_type = WalletType::from_str(&arguments.wallet_type)?;

        if matches!(wallet_type, WalletType::Evm) {
            if let Some(key) = &arguments.private_key {
                let hex_encoded = arguments.hex || !arguments.fendermint;
                println!(
                    "{:?}",
                    provider
                        .import_evm_key_from_privkey(key.to_string(), hex_encoded)?
                        .to_string()
                );
                Ok(())
            } else {
                let keyinfo = std::fs::read_to_string(arguments.path.as_ref().unwrap())?;
                println!(
                    "{:?}",
                    provider.import_evm_key_from_json(keyinfo)?.to_string()
                );
                Ok(())
            }
        } else {
            // Get keyinfo from file or stdin
            let keyinfo = if arguments.path.is_some() {
                std::fs::read_to_string(arguments.path.as_ref().unwrap())?
            } else {
                // FIXME: Accept keyinfo from stdin
                return Err(anyhow::anyhow!("stdin not supported yet"));
            };

            println!("{:?}", provider.import_fvm_key(keyinfo)?);
            Ok(())
        }
    }
}

#[derive(Debug, Args)]
#[command(about = "Import a key into the agent's wallet")]
#[clap(
group(ArgGroup::new("key_source").required(true).multiple(false).args(&["path", "private_key"])),
group(ArgGroup::new("key_encoding").required(false).multiple(false).args(&["fendermint", "hex"])),
)]
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
    #[arg(
        long,
        help = "Imports a secret key encoded in base64 as Fendermint expects"
    )]
    pub fendermint: bool,
    #[arg(long, help = "Imports a hex encoded secret key")]
    pub hex: bool,
}
