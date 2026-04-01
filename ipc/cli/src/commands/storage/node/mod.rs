// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use clap::{Args, Subcommand};

pub mod init;
pub mod run;

use self::init::{InitStorage, InitStorageArgs};
use self::run::{RunStorage, RunStorageArgs};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
#[command(name = "node", about = "Storage provider node commands")]
pub struct StorageNodeCommandArgs {
    #[command(subcommand)]
    command: StorageNodeCommands,
}

#[derive(Debug, Subcommand)]
pub enum StorageNodeCommands {
    /// Initialize storage provider config
    Init(InitStorageArgs),
    /// Run storage node and/or gateway
    Run(RunStorageArgs),
}

impl StorageNodeCommandArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            StorageNodeCommands::Init(args) => InitStorage::handle(global, args).await,
            StorageNodeCommands::Run(args) => RunStorage::handle(global, args).await,
        }
    }
}
