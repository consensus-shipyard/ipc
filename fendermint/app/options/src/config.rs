// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct PrintConfigArgs;

#[derive(Args, Debug)]
pub struct InitConfigArgs;

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Create a new default config file
    Init(InitConfigArgs),
    /// Print the current config
    Print(PrintConfigArgs),
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}
