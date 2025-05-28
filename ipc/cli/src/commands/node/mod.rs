// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

mod init;
mod start;

use clap::{Args, Subcommand};
use std::fmt::Debug;

use crate::commands::node::init::{InitNode, InitNodeArgs};
use crate::commands::node::start::{StartNode, StartNodeArgs};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
#[command(name = "node", about = "node related commands")]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct NodeCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl NodeCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::Start(args) => StartNode::handle(global, args).await,
            Commands::Init(args) => InitNode::handle(global, args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Init(InitNodeArgs),
    Start(StartNodeArgs),
}
