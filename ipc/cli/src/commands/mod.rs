// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! This mod contains the different command line implementations.

// mod checkpoint;
// mod config;
// mod crossmsg;
// mod daemon;
mod subnet;
mod util;
mod wallet;

// use crate::commands::checkpoint::CheckpointCommandsArgs;
// use crate::commands::crossmsg::CrossMsgsCommandsArgs;
// use crate::commands::daemon::{LaunchDaemon, LaunchDaemonArgs};
use crate::commands::util::UtilCommandsArgs;
// use crate::server::{new_evm_keystore_from_path, new_keystore_from_path};
use crate::GlobalArguments;
use anyhow::{Context, Result};

use clap::{Command, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use fvm_shared::econ::TokenAmount;

use std::fmt::Debug;
use std::io;

use subnet::SubnetCommandsArgs;
// use crate::commands::config::ConfigCommandsArgs;
use crate::commands::wallet::WalletCommandsArgs;

// pub use subnet::*;

/// We only support up to 9 decimal digits for transaction
const FIL_AMOUNT_NANO_DIGITS: u32 = 9;

/// The collection of all subcommands to be called, see clap's documentation for usage. Internal
/// to the current mode. Register a new command accordingly.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Launch the ipc agent daemon.
    ///
    /// Note that, technically speaking, this just launches the ipc agent node and runs in the foreground
    /// and not in the background as what daemon processes are. Still, this struct contains `Daemon`
    /// due to the convention from `lotus` and the expected behavior from the filecoin user group.
    // Daemon(LaunchDaemonArgs),
    // Config(ConfigCommandsArgs),
    Subnet(SubnetCommandsArgs),
    Wallet(WalletCommandsArgs),
    // CrossMsg(CrossMsgsCommandsArgs),
    // Checkpoint(CheckpointCommandsArgs),
    Util(UtilCommandsArgs),
}

#[derive(Debug, Parser)]
#[command(
    name = "ipc-agent",
    about = "The IPC agent command line tool",
    version = "v0.0.1"
)]
#[command(propagate_version = true)]
struct IPCAgentCliCommands {
    // If provided, outputs the completion file for given shell
    #[arg(long = "cli-autocomplete-gen", value_enum)]
    generator: Option<Shell>,
    #[clap(flatten)]
    global_params: GlobalArguments,
    #[command(subcommand)]
    command: Option<Commands>,
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

    if let Some(generator) = args.generator {
        let mut cmd = IPCAgentCliCommands::command();
        print_completions(generator, &mut cmd);
        Ok(())
    } else {
        let global = &args.global_params;
        if let Some(c) = &args.command {
            let r = match &c {
                // Commands::Daemon(args) => LaunchDaemon::handle(global, args).await,
                // Commands::Config(args) => args.handle(global).await,
                Commands::Subnet(args) => args.handle(global).await,
                // Commands::CrossMsg(args) => args.handle(global).await,
                Commands::Wallet(args) => args.handle(global).await,
                // Commands::Checkpoint(args) => args.handle(global).await,
                Commands::Util(args) => args.handle(global).await,
            };

            r.with_context(|| format!("error processing command {:?}", args.command))
        } else {
            Ok(())
        }
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

pub(crate) fn get_ipc_provider(global: &GlobalArguments) -> Result<ipc_provider::IpcProvider> {
    ipc_provider::IpcProvider::new_from_config(global.config_path())
}

pub(crate) fn f64_to_token_amount(f: f64) -> anyhow::Result<TokenAmount> {
    // no rounding, just the integer part
    let nano = f64::trunc(f * (10u64.pow(FIL_AMOUNT_NANO_DIGITS) as f64));
    Ok(TokenAmount::from_nano(nano as u128))
}

// pub(crate) fn get_evm_keystore(path: &Option<String>) -> Result<PersistentKeyStore<EthKeyAddress>> {
//     match path {
//         Some(p) => new_evm_keystore_from_path(p),
//         None => new_evm_keystore_from_path(&default_repo_path()),
//     }
// }

#[cfg(test)]
mod tests {
    use crate::f64_to_token_amount;
    use fvm_shared::econ::TokenAmount;

    #[test]
    fn test_amount() {
        let amount = f64_to_token_amount(1000000.1f64).unwrap();
        assert_eq!(amount, TokenAmount::from_nano(1000000100000000u128));
    }
}
