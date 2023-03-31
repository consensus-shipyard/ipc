// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Subcommand, Debug)]
pub enum KeyCommands {
    /// Generate a new Secp256k1 key pair and export them to files in base64 format.
    Gen(KeyGenArgs),
    /// Convert a secret key file from base64 into the format expected by Tendermint.
    IntoTendermint(KeyIntoTendermintArgs),
}

#[derive(Args, Debug)]
pub struct KeyArgs {
    #[command(subcommand)]
    pub command: KeyCommands,
}

#[derive(Args, Debug)]
pub struct KeyGenArgs {
    /// Name used to distinguish the files from other exported keys.
    #[arg(long, short)]
    pub name: String,
    /// Directory to export the key files to; it must exist.
    #[arg(long, short, default_value = ".")]
    pub out_dir: PathBuf,
}

#[derive(Args, Debug)]
pub struct KeyIntoTendermintArgs {
    /// Path to the secret key we want to convert to Tendermint format.
    #[arg(long, short)]
    pub secret_key: PathBuf,
    /// Output file name for the Tendermint private validator key JSON file.
    #[arg(long, short)]
    pub out: PathBuf,
}
