// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! SendValue cli handler

use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use ipc_provider::manager::evm::ethers_address_to_fil_address;
use ipc_sdk::subnet_id::SubnetID;
use std::{fmt::Debug, str::FromStr};

use crate::{f64_to_token_amount, get_ipc_provider, CommandLineHandler, GlobalArguments};

pub(crate) struct SendValue;

#[async_trait]
impl CommandLineHandler for SendValue {
    type Arguments = SendValueArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("send value in subnet with args: {:?}", arguments);

        let mut provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let from = match &arguments.from {
            Some(address) => Some(Address::from_str(address)?),
            None => None,
        };

        // try to get the `to` as an FVM address and an Eth
        // address. We should include a wrapper type to make
        // this easier through the whole code base.
        let to = match Address::from_str(&arguments.to) {
            Err(_) => {
                // see if it is an eth address
                let addr = ethers::types::Address::from_str(&arguments.to)?;
                ethers_address_to_fil_address(&addr)?
            }
            Ok(addr) => addr,
        };

        provider
            .send_value(&subnet, from, to, f64_to_token_amount(arguments.amount)?)
            .await
    }
}

#[derive(Debug, Args)]
#[command(about = "Send value to an address within a subnet")]
pub(crate) struct SendValueArgs {
    #[arg(long, short, help = "The address to send value from")]
    pub from: Option<String>,
    #[arg(long, short, help = "The address to send value to")]
    pub to: String,
    #[arg(long, short, help = "The subnet of the addresses")]
    pub subnet: String,
    #[arg(help = "The amount to send (in whole FIL units)")]
    pub amount: f64,
}
