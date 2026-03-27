// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Sync command for synchronizing directories with storage

use anyhow::{anyhow, Result};
use clap::Args;
use std::path::PathBuf;

use async_trait::async_trait;

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

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let source_is_storage = path::is_storage_path(&args.source);
        let dest_is_storage = path::is_storage_path(&args.dest);

        match (source_is_storage, dest_is_storage) {
            (false, true) => {
                // Local -> Storage sync
                sync_local_to_storage(args).await
            }
            (true, false) => {
                // Storage -> Local sync
                sync_storage_to_local(args).await
            }
            (true, true) => {
                // Storage -> Storage sync
                Err(anyhow!("Syncing between storage locations not yet implemented"))
            }
            (false, false) => {
                Err(anyhow!(
                    "At least one path must be a storage path (ipc://...)"
                ))
            }
        }
    }
}

async fn sync_local_to_storage(_args: &SyncArgs) -> Result<()> {
    // TODO: Implement by:
    // 1. List all local files in source directory
    // 2. List all storage objects with destination prefix
    // 3. Compare and determine:
    //    - New files to upload
    //    - Modified files to re-upload (compare size/hash)
    //    - Files to delete (if --delete flag)
    // 4. Perform operations (unless --dry-run)

    Err(anyhow!(
        "Sync functionality not yet implemented.\n\
         Use 'ipc-cli storage cp -r' for recursive upload/download."
    ))
}

async fn sync_storage_to_local(_args: &SyncArgs) -> Result<()> {
    // TODO: Similar to sync_local_to_storage but in reverse

    Err(anyhow!(
        "Sync functionality not yet implemented.\n\
         Use 'ipc-cli storage cp -r' for recursive upload/download."
    ))
}
