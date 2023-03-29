// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use num_traits::Num;

use fvm_shared::{bigint::BigInt, econ::TokenAmount, version::NetworkVersion};

use crate::settings::expand_tilde;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Parser, Debug)]
#[command(version)]
pub struct Options {
    /// Set a custom directory for data and configuration files.
    #[arg(short = 'd', long, default_value = "~/.fendermint")]
    pub home_dir: PathBuf,

    /// Optionally override the default configuration.
    #[arg(short, long, default_value = "dev")]
    pub mode: String,

    /// Set the logging level.
    #[arg(short, long, default_value = "info", value_enum)]
    pub log_level: LogLevel,

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
        expand_tilde(self.home_dir.join("config"))
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run the [`App`], listening to ABCI requests from Tendermint.
    Run(RunArgs),
    /// Generate a new Secp256k1 key pair and export them to files in base64 format.
    Keygen(KeygenArgs),
    /// Subcommands related to the construction of Genesis files.
    Genesis(GenesisArgs),
}

#[derive(Subcommand, Debug)]
pub enum GenesisCommands {
    /// Create a new Genesis file, with accounts and validators to be added later.
    New(GenesisNewArgs),
    /// Add an account to the genesis file.
    AddAccount(GenesisAddAccountArgs),
    /// Add a multi-sig account to the genesis file.
    AddMultisig(GenesisAddMultisigArgs),
    /// Add a validator to the genesis file.
    AddValidator(GenesisAddValidatorArgs),
    /// Convert the genesis file into the format expected by Tendermint.
    IntoTendermint(GenesisIntoTendermintArgs),
}

#[derive(Args, Debug)]
pub struct RunArgs;

#[derive(Args, Debug)]
pub struct KeygenArgs {
    /// Name used to distinguish the files from other exported keys.
    #[arg(long, short)]
    pub name: String,
    /// Directory to export the key files to; it must exist.
    #[arg(long, short, default_value = ".")]
    pub out_dir: PathBuf,
}

#[derive(Args, Debug)]
pub struct GenesisArgs {
    /// Path to the genesis JSON file.
    #[arg(long, short)]
    pub genesis_file: PathBuf,

    #[command(subcommand)]
    pub command: GenesisCommands,
}

#[derive(Args, Debug)]
pub struct GenesisNewArgs {
    /// Genesis timestamp as seconds since Unix epoch.
    #[arg(long, short)]
    pub timestamp: u64,
    /// Name of the network and chain.
    #[arg(long, short = 'n')]
    pub network_name: String,
    /// Network version, governs which set of built-in actors to use.
    #[arg(long, short = 'v', default_value = "18", value_parser = parse_network_version)]
    pub network_version: NetworkVersion,
    /// Base fee for running transactions in atto.
    #[arg(long, short = 'f', value_parser = parse_token_amount)]
    pub base_fee: TokenAmount,
}

#[derive(Args, Debug)]
pub struct GenesisAddAccountArgs {
    /// Path to the Secp256k1 public key exported in base64 format.
    #[arg(long, short)]
    pub public_key: PathBuf,
    /// Initial balance in atto.
    #[arg(long, short, value_parser = parse_token_amount)]
    pub balance: TokenAmount,
}

#[derive(Args, Debug)]
pub struct GenesisAddMultisigArgs {
    /// Path to the Secp256k1 public key exported in base64 format, one for each signatory.
    #[arg(long, short)]
    pub public_key: Vec<PathBuf>,
    /// Initial balance in atto.
    #[arg(long, short, value_parser = parse_token_amount)]
    pub balance: TokenAmount,
    /// Number of signatures required.
    #[arg(long, short)]
    pub threshold: u64,
    /// Linear unlock duration in block heights.
    #[arg(long, short = 'd')]
    pub vesting_duration: u64,
    /// Linear unlock start block height.
    #[arg(long, short = 's')]
    pub vesting_start: u64,
}

#[derive(Args, Debug)]
pub struct GenesisAddValidatorArgs {
    /// Path to the Secp256k1 public key exported in base64 format.
    #[arg(long, short)]
    pub public_key: PathBuf,
    /// Voting power.
    #[arg(long, short = 'v')]
    pub power: u64,
}

#[derive(Args, Debug)]
pub struct GenesisIntoTendermintArgs {
    /// Output file name for the Tendermint genesis JSON file.
    #[arg(long, short)]
    pub out: PathBuf,
    /// Maximum block size in bytes.
    #[arg(long, default_value_t = 22020096)]
    pub block_max_bytes: u64,
}

fn parse_network_version(s: &str) -> Result<NetworkVersion, String> {
    let nv: u32 = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a network version"))?;
    if nv >= 18 {
        Ok(NetworkVersion::from(nv))
    } else {
        Err("the minimum network version is 18".to_owned())
    }
}

fn parse_token_amount(s: &str) -> Result<TokenAmount, String> {
    BigInt::from_str_radix(s, 10)
        .map_err(|e| format!("not a token amount: {e}"))
        .map(TokenAmount::from_atto)
}
