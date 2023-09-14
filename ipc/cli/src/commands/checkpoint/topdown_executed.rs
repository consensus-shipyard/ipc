// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Last top-down checkpoint executed in subnet

use std::{fmt::Debug, str::FromStr};

use async_trait::async_trait;
use clap::Args;
use ipc_sdk::subnet_id::SubnetID;

use crate::{get_ipc_provider, require_fil_addr_from_str, CommandLineHandler, GlobalArguments};

/// The command to get the latest epoch executed for a top-down checkpoint
pub(crate) struct LastTopDownExec;

#[async_trait]
impl CommandLineHandler for LastTopDownExec {
    type Arguments = LastTopDownExecArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("last topdown exec with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;

        let gateway_addr = match &arguments.gateway_address {
            Some(address) => Some(require_fil_addr_from_str(address)?),
            None => None,
        };

        println!(
            "epoch: {:?}",
            provider
                .last_topdown_executed(&subnet, gateway_addr)
                .await?
        );

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Epoch of the last top-down checkpoint executed")]
pub(crate) struct LastTopDownExecArgs {
    #[arg(long, short, help = "The gateway address to query subnets")]
    pub gateway_address: Option<String>,
    #[arg(long, short, help = "The subnet id of the checkpointing subnet")]
    pub subnet: String,
}
