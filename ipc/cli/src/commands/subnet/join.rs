// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Join subnet cli command handler.

use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use std::{fmt::Debug, str::FromStr};

use crate::{f64_to_token_amount, get_ipc_provider, CommandLineHandler, GlobalArguments};

/// The command to join a subnet
pub struct JoinSubnet;

#[async_trait]
impl CommandLineHandler for JoinSubnet {
    type Arguments = JoinSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("join subnet with args: {:?}", arguments);

        let mut provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let from = match &arguments.from {
            Some(address) => Some(Address::from_str(address)?),
            None => None,
        };
        let worker_addr = match &arguments.worker_addr {
            Some(address) => Some(Address::from_str(address)?),
            None => None,
        };

        provider
            .join_subnet(
                subnet,
                from,
                f64_to_token_amount(arguments.collateral)?,
                arguments.validator_net_addr.clone(),
                worker_addr,
            )
            .await
    }
}

#[derive(Debug, Args)]
#[command(name = "join", about = "Join a subnet")]
pub struct JoinSubnetArgs {
    #[arg(long, short, help = "The address that joins the subnet")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to join")]
    pub subnet: String,
    #[arg(
        long,
        short,
        help = "The collateral to stake in the subnet (in whole FIL units)"
    )]
    pub collateral: f64,
    #[arg(long, short, help = "The validator net address")]
    pub validator_net_addr: String,
    #[arg(
        long,
        short,
        help = "The validator worker address. If not set will be the same as `from`"
    )]
    pub worker_addr: Option<String>,
}
