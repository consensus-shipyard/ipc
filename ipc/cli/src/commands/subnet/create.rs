// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Create subnet cli command handler.

use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;
use std::fmt::Debug;
use std::str::FromStr;

use crate::commands::get_ipc_provider;
use crate::{f64_to_token_amount, CommandLineHandler, GlobalArguments};

const DEFAULT_ACTIVE_VALIDATORS: u16 = 100;

/// The command to create a new subnet actor.
pub struct CreateSubnet;

impl CreateSubnet {
    pub async fn create(
        global: &GlobalArguments,
        arguments: &CreateSubnetArgs,
    ) -> anyhow::Result<String> {
        let mut provider = get_ipc_provider(global)?;
        let parent = SubnetID::from_str(&arguments.parent)?;
        let from = match &arguments.from {
            Some(address) => Some(Address::from_str(address)?),
            None => None,
        };

        let addr = provider
            .create_subnet(
                from,
                parent,
                arguments.min_validators,
                f64_to_token_amount(arguments.min_validator_stake)?,
                f64_to_token_amount(arguments.min_cross_msg_fee)?,
                arguments.bottomup_check_period,
                arguments
                    .active_validators_limit
                    .unwrap_or(DEFAULT_ACTIVE_VALIDATORS),
            )
            .await?;

        Ok(addr.to_string())
    }
}

#[async_trait]
impl CommandLineHandler for CreateSubnet {
    type Arguments = CreateSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("create subnet with args: {:?}", arguments);

        let address = CreateSubnet::create(global, arguments).await?;

        log::info!(
            "created subnet actor with id: {}/{}",
            arguments.parent,
            address
        );

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(name = "create", about = "Create a new subnet actor")]
pub struct CreateSubnetArgs {
    #[arg(long, short, help = "The address that creates the subnet")]
    pub from: Option<String>,
    #[arg(long, short, help = "The parent subnet to create the new actor in")]
    pub parent: String,
    #[arg(long, short, help = "The minimal validator stake")]
    pub min_validator_stake: f64,
    #[arg(long, short, help = "The minimal cross message fee")]
    pub min_cross_msg_fee: f64,
    #[arg(long, help = "The minimal number of validators")]
    pub min_validators: u64,
    #[arg(long, help = "The bottom up checkpoint period in number of blocks")]
    pub bottomup_check_period: ChainEpoch,
    #[arg(long, help = "The max number of active validators in subnet")]
    pub active_validators_limit: Option<u16>,
}
