// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! This mod triggers a config reload in the IPC-Agent Json RPC server.

mod init;
mod reload;

use clap::{Args, Subcommand};
use std::fmt::Debug;

use crate::cli::commands::config::init::{InitConfig, InitConfigArgs};
use crate::cli::commands::config::reload::{ReloadConfig, ReloadConfigArgs};
use crate::cli::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
#[command(name = "config", about = "config related commands")]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct ConfigCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl ConfigCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::Reload(args) => ReloadConfig::handle(global, args).await,
            Commands::Init(args) => InitConfig::handle(global, args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Reload(ReloadConfigArgs),
    Init(InitConfigArgs),
}
