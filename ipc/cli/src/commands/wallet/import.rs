// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet import cli handler

use anyhow::bail;
use async_trait::async_trait;
use clap::{ArgGroup, Args};
use fs_err as fs;
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

        if let Some(key) = &arguments.private_key {
            if !matches!(wallet_type, WalletType::Etherium) {
                bail!("--private-key only supported by --wallet-type=evm");
            }
            println!(
                "{:?}",
                provider.import_evm_key_from_privkey(key)?.to_string()
            );
            Ok(())
        } else {
            // Get keyinfo from file or stdin
            let (keyinfo_path, keyinfo) = if let Some(path) = arguments.path.as_ref() {
                let content = fs::read_to_string(path)?;
                (std::path::PathBuf::from(path), content)
            } else {
                // FIXME: Accept keyinfo from stdin
                bail!("stdin not supported yet")
            };

            match wallet_type {
                WalletType::Filecoin => println!("{:?}", provider.import_fvm_key(&keyinfo)?),
                WalletType::Etherium => {
                    let key = provider
                        .import_evm_key_from_privkey(&keyinfo)
                        .or_else(|e| {
                            println!(
                                "Failed to import file {p} as key private key{e:?}",
                                e = e,
                                p = keyinfo_path.display()
                            );
                            provider
                                .import_evm_key_from_json(&keyinfo)
                                .inspect_err(|e| {
                                    println!(
                                        "Failed to import file {p} as json key config {e:?}",
                                        e = e,
                                        p = keyinfo_path.display()
                                    );
                                })
                        })?;

                    println!("{:?}", key.to_string())
                }
            };
            Ok(())
        }
    }
}

#[derive(Debug, Args)]
#[command(about = "Import a key into the agent's wallet")]
#[clap(group(ArgGroup::new("key_source")
.required(true)
.multiple(false)
.args(&["path", "private_key"]),
))]
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
