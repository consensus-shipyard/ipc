// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use fvm_shared::address::Network;
use mat::MaterializerArgs;

use self::{eth::EthArgs, genesis::GenesisArgs, key::KeyArgs, rpc::RpcArgs, run::RunArgs};

pub mod eth;
pub mod genesis;
pub mod key;
pub mod mat;
pub mod rpc;
pub mod run;

mod parse;

use parse::parse_network;

/// Parse the main arguments by:
/// 1. Parsing the [GlobalOptions]
/// 2. Setting any system wide parameters based on the globals
/// 3. Parsing and returning the final [Options]
pub fn parse() -> Options {
    let opts: GlobalOptions = GlobalOptions::parse();
    fvm_shared::address::set_current_network(opts.global.network);
    let opts: Options = Options::parse();
    opts
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Args, Debug)]
pub struct GlobalArgs {
    /// Set the FVM Address Network. It's value affects whether `f` (main) or `t` (test) prefixed addresses are accepted.
    #[arg(short, long, default_value = "mainnet", env = "FM_NETWORK", value_parser = parse_network)]
    pub network: Network,
}

/// A version of options that does partial matching on the arguments, with its only interest
/// being the capture of global parameters that need to take effect first, before we parse [Options],
/// because their value affects how others arse parsed.
///
/// This one doesn't handle `--help` or `help` so that it is passed on to the next parser,
/// where the full set of commands and arguments can be printed properly.
#[derive(Parser, Debug)]
#[command(version, disable_help_flag = true)]
pub struct GlobalOptions {
    #[command(flatten)]
    pub global: GlobalArgs,

    /// Capture all the normal commands, basically to ingore them.
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub cmd: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(version)]
pub struct Options {
    /// Set a custom directory for data and configuration files.
    #[arg(
        short = 'd',
        long,
        default_value = "~/.fendermint",
        env = "FM_HOME_DIR"
    )]
    pub home_dir: PathBuf,

    /// Set a custom directory for ipc log files.
    #[arg(long, env = "FM_LOG_DIR")]
    pub log_dir: Option<PathBuf>,

    /// Set a custom prefix for ipc log files.
    #[arg(long, env = "FM_LOG_FILE_PREFIX")]
    pub log_file_prefix: Option<String>,

    /// Optionally override the default configuration.
    #[arg(short, long, default_value = "dev")]
    pub mode: String,

    /// Set the logging level.
    #[arg(
        short = 'l',
        long,
        default_value = "info",
        value_enum,
        env = "LOG_LEVEL"
    )]
    pub log_level: LogLevel,

    /// Global options repeated here for discoverability, so they show up in `--help` among the others.
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Commands,
}

impl Options {
    /// Tracing level, unless it's turned off.
    pub fn tracing_level(&self) -> Option<tracing::Level> {
        match self.log_level {
            LogLevel::Off => None,
            LogLevel::Error => Some(tracing::Level::ERROR),
            LogLevel::Warn => Some(tracing::Level::WARN),
            LogLevel::Info => Some(tracing::Level::INFO),
            LogLevel::Debug => Some(tracing::Level::DEBUG),
            LogLevel::Trace => Some(tracing::Level::TRACE),
        }
    }

    pub fn config_dir(&self) -> PathBuf {
        self.home_dir.join("config")
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run the `App`, listening to ABCI requests from Tendermint.
    Run(RunArgs),
    /// Subcommands related to the construction of signing keys.
    Key(KeyArgs),
    /// Subcommands related to the construction of Genesis files.
    Genesis(GenesisArgs),
    /// Subcommands related to sending JSON-RPC commands/queries to Tendermint.
    Rpc(RpcArgs),
    /// Subcommands related to the Ethereum API facade.
    Eth(EthArgs),
    /// Subcommands related to the Testnet Materializer.
    Mat(MaterializerArgs),
}

#[cfg(test)]
mod tests {
    use crate::*;
    use clap::Parser;
    use fvm_shared::address::Network;

    #[test]
    fn parse_global() {
        let cmd = "fendermint --network testnet genesis --genesis-file ./genesis.json ipc gateway --subnet-id /r123/t0456 -b 10 -t 10 -f 10 -m 65";
        let opts: GlobalOptions = GlobalOptions::parse_from(cmd.split_ascii_whitespace());
        assert_eq!(opts.global.network, Network::Testnet);
    }

    #[test]
    fn global_options_ignore_help() {
        let cmd = "fendermint --help";
        let _opts: GlobalOptions = GlobalOptions::parse_from(cmd.split_ascii_whitespace());
    }

    #[test]
    fn options_handle_help() {
        let cmd = "fendermint --help";
        // This test would fail with a panic if we have a misconfiguration in our options.
        // On successfully parsing `--help` with `parse_from` the library would `.exit()` the test framework itself,
        // which is why we must use `try_parse_from`. An error results in a panic from `parse_from` and an `Err`
        // from this, but `--help` is not an `Ok`, since we aren't getting `Options`; it's an `Err` with a help message.
        let e = Options::try_parse_from(cmd.split_ascii_whitespace())
            .expect_err("--help is not Options");

        assert!(e.to_string().contains("Usage:"), "unexpected help: {e}");
    }
}
