// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! This mod contains the different command line implementations.

mod config;
mod daemon;
mod manager;

use crate::cli::commands::config::{InitConfig, InitConfigArgs, ReloadConfig, ReloadConfigArgs};
use crate::cli::commands::daemon::{LaunchDaemon, LaunchDaemonArgs};
use crate::cli::commands::manager::create::{CreateSubnet, CreateSubnetArgs};
use crate::cli::commands::manager::fund::{Fund, FundArgs};
use crate::cli::commands::manager::join::{JoinSubnet, JoinSubnetArgs};
use crate::cli::commands::manager::kill::{KillSubnet, KillSubnetArgs};
use crate::cli::commands::manager::leave::{LeaveSubnet, LeaveSubnetArgs};
use crate::cli::commands::manager::list_subnets::{ListSubnets, ListSubnetsArgs};
use crate::cli::commands::manager::propagate::{Propagate, PropagateArgs};
use crate::cli::commands::manager::release::{Release, ReleaseArgs};
use crate::cli::commands::manager::whitelist::{WhitelistPropagator, WhitelistPropagatorArgs};
use crate::cli::{CommandLineHandler, GlobalArguments};
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fmt::Debug;
use url::Url;

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

    /// Config commands
    ReloadConfig(ReloadConfigArgs),
    InitConfig(InitConfigArgs),

    /// Subnet manager commands
    CreateSubnet(CreateSubnetArgs),
    ListSubnets(ListSubnetsArgs),
    JoinSubnet(JoinSubnetArgs),
    LeaveSubnet(LeaveSubnetArgs),
    KillSubnet(KillSubnetArgs),
    Fund(FundArgs),
    Release(ReleaseArgs),
    Propagate(PropagateArgs),
    WhitelistPropagator(WhitelistPropagatorArgs),
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
        // Config commands
        Commands::ReloadConfig(args) => ReloadConfig::handle(global, args).await,
        Commands::InitConfig(args) => InitConfig::handle(global, args).await,
        // Subnet manager commands
        Commands::CreateSubnet(args) => CreateSubnet::handle(global, args).await,
        Commands::ListSubnets(args) => ListSubnets::handle(global, args).await,
        Commands::JoinSubnet(args) => JoinSubnet::handle(global, args).await,
        Commands::LeaveSubnet(args) => LeaveSubnet::handle(global, args).await,
        Commands::KillSubnet(args) => KillSubnet::handle(global, args).await,
        Commands::Fund(args) => Fund::handle(global, args).await,
        Commands::Release(args) => Release::handle(global, args).await,
        Commands::Propagate(args) => Propagate::handle(global, args).await,
        Commands::WhitelistPropagator(args) => WhitelistPropagator::handle(global, args).await,
    };

    if let Err(e) = r {
        log::error!(
            "process command: {:?} failed due to error: {:?}",
            args.command,
            e
        )
    }
}

pub(crate) fn get_ipc_agent_url(
    ipc_agent_url: &Option<String>,
    global: &GlobalArguments,
) -> Result<Url> {
    let url = match ipc_agent_url {
        Some(url) => url.parse()?,
        None => {
            let config = global.config()?;
            let addr = config.server.json_rpc_address.to_string();
            // We are resolving back to our own ipc-agent node.
            // Since it's our own node, we will use http since we
            // should be in the same network.
            format!("http://{addr:}/json_rpc").parse()?
        }
    };
    Ok(url)
}
