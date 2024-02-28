// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use clap::{Args, Subcommand};
use fendermint_materializer::TestnetId;

#[derive(Args, Debug)]
pub struct MaterializerArgs {
    /// Path to the directory where the materializer can store its artifacts.
    ///
    /// This must be the same between materializer invocations.
    #[arg(
        long,
        short,
        env = "FM_MATERIALIZER__DATA_DIR",
        default_value = "~/.ipc/materializer"
    )]
    pub data_dir: PathBuf,

    #[command(subcommand)]
    pub command: MaterializerCommands,
}

#[derive(Subcommand, Debug)]
pub enum MaterializerCommands {
    /// Setup a testnet.
    Setup(MaterializerSetupArgs),
    /// Tear down a testnet.
    Remove(MaterializerRemoveArgs),
}

#[derive(Args, Debug)]
pub struct MaterializerSetupArgs {
    /// Path to the manifest file.
    ///
    /// The format of the manifest (e.g. JSON or YAML) will be determined based on the file extension.
    #[arg(long, short)]
    pub manifest_file: PathBuf,
}

#[derive(Args, Debug)]
pub struct MaterializerRemoveArgs {
    /// ID of the testnet to remove.
    #[arg(long, short)]
    pub testnet_id: TestnetId,
}
