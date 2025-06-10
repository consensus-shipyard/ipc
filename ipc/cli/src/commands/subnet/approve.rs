// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Approve a subnet cli command handler.

use async_trait::async_trait;
use clap::Args;
use ipc_api::subnet_id::SubnetID;
use std::{fmt::Debug, str::FromStr};

use crate::{get_ipc_provider, require_fil_addr_from_str, CommandLineHandler, GlobalArguments};

/// The command to approve subnet in a gateway
pub struct ApproveSubnet;

#[async_trait]
impl CommandLineHandler for ApproveSubnet {
    type Arguments = ApproveSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("approve subnet with args: {:?}", arguments);

        let mut provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let from = match &arguments.from {
            Some(address) => Some(require_fil_addr_from_str(address)?),
            None => None,
        };

        provider.approve_subnet(subnet, from).await
    }
}

#[derive(Debug, Args)]
#[command(name = "approve", about = "Approve subnet in a gateway")]
pub struct ApproveSubnetArgs {
    #[arg(long, help = "The address should be the gateway contract owner")]
    pub from: Option<String>,
    #[arg(long, help = "The subnet to approve")]
    pub subnet: String,
}
