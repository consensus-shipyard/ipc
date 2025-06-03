// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet list cli handler

use async_trait::async_trait;
use clap::Args;
use ipc_wallet::{evm::adapter::EthKeyAddress, WalletType};
use std::fmt::Debug;
use std::str::FromStr;

use crate::{get_ipc_provider, CommandLineHandler, GlobalArguments};

pub(crate) struct WalletList;

#[async_trait]
impl CommandLineHandler for WalletList {
    type Arguments = WalletListArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        let provider = get_ipc_provider(global)?;
        let wallet_type = WalletType::from_str(&arguments.wallet_type)?;
        match wallet_type {
            WalletType::Etherium => {
                let wallet = provider.evm_wallet()?;
                let guard = wallet.read().unwrap();
                for address in guard.list() {
                    let address = EthKeyAddress::from_str(&address)?;
                    if address == EthKeyAddress::default() {
                        continue;
                    }
                    print!("Address: {}", address);

                    let key_info = guard.get(&address.to_string())?;
                    let sk = libsecp256k1::SecretKey::parse_slice(key_info.private_key())?;
                    let pub_key =
                        hex::encode(libsecp256k1::PublicKey::from_secret_key(&sk).serialize())
                            .to_string();
                    println!("\tPubKey: {}", pub_key);
                }
                Ok(())
            }
            WalletType::Filecoin => {
                let wallet = provider.fvm_wallet()?;
                let guard = wallet.read().unwrap();
                let addresses = guard.list_addrs()?;
                for address in addresses {
                    print!("Address: {}", address);

                    let mut guard = wallet.write().unwrap();
                    let key_info = guard.export(&address)?;
                    let sk = libsecp256k1::SecretKey::parse_slice(key_info.private_key())?;
                    let pub_key =
                        hex::encode(libsecp256k1::PublicKey::from_secret_key(&sk).serialize())
                            .to_string();
                    print!("\tPubKey: {}", pub_key);
                    println!("\tKeyType: {:?}", key_info.key_type());
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Args)]
#[command(about = "List addresses and pubkeys in the wallet")]
pub(crate) struct WalletListArgs {
    #[arg(long, help = "The type of the wallet, i.e. fvm, evm")]
    pub wallet_type: String,
}
