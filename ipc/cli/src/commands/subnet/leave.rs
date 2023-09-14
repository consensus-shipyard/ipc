// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Leave subnet cli command handler.

use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use std::{fmt::Debug, str::FromStr};

use crate::{get_ipc_provider, CommandLineHandler, GlobalArguments};

/// The command to leave a new subnet.
pub struct LeaveSubnet;

#[async_trait]
impl CommandLineHandler for LeaveSubnet {
    type Arguments = LeaveSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("leave subnet with args: {:?}", arguments);

        let mut provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let from = match &arguments.from {
            Some(address) => Some(Address::from_str(address)?),
            None => None,
        };
        provider.leave_subnet(subnet, from).await
    }
}

#[derive(Debug, Args)]
#[command(name = "leave", about = "Leaving a subnet")]
pub struct LeaveSubnetArgs {
    #[arg(long, short, help = "The address that leaves the subnet")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to leave")]
    pub subnet: String,
}
