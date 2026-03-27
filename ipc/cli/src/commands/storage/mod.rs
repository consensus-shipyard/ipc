// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

pub mod bucket;
pub mod bucket_cmd;
pub mod cat;
pub mod credit;
pub mod client;
pub mod config;
pub mod cp;
pub mod init;
pub mod ls;
pub mod mv;
pub mod path;
pub mod rm;
pub mod run;
pub mod stat;
pub mod sync;

use crate::commands::storage::bucket_cmd::BucketCommandArgs;
use crate::commands::storage::cat::{CatArgs, CatStorage};
use crate::commands::storage::credit::CreditCommandArgs;
use crate::commands::storage::cp::{CopyArgs, CopyStorage};
use crate::commands::storage::init::{InitStorage, InitStorageArgs};
use crate::commands::storage::ls::{ListArgs, ListStorage};
use crate::commands::storage::mv::{MoveArgs, MoveStorage};
use crate::commands::storage::rm::{RemoveArgs, RemoveStorage};
use crate::commands::storage::run::{RunStorage, RunStorageArgs};
use crate::commands::storage::stat::{StatArgs, StatStorage};
use crate::commands::storage::sync::{SyncArgs, SyncStorage};
use crate::{CommandLineHandler, GlobalArguments};
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(name = "storage", about = "storage node automation and file operations")]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct StorageCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl StorageCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::Bucket(args) => args.handle(global).await,
            Commands::Credit(args) => args.handle(global).await,
            Commands::Init(args) => InitStorage::handle(global, args).await,
            Commands::Run(args) => RunStorage::handle(global, args).await,
            Commands::Cp(args) => CopyStorage::handle(global, args).await,
            Commands::Ls(args) => ListStorage::handle(global, args).await,
            Commands::Cat(args) => CatStorage::handle(global, args).await,
            Commands::Stat(args) => StatStorage::handle(global, args).await,
            Commands::Rm(args) => RemoveStorage::handle(global, args).await,
            Commands::Mv(args) => MoveStorage::handle(global, args).await,
            Commands::Sync(args) => SyncStorage::handle(global, args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Create and manage storage buckets
    Bucket(BucketCommandArgs),
    /// Buy and query storage credits
    Credit(CreditCommandArgs),
    /// Initialize storage configuration
    Init(InitStorageArgs),
    /// Run storage node and/or gateway
    Run(RunStorageArgs),
    /// Copy files to/from storage
    Cp(CopyArgs),
    /// List objects in storage
    Ls(ListArgs),
    /// Display file contents from storage
    Cat(CatArgs),
    /// Show object metadata
    Stat(StatArgs),
    /// Remove objects from storage
    Rm(RemoveArgs),
    /// Move/rename objects in storage
    Mv(MoveArgs),
    /// Sync directories with storage
    Sync(SyncArgs),
}
