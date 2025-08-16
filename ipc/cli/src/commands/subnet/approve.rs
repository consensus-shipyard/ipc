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
        approve_subnet(&mut provider, arguments).await
    }
}

pub(crate) async fn approve_subnet(
    provider: &mut ipc_provider::IpcProvider,
    args: &ApproveSubnetArgs,
) -> anyhow::Result<()> {
    log::info!("approve_subnet command handler called");
    log::info!("  Args: {:?}", args);

    let subnet = SubnetID::from_str(&args.subnet)?;
    log::info!("  Parsed subnet: {:?}", subnet);

    let from = match &args.from {
        Some(address) => {
            let addr = require_fil_addr_from_str(address)?;
            log::info!("  From address (parsed): {:?}", addr);
            Some(addr)
        }
        None => {
            log::info!("  No from address provided");
            None
        }
    };

    log::info!("  Calling provider.approve_subnet...");
    match provider.approve_subnet(subnet.clone(), from).await {
        Ok(()) => {
            log::info!("  ✓ provider.approve_subnet succeeded for subnet: {}", subnet);
            Ok(())
        }
        Err(e) => {
            log::error!("  ✗ provider.approve_subnet failed for subnet: {}", subnet);
            log::error!("  Error: {}", e);
            log::error!("  Error chain: {:?}", e);
            Err(e)
        }
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
