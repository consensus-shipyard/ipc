// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Args)]
#[command(name = "proof-cache", about = "Inspect and debug F3 proof cache")]
pub struct ProofCacheArgs {
    #[command(subcommand)]
    pub command: ProofCacheCommands,
}

#[derive(Debug, Subcommand)]
pub enum ProofCacheCommands {
    /// Inspect cache contents
    Inspect {
        /// Database path
        #[arg(long, env = "FM_PROOF_CACHE_DB")]
        db_path: PathBuf,
    },
    /// Show cache statistics
    Stats {
        /// Database path
        #[arg(long, env = "FM_PROOF_CACHE_DB")]
        db_path: PathBuf,
    },
    /// Get specific proof by instance ID
    Get {
        /// Database path
        #[arg(long, env = "FM_PROOF_CACHE_DB")]
        db_path: PathBuf,

        /// Instance ID to fetch
        #[arg(long)]
        instance_id: u64,
    },
}
