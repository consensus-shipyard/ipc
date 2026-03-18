// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

pub mod config;
pub mod init;
pub mod run;

use crate::commands::storage::init::{InitStorage, InitStorageArgs};
use crate::commands::storage::run::{RunStorage, RunStorageArgs};
use crate::{CommandLineHandler, GlobalArguments};
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(name = "storage", about = "storage node automation commands")]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct StorageCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl StorageCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::Init(args) => InitStorage::handle(global, args).await,
            Commands::Run(args) => RunStorage::handle(global, args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Init(InitStorageArgs),
    Run(RunStorageArgs),
}
