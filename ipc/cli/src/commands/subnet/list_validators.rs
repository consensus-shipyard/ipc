// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! List subnet validators cli command

use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use std::{fmt::Debug, str::FromStr};

use crate::{get_ipc_provider, CommandLineHandler, GlobalArguments};

/// The command to create a new subnet actor.
pub(crate) struct ListValidators;

#[async_trait]
impl CommandLineHandler for ListValidators {
    type Arguments = ListValidatorsArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("list validators with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;

        let gateway_addr = match &arguments.gateway_address {
            Some(address) => Some(Address::from_str(address)?),
            None => None,
        };

        let valset = provider
            .get_validator_set(&subnet, gateway_addr, None)
            .await?;

        println!("minimum number of validators: {}", valset.min_validators);
        println!("validator set: {:?}", valset.validator_set);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(name = "list-validators", about = "Show the validators of the subnet")]
pub(crate) struct ListValidatorsArgs {
    #[arg(long, short, help = "The gateway address to query subnets")]
    pub gateway_address: Option<String>,
    #[arg(long, short, help = "The subnet id to query validators")]
    pub subnet: String,
}
