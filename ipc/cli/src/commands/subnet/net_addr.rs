// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The command to set the validator net address

use std::str::FromStr;

use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;

use crate::{get_ipc_provider, CommandLineHandler, GlobalArguments};

/// Setting the validator net address
pub(crate) struct SetValidatorNetAddr;

#[async_trait]
impl CommandLineHandler for SetValidatorNetAddr {
    type Arguments = SetValidatorNetAddrArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("set the validator net addr args: {:?}", arguments);

        let mut provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let from = match &arguments.from {
            Some(address) => Some(Address::from_str(address)?),
            None => None,
        };

        provider
            .set_validator_net_addr(subnet, from, arguments.validator_net_addr.clone())
            .await
    }
}

#[derive(Debug, Args)]
#[command(about = "Set the validator net address")]
pub(crate) struct SetValidatorNetAddrArgs {
    #[arg(long, short, help = "Owner address of the validator being updated")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to set the validator")]
    pub subnet: String,
    #[arg(long, short, help = "New validator net address")]
    pub validator_net_addr: String,
}
