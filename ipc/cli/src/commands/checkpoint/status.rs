// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! List the bottom up checkpoint status

use std::fmt::Debug;
use std::str::FromStr;

use async_trait::async_trait;
use clap::Args;
use fvm_shared::clock::ChainEpoch;
use ipc_api::subnet_id::SubnetID;

use crate::commands::get_ipc_provider;
use crate::{CommandLineHandler, GlobalArguments};

const DEFAULT_MAX_PENDING: usize = 10;

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

        println!(
            "last checkpoint height: {}, chain head {}",
            height, chain_head
        );
        println!("last submitted checkpoint: {:?}", checkpoint);

        let max_pending = arguments.max_pending.unwrap_or(DEFAULT_MAX_PENDING);
        for h in height + 1..max_pending as ChainEpoch {
            let c = provider.get_bottom_up_bundle(&subnet, h).await?;
            if c.checkpoint.block_height != 0 {
                println!("pending checkpoint bundle at height: {} \n {:?}", h, c);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "List bottom up checkpoint status of the subnet")]
pub(crate) struct StatusArgs {
    #[arg(long, help = "The target subnet to check status")]
    pub subnet: String,
    #[arg(
        long,
        help = "The max number of pending checkpoint to print, default 10"
    )]
    pub max_pending: Option<usize>,
}
