// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! List top down cross messages

use anyhow::anyhow;
use std::fmt::Debug;
use std::str::FromStr;

use async_trait::async_trait;
use clap::Args;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;

use crate::commands::get_ipc_provider;
use crate::{CommandLineHandler, GlobalArguments};

/// The command to list top down cross messages in a subnet
pub(crate) struct ListTopdownCrossMessages;

#[async_trait]
impl CommandLineHandler for ListTopdownCrossMessages {
    type Arguments = ListTopdownCrossMessagesArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("list topdown messages with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;

        let hash = if let Some(hash) = &arguments.block_hash {
            hex::decode(hash)?
        } else {
            let parent = subnet
                .parent()
                .ok_or_else(|| anyhow!("subnet has not parent"))?;
            let epoch = provider.get_chain_head_height(&parent).await?;
            let hash = provider.get_block_hash(&parent, epoch).await?;
            hash.block_hash
        };
        let msgs = provider
            .get_top_down_msgs(&subnet, arguments.epoch, &hash)
            .await?;
        for msg in msgs {
            println!("{msg:?}");
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "List topdown cross messages for a specific epoch")]
pub(crate) struct ListTopdownCrossMessagesArgs {
    #[arg(long, short, help = "The subnet id of the topdown subnet")]
    pub subnet: String,
    #[arg(long, short, help = "Include topdown messages of this epoch")]
    pub epoch: ChainEpoch,
    #[arg(long, short, help = "The block hash to query until")]
    pub block_hash: Option<String>,
}
