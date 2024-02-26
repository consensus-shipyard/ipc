// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! List the bottom up checkpoint status

use std::fmt::Debug;
use std::str::FromStr;

use async_trait::async_trait;
use clap::Args;
use ipc_api::subnet_id::SubnetID;

use crate::commands::get_ipc_provider;
use crate::{CommandLineHandler, GlobalArguments};

/// The command to list bottom up checkpoint status.
pub(crate) struct Status;

#[async_trait]
impl CommandLineHandler for Status {
    type Arguments = StatusArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("bottom up checkpoint status with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;

        let height = provider.last_bottom_up_checkpoint_height(&subnet).await?;
        let checkpoint = provider.get_bottom_up_bundle(&subnet, height).await?;
        let chain_head = provider.get_chain_head_height(&subnet).await?;

        let maybe_height = provider.max_quorum_reached_height(&subnet, height + 1, chain_head).await?;

        println!(
            "last checkpoint height: {}, chain head {}, max quorum reached height {:?}",
            height, chain_head, maybe_height
        );
        println!("last submitted checkpoint: {:?}", checkpoint);

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "List bottom up checkpoint status of the subnet")]
pub(crate) struct StatusArgs {
    #[arg(long, help = "The target subnet to check status")]
    pub subnet: String,
}
