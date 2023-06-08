// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! List checkpoints cli command

use std::fmt::Debug;

use async_trait::async_trait;
use clap::Args;
use fvm_shared::clock::ChainEpoch;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::sdk::IpcAgentClient;

/// The command to list checkpoints committed in a subnet actor.
pub(crate) struct ListBottomUpCheckpoints;

#[async_trait]
impl CommandLineHandler for ListBottomUpCheckpoints {
    type Arguments = ListBottomUpCheckpointsArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("list checkpoints with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let client = IpcAgentClient::default_from_url(url);
        let checkpoints = client
            .list_bottom_up_checkpoints(&arguments.subnet, arguments.from_epoch, arguments.to_epoch)
            .await?;

        for c in checkpoints.iter() {
            let c = &c["data"];
            log::info!(
                "epoch {} - prev_check={}, cross_msgs={}, child_checks={}",
                c["epoch"],
                c["prev_check"],
                c["cross_msgs"],
                c["children"]
            );
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "List bottom-up checkpoints")]
pub(crate) struct ListBottomUpCheckpointsArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The subnet id of the checkpointing subnet")]
    pub subnet: String,
    #[arg(long, short, help = "Include checkpoints from this epoch")]
    pub from_epoch: ChainEpoch,
    #[arg(long, short, help = "Include checkpoints up to this epoch")]
    pub to_epoch: ChainEpoch,
}
