// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

pub mod client;
pub mod node;
pub mod shared;

pub use shared::{bucket, client_context, config, gateway, path};

use crate::commands::storage::client::StorageClientCommandArgs;
use crate::commands::storage::node::StorageNodeCommandArgs;
use crate::GlobalArguments;
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(
    name = "storage",
    about = "storage provider (node) and user (client) commands"
)]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct StorageCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl StorageCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::Node(args) => args.handle(global).await,
            Commands::Client(args) => args.handle(global).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Storage provider (node) setup and runtime
    Node(StorageNodeCommandArgs),
    /// Storage user/client operations and configuration
    Client(StorageClientCommandArgs),
}
