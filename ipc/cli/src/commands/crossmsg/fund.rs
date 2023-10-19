// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Fund cli command handler.

use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use std::{fmt::Debug, str::FromStr};

use crate::{
    f64_to_token_amount, get_ipc_provider, require_fil_addr_from_str, CommandLineHandler,
    GlobalArguments,
};

/// The command to send funds to a subnet from parent
pub(crate) struct Fund;

#[async_trait]
impl CommandLineHandler for Fund {
    type Arguments = FundArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("fund operation with args: {:?}", arguments);

        let mut provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let from = match &arguments.from {
            Some(address) => Some(require_fil_addr_from_str(address)?),
            None => None,
        };
        let to = match &arguments.to {
            Some(address) => Some(require_fil_addr_from_str(address)?),
            None => None,
        };
        let gateway_addr = match &arguments.gateway_address {
            Some(address) => Some(Address::from_str(address)?),
            None => None,
        };

        println!(
            "fund performed in epoch: {:?}",
            provider
                .fund(
                    subnet,
                    gateway_addr,
                    from,
                    to,
                    f64_to_token_amount(arguments.amount)?,
                )
                .await?,
        );

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Send funds from a parent to a child subnet")]
pub(crate) struct FundArgs {
    #[arg(long, short, help = "The gateway address of the subnet")]
    pub gateway_address: Option<String>,
    #[arg(long, short, help = "The address to send funds from")]
    pub from: Option<String>,
    #[arg(
        long,
        short,
        help = "The address to send funds to (if not set, amount sent to from address)"
    )]
    pub to: Option<String>,
    #[arg(long, short, help = "The subnet to fund")]
    pub subnet: String,
    #[arg(help = "The amount to fund in FIL, in whole FIL")]
    pub amount: f64,
}
