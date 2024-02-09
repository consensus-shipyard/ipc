// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet list cli handler

use async_trait::async_trait;
use clap::Args;
use ipc_wallet::{EthKeyAddress, EvmKeyStore};
use std::fmt::Debug;

use crate::{get_ipc_provider, CommandLineHandler, GlobalArguments};

pub(crate) struct WalletList;

#[async_trait]
impl CommandLineHandler for WalletList {
    type Arguments = WalletListArgs;

    async fn handle(global: &GlobalArguments, _: &Self::Arguments) -> anyhow::Result<()> {
        let provider = get_ipc_provider(global)?;
        let wallet = provider.evm_wallet()?;
        let addresses = wallet.read().unwrap().list()?;
        for address in addresses.iter() {
            if *address == EthKeyAddress::default() {
                continue;
            }

            print!("Address: {}", address.to_string());

            let key_info = wallet.read().unwrap().get(address)?.unwrap();
            let sk = libsecp256k1::SecretKey::parse_slice(key_info.private_key())?;
            let pub_key =
                hex::encode(libsecp256k1::PublicKey::from_secret_key(&sk).serialize()).to_string();

            println!("\tPubKey: {}", pub_key);
        }
        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "List addresses and pubkeys in the wallet")]
pub(crate) struct WalletListArgs {}
