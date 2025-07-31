// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet import cli handler

use anyhow::{bail, ensure, Context, Result};
use async_trait::async_trait;
use clap::{ArgGroup, Args};
use fs_err as fs;
use ipc_wallet::WalletType;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::str::FromStr;

use crate::{get_ipc_provider, CommandLineHandler, GlobalArguments};

pub(crate) struct WalletImport;

#[async_trait]
impl CommandLineHandler for WalletImport {
    type Arguments = WalletImportArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> Result<()> {
        log::debug!("import wallet with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;
        let imported_wallet = import_wallet(&provider, arguments)?;

        println!("{:?}", imported_wallet.address);
        Ok(())
    }
}

pub(crate) struct ImportedWallet {
    pub address: String,
    pub private_key: Vec<u8>,
}

/// Import a key into the IPC keystore and return the address
pub(crate) fn import_wallet(
    provider: &ipc_provider::IpcProvider,
    arguments: &WalletImportArgs,
) -> Result<ImportedWallet> {
    let wallet_type = WalletType::from_str(&arguments.wallet_type)?;

    // Handle private key import (EVM only)
    if let Some(private_key) = &arguments.private_key {
        ensure!(
            matches!(wallet_type, WalletType::Evm),
            "--private-key only supported by --wallet-type=evm"
        );

        let imported = provider.import_evm_key_from_privkey(private_key)?;
        return Ok(ImportedWallet {
            address: imported.address.to_string(),
            private_key: imported.private_key,
        });
    }

    // Handle file/stdin import
    let keyinfo = match &arguments.path {
        Some(path) => fs::read_to_string(path)
            .with_context(|| format!("Failed to read key file: {:?}", path))?,
        None => bail!("stdin not supported yet"),
    };

    let (address, private_key) = match wallet_type {
        WalletType::Fvm => {
            let imported = provider.import_fvm_key(&keyinfo)?;
            (imported.address.to_string(), imported.private_key)
        }
        WalletType::Evm => {
            // Try as private key first, fall back to JSON format
            let imported = provider
                .import_evm_key_from_privkey(&keyinfo)
                .or_else(|_| provider.import_evm_key_from_json(&keyinfo))?;
            (imported.address.to_string(), imported.private_key)
        }
    };

    Ok(ImportedWallet {
        address,
        private_key,
    })
}

#[derive(Debug, Args, Deserialize, Serialize)]
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
