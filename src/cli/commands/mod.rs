// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! This mod contains the different command line implementations.

mod create;
mod daemon;

use crate::cli::commands::create::{CreateSubnet, CreateSubnetArgs};
use crate::cli::commands::daemon::{LaunchDaemon, LaunchDaemonArgs};
use crate::cli::{CommandLineHandler, GlobalArguments};
use clap::{Parser, Subcommand};
use std::fmt::Debug;

/// The collection of all subcommands to be called, see clap's documentation for usage. Internal
/// to the current mode. Register a new command accordingly.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Launch the ipc agent daemon.
    ///
    /// Note that, technically speaking, this just launches the ipc agent node and runs in the foreground
    /// and not in the background as what daemon processes are. Still, this struct contains `Daemon`
    /// due to the convention from `lotus` and the expected behavior from the filecoin user group.
    Daemon(LaunchDaemonArgs),
    CreateSubnet(CreateSubnetArgs),
}

/// The overall command line struct to be used by `clap`.
#[derive(Debug, Parser)]
#[command(
    name = "ipc",
    about = "The IPC agent command line tool",
    version = "v0.0.1"
)]
#[command(propagate_version = true)]
struct IPCAgentCliCommands {
    #[clap(flatten)]
    global_params: GlobalArguments,
    #[command(subcommand)]
    command: Commands,
}

/// The `cli` method exposed to handle all the cli commands, ideally from main.
///
/// # Examples
/// Sample usage:
/// ```ignore
/// # to start the daemon with
/// ipc-client daemon ./config/template.toml
/// ```
///
/// To register a new command, add the command to
/// ```ignore
/// pub async fn cli() {
///
///     // ... other code
///
///     let r = match &args.command {
///         // ... other existing commands
///         Commands::NewCommand => NewCommand::handle(n).await,
///     };
///
///     // ... other code
/// ```
/// Also add this type to Command enum.
/// ```ignore
/// enum Commands {
///     NewCommand(NewCommandArgs),
/// }
/// ```
pub async fn cli() {
    // parse the arguments
    let args = IPCAgentCliCommands::parse();

    let global = &args.global_params;
    let r = match &args.command {
        Commands::Daemon(args) => LaunchDaemon::handle(global, args).await,
        Commands::CreateSubnet(args) => CreateSubnet::handle(global, args).await,
    };

    if let Err(e) = r {
        log::error!(
            "process command: {:?} failed due to error: {:?}",
            args.command,
            e
        )
    }
}
