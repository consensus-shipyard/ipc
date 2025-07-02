// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet balances cli handler

use crate::{get_ipc_provider, CommandLineHandler, GlobalArguments};
use async_trait::async_trait;
use clap::Args;
use futures_util::future::join_all;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_api::ethers_address_to_fil_address;
use ipc_api::subnet_id::SubnetID;
use ipc_wallet::{evm::adapter::EthKeyAddress, WalletType};
use std::{fmt::Debug, str::FromStr};

pub(crate) struct WalletBalances;

#[async_trait]
impl CommandLineHandler for WalletBalances {
    type Arguments = WalletBalancesArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("list wallets with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;

        let wallet_type = WalletType::from_str(&arguments.wallet_type)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let mut errors = Vec::new();

        match wallet_type {
            WalletType::Etherium => {
                let wallet = provider.evm_wallet()?;
                let addresses = wallet.read().unwrap().list();
                let r = Vec::from_iter(addresses.iter().map(|addr| {
                    let provider = provider.clone();
                    let subnet = subnet.clone();
                    async move {
                        let addr_eth = EthKeyAddress::from_str(addr)?;
                        provider
                            .wallet_balance(
                                &subnet,
                                &ethers_address_to_fil_address(addr_eth.clone().into())?,
                            )
                            .await
                            .map(|balance| (balance, addr_eth))
                    }
                }));

                let v: Vec<anyhow::Result<(TokenAmount, EthKeyAddress)>> = join_all(r).await;

                for r in v.into_iter() {
                    match r {
                        Ok((balance, addr)) => {
                            if addr.to_string() != "default-key" {
                                println!("{} - Balance: {}", addr, balance);
                            }
                        }
                        Err(e) => {
                            errors.push(e);
                        }
                    }
                }

                if !errors.is_empty() {
                    let error = errors
                        .into_iter()
                        .fold(anyhow::anyhow!("Error fetching balances"), |acc, err| {
                            acc.context(err)
                        });
                    return Err(error);
                }
            }
            WalletType::Filecoin => {
                let wallet = provider.fvm_wallet()?;
                let addresses = wallet.read().unwrap().list_addrs()?;
                let r = Vec::from_iter(addresses.iter().map(|addr| {
                    let provider = provider.clone();
                    let subnet = subnet.clone();
                    async move {
                        provider
                            .wallet_balance(&subnet, addr)
                            .await
                            .map(|balance| (balance, addr))
                    }
                }));

                let r = anyhow::Result::<Vec<(TokenAmount, &Address)>>::from_iter(
                    join_all(r).await.into_iter(),
                )?;
                for (balance, addr) in r {
                    println!("{:?} - Balance: {}", addr, balance);
                }
            }
        };

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "List balance of wallets in a subnet")]
pub(crate) struct WalletBalancesArgs {
    #[arg(long, help = "The subnet to list wallets from")]
    pub subnet: String,
    #[arg(long, help = "The type of the wallet, i.e. fvm, evm")]
    pub wallet_type: String,
}
