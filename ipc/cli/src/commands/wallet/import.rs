// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet import cli handler

use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::{ArgGroup, Args};
use fs_err as fs;
use ipc_wallet::WalletType;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::path::PathBuf;
use std::str::FromStr;

use crate::errors::{CliError, WalletError};
use crate::{get_ipc_provider, CommandLineHandler, GlobalArguments};

pub(crate) struct WalletImport;

#[async_trait]
impl CommandLineHandler for WalletImport {
    type Arguments = WalletImportArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> Result<()> {
        log::debug!("import wallet with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;
        let imported_wallet = import_wallet(&provider, arguments)
            .map_err(|e| match e.downcast::<CliError>() {
                Ok(cli_err) => cli_err,
                Err(e) => CliError::Internal(e.to_string()),
            })?;

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
    let wallet_type = WalletType::from_str(&arguments.wallet_type)
        .map_err(|_| WalletError::UnsupportedWalletType {
            wallet_type: arguments.wallet_type.clone(),
        })?;

    // Handle private key import (EVM only)
    if let Some(private_key) = &arguments.private_key {
        if !matches!(wallet_type, WalletType::Evm) {
            return Err(WalletError::PrivateKeyOnlyForEvm.into());
        }

        let imported = provider
            .import_evm_key_from_privkey(private_key)
            .map_err(|e| WalletError::InvalidKeyFormat {
                wallet_type: "evm".to_string(),
                details: e.to_string(),
                expected_format: "A 64-character hexadecimal string (32 bytes) without 0x prefix\nExample: e88d7f1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            })?;
        return Ok(ImportedWallet {
            address: imported.address.to_string(),
            private_key: imported.private_key,
        });
    }

    // Handle file/stdin import
    let keyinfo = match &arguments.path {
        Some(path) => {
            let path_buf = PathBuf::from(path);
            fs::read_to_string(&path_buf)
                .map_err(|_| WalletError::CannotReadKeyFile { path: path_buf })?
        }
        None => {
            return Err(WalletError::StdinNotSupported {
                wallet_type: arguments.wallet_type.clone(),
            }
            .into())
        }
    };

    let (address, private_key) = match wallet_type {
        WalletType::Fvm => {
            let imported = provider.import_fvm_key(&keyinfo).map_err(|e| {
                WalletError::InvalidKeyFormat {
                    wallet_type: "fvm".to_string(),
                    details: e.to_string(),
                    expected_format: "Base64-encoded key info JSON\nExample format:\n{\n  \"Type\": \"secp256k1\",\n  \"PrivateKey\": \"<base64-encoded-key>\"\n}".to_string(),
                }
            })?;
            (imported.address.to_string(), imported.private_key)
        }
        WalletType::Evm => {
            // Try as private key first, fall back to JSON format
            let imported = provider
                .import_evm_key_from_privkey(&keyinfo)
                .or_else(|_| provider.import_evm_key_from_json(&keyinfo))
                .map_err(|e| WalletError::InvalidKeyFormat {
                    wallet_type: "evm".to_string(),
                    details: e.to_string(),
                    expected_format: "Either:\n1. A 64-character hex private key\n2. A JSON keyfile with encrypted private key".to_string(),
                })?;
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
