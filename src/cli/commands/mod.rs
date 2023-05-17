// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! This mod contains the different command line implementations.

mod checkpoint;
mod config;
mod crossmsg;
mod daemon;
mod subnet;
mod util;
mod wallet;

use crate::cli::commands::checkpoint::CheckpointCommandsArgs;
use crate::cli::commands::crossmsg::CrossMsgsCommandsArgs;
use crate::cli::commands::daemon::{LaunchDaemon, LaunchDaemonArgs};
use crate::cli::commands::util::UtilCommandsArgs;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::server::new_keystore_from_path;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ipc_identity::KeyStore;
use std::fmt::Debug;
use subnet::SubnetCommandsArgs;
use url::Url;

use crate::cli::commands::config::ConfigCommandsArgs;
use crate::cli::commands::wallet::WalletCommandsArgs;

pub use subnet::*;

use super::DEFAULT_CONFIG_PATH;

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
    Config(ConfigCommandsArgs),
    Subnet(SubnetCommandsArgs),
    Wallet(WalletCommandsArgs),
    CrossMsg(CrossMsgsCommandsArgs),
    Checkpoint(CheckpointCommandsArgs),
    Util(UtilCommandsArgs),
}
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
pub async fn cli() -> anyhow::Result<()> {
    // parse the arguments
    let args = IPCAgentCliCommands::parse();

    let global = &args.global_params;
    let r = match &args.command {
        Commands::Daemon(args) => LaunchDaemon::handle(global, args).await,
        Commands::Config(args) => args.handle(global).await,
        Commands::Subnet(args) => args.handle(global).await,
        Commands::CrossMsg(args) => args.handle(global).await,
        Commands::Wallet(args) => args.handle(global).await,
        Commands::Checkpoint(args) => args.handle(global).await,
        Commands::Util(args) => args.handle(global).await,
    };

    r.with_context(|| format!("error processing command {:?}", args.command))
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

pub(crate) fn get_keystore(path: &Option<String>) -> Result<KeyStore> {
    let path = match path {
        Some(p) => p,
        None => DEFAULT_CONFIG_PATH,
    };
    new_keystore_from_path(path)
}
