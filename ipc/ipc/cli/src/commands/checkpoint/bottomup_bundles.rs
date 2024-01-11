// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! List bottom up bundles

use std::fmt::Debug;
use std::str::FromStr;

use async_trait::async_trait;
use clap::Args;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::checkpoint::QuorumObjKind;
use ipc_sdk::subnet_id::SubnetID;

use crate::commands::get_ipc_provider;
use crate::{CommandLineHandler, GlobalArguments};

/// The command to get bottom up bundles at height.
pub(crate) struct GetBottomUpBundles;

#[async_trait]
impl CommandLineHandler for GetBottomUpBundles {
    type Arguments = GetBottomUpBundlesArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("get bottom up bundles with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;
        let kind = QuorumObjKind::try_from(arguments.kind)?;

        for h in arguments.from_epoch..=arguments.to_epoch {
            match kind {
                QuorumObjKind::Checkpoint => {
                    let bundle = provider.get_bottom_up_checkpoint_bundle(&subnet, h).await?;
                    println!(
                        "checkpoint: {:?}, signatures: {:?}, signatories: {:?}",
                        bundle.checkpoint, bundle.signatures, bundle.signatories,
                    );
                }
                QuorumObjKind::BottomUpMsgBatch => {
                    let bundle = provider.get_bottom_up_msg_batch_bundle(&subnet, h).await?;
                    println!(
                        "batch: {:?}, signatures: {:?}, signatories: {:?}",
                        bundle.checkpoint, bundle.signatures, bundle.signatories,
                    );
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "List bottom up checkpoint signature bundle for a child subnet")]
pub(crate) struct GetBottomUpBundlesArgs {
    #[arg(long, short, help = "The target subnet to perform query")]
    pub subnet: String,
    #[arg(
        long,
        short,
        help = "List the bottom up checkpoint(0) or the msg batch(1)"
    )]
    pub kind: u8,
    #[arg(long, short, help = "Include data from this epoch")]
    pub from_epoch: ChainEpoch,
    #[arg(long, short, help = "Include data up to this epoch")]
    pub to_epoch: ChainEpoch,
}
