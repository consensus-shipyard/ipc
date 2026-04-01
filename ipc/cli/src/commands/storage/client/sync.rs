// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Sync command for synchronizing directories with storage

use anyhow::{anyhow, Result};
use clap::Args;
use std::path::PathBuf;

use async_trait::async_trait;

use crate::commands::storage::client::cp::{CopyArgs, CopyStorage};
use crate::commands::storage::path;
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
pub struct SyncArgs {
    /// Source path (local directory or ipc://bucket/prefix)
    #[arg(value_name = "SOURCE")]
    pub source: String,

    /// Destination path (ipc://bucket/prefix or local directory)
    #[arg(value_name = "DEST")]
    pub dest: String,

    /// Gateway URL (overrides config and env var)
    #[arg(long)]
    pub gateway: Option<String>,

    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Dry run (show what would be synced)
    #[arg(long)]
    pub dry_run: bool,

    /// Delete files in destination that don't exist in source
    #[arg(long)]
    pub delete: bool,
}

pub struct SyncStorage;

#[async_trait]
impl CommandLineHandler for SyncStorage {
    type Arguments = SyncArgs;

    async fn handle(global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        if args.dry_run || args.delete {
            return Err(anyhow!(
                "sync --dry-run/--delete is not implemented yet; use 'storage client cp -r' for now"
            ));
        }
        let source_is_storage = path::is_storage_path(&args.source);
        let dest_is_storage = path::is_storage_path(&args.dest);

        match (source_is_storage, dest_is_storage) {
            (false, true) => {
                // Local -> Storage: currently mapped to recursive copy.
                let cp_args = CopyArgs {
                    source: args.source.clone(),
                    dest: args.dest.clone(),
                    gateway: args.gateway.clone(),
                    config: args.config.clone(),
                    recursive: true,
                    overwrite: true,
                };
                CopyStorage::handle(global, &cp_args).await
            }
            (true, false) => {
                // Storage -> Local: currently mapped to recursive copy.
                let cp_args = CopyArgs {
                    source: args.source.clone(),
                    dest: args.dest.clone(),
                    gateway: args.gateway.clone(),
                    config: args.config.clone(),
                    recursive: true,
                    overwrite: true,
                };
                CopyStorage::handle(global, &cp_args).await
            }
            (true, true) => {
                // Storage -> Storage sync
                Err(anyhow!(
                    "Syncing between storage locations not yet implemented"
                ))
            }
            (false, false) => Err(anyhow!(
                "At least one path must be a storage path (ipc://...)"
            )),
        }
    }
}
