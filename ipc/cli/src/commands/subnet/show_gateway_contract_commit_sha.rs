// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! List subnets cli command

use async_trait::async_trait;
use clap::Args;
use ipc_api::subnet_id::SubnetID;
use std::fmt::Debug;
use std::str::FromStr;

use crate::{get_ipc_provider, require_fil_addr_from_str, CommandLineHandler, GlobalArguments};

/// The command to create a new subnet actor.
pub(crate) struct ShowGatewayContractCommitSha;

#[async_trait]
impl CommandLineHandler for ShowGatewayContractCommitSha {
    type Arguments = ShowGatewayContractCommitShaArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("show gateway contract commit sha with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.network)?;

        // let gateway_addr = match &arguments.gateway_address {
        //     Some(address) => Some(require_fil_addr_from_str(address)?),
        //     None => None,
        // };

        let commit_sha = provider.get_commit_sha(&subnet).await?;
        println!("Commit sha {:?}", commit_sha);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(
    name = "list",
    about = "List all child subnets registered in the gateway (i.e. that have provided enough collateral)"
)]
pub(crate) struct ShowGatewayContractCommitShaArgs {
    // #[arg(long, help = "The gateway address to query subnets")]
    // pub gateway_address: Option<String>,
    #[arg(long, help = "The network id to query child subnets")]
    pub network: String,
}
