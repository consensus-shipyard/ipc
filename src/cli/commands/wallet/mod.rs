// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::cli::{CommandLineHandler, GlobalArguments};

use crate::cli::commands::wallet::new::{WalletNew, WalletNewArgs};
use clap::{Args, Subcommand};

mod new;

#[derive(Debug, Args)]
#[command(name = "wallet", about = "wallet related commands")]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct WalletCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl WalletCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::New(args) => WalletNew::handle(global, args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    New(WalletNewArgs),
}
