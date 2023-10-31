// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Release cli command handler.

use async_trait::async_trait;
use clap::Args;
use ipc_sdk::subnet_id::SubnetID;
use std::{fmt::Debug, str::FromStr};

use crate::{
    f64_to_token_amount, get_ipc_provider, require_fil_addr_from_str, CommandLineHandler,
    GlobalArguments,
};

/// The command to release funds from a child to a parent
pub(crate) struct Release;

#[async_trait]
impl CommandLineHandler for Release {
    type Arguments = ReleaseArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("release operation with args: {:?}", arguments);

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
            Some(address) => Some(require_fil_addr_from_str(address)?),
            None => None,
        };
        let fee = match &arguments.fee {
            Some(f) => Some(f64_to_token_amount(*f)?),
            None => None,
        };

        println!(
            "release performed in epoch: {:?}",
            provider
                .release(
                    subnet,
                    gateway_addr,
                    from,
                    to,
                    f64_to_token_amount(arguments.amount)?,
                    fee,
                )
                .await?,
        );

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Release operation in the gateway actor")]
pub(crate) struct ReleaseArgs {
    #[arg(long, short, help = "The gateway address of the subnet")]
    pub gateway_address: Option<String>,
    #[arg(long, short, help = "The address that releases funds")]
    pub from: Option<String>,
    #[arg(
        long,
        short,
        help = "The address to release funds to (if not set, amount sent to from address)"
    )]
    pub to: Option<String>,
    #[arg(long, short, help = "The subnet to release funds from")]
    pub subnet: String,
    #[arg(long, help = "The fee to pay for the cross-net message, in whole FIL")]
    pub fee: Option<f64>,
    #[arg(help = "The amount to release in FIL, in whole FIL")]
    pub amount: f64,
}
