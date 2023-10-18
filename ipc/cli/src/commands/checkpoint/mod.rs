// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::commands::checkpoint::list_checkpoints::{
    ListBottomUpCheckpoints, ListBottomUpCheckpointsArgs,
};
use crate::commands::checkpoint::relayer::{BottomUpRelayer, BottomUpRelayerArgs};
use crate::{CommandLineHandler, GlobalArguments};
use clap::{Args, Subcommand};

mod list_checkpoints;
mod relayer;

#[derive(Debug, Args)]
#[command(name = "checkpoint", about = "checkpoint related commands")]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct CheckpointCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl CheckpointCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::ListBottomup(args) => ListBottomUpCheckpoints::handle(global, args).await,
            Commands::Relayer(args) => BottomUpRelayer::handle(global, args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    ListBottomup(ListBottomUpCheckpointsArgs),
    Relayer(BottomUpRelayerArgs),
}
