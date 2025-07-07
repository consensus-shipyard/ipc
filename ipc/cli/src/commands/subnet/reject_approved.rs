// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Reject approved subnet cli command handler.

use async_trait::async_trait;
use clap::Args;
use ipc_api::subnet_id::SubnetID;
use std::{fmt::Debug, str::FromStr};

use crate::{get_ipc_provider, require_fil_addr_from_str, CommandLineHandler, GlobalArguments};

/// The command to reject approved subnet in a gateway
pub struct RejectApprovedSubnet;

#[async_trait]
impl CommandLineHandler for RejectApprovedSubnet {
    type Arguments = RejectApprovedSubnetArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("reject approved subnet with args: {:?}", arguments);

        let mut provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let from = match &arguments.from {
            Some(address) => Some(require_fil_addr_from_str(address)?),
            None => None,
        };

        provider.reject_approved_subnet(subnet, from).await
    }
}

#[derive(Debug, Args)]
#[command(
    name = "reject-approved",
    about = "Reject approved subnet in a gateway"
)]
pub struct RejectApprovedSubnetArgs {
    #[arg(long, help = "The address should be the gateway contract owner")]
    pub from: Option<String>,
    #[arg(long, help = "The approved subnet to reject")]
    pub subnet: String,
}
