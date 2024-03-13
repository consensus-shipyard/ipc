// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct UpgradeArgs {
    /// Path to the upgrade_info JSON file.
    #[arg(long)]
    pub upgrade_file: PathBuf,

    #[command(subcommand)]
    pub command: UpgradeCommands,
}

#[derive(Subcommand, Debug)]
pub enum UpgradeCommands {
    AddUpgrade(AddUpgrade),
}

#[derive(Args, Debug)]
pub struct AddUpgrade {
    /// the height at which to schedule the upgrade
    #[arg(long)]
    pub height: u64,

    /// the application version the upgrade will update to
    #[arg(long)]
    pub new_app_version: u64,

    /// the required cometbft version for the upgrade
    #[arg(long)]
    pub cometbft_version: String,

    /// whether this is a required upgrade or not. A required upgrade
    /// will cause the node to freeze and not process any more blocks
    /// if it does not have the corresponding Upgrade defined to
    /// migrate to that version
    #[arg(long)]
    pub required: bool,
}
