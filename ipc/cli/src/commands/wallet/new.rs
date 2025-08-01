// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet new cli handler

use async_trait::async_trait;
use clap::Args;
use ipc_provider::lotus::message::wallet::WalletKeyType;
use ipc_wallet::WalletType;
use std::fmt::Debug;
use std::str::FromStr;

use crate::errors::{CliError, WalletError};
use crate::{get_ipc_provider, CommandLineHandler, GlobalArguments};

pub(crate) struct WalletNew;

#[async_trait]
impl CommandLineHandler for WalletNew {
    type Arguments = WalletNewArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("create new wallet with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;

        let wallet_type = WalletType::from_str(&arguments.wallet_type)
            .map_err(|_| WalletError::UnsupportedWalletType {
                wallet_type: arguments.wallet_type.clone(),
            })?;
            
        match wallet_type {
            WalletType::Evm => {
                let address = provider.new_evm_key()
                    .map_err(|e| CliError::Internal(format!("Failed to create EVM wallet: {}", e)))?;
                println!("{:?}", address.to_string());
            }
            WalletType::Fvm => {
                let key_type_str = arguments
                    .key_type
                    .as_ref()
                    .ok_or(WalletError::FvmKeyTypeRequired)?;
                    
                let tp = WalletKeyType::from_str(key_type_str)
                    .map_err(|_| CliError::Internal(format!(
                        "Invalid FVM key type '{}'. Valid types are: secp256k1, bls, secp256k1-ledger",
                        key_type_str
                    )))?;
                    
                let address = provider.new_fvm_key(tp)
                    .map_err(|e| CliError::Internal(format!("Failed to create FVM wallet: {}", e)))?;
                println!("{:?}", address)
            }
        };

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Create new wallet in subnet")]
pub(crate) struct WalletNewArgs {
    #[arg(
        long,
        help = "The fvm key type of the wallet (secp256k1, bls, secp256k1-ledger), only for fvm wallet type"
    )]
    pub key_type: Option<String>,
    #[arg(long, help = "The type of the wallet, i.e. fvm, evm")]
    pub wallet_type: String,
}
