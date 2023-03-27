// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use tracing::Level;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Options {
    /// Set a custom directory for configuration files.
    ///
    /// By default the application will try to find where the config directory is.
    #[arg(short, long, value_name = "FILE")]
    config_dir: Option<PathBuf>,

    /// Optionally override the default configuration.
    #[arg(short, long, default_value = "dev")]
    pub mode: String,

    /// Turn debugging information on.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Commands,
}

impl Options {
    /// Return the configured config directory, or a default, if they exist.
    pub fn config_dir(&self) -> Option<PathBuf> {
        if let Some(config_dir) = &self.config_dir {
            return Some(config_dir.clone());
        }
        for d in &["./config", "~/.fendermint/config"] {
            let p = PathBuf::from(d);
            if p.is_dir() {
                return Some(p);
            }
        }
        None
    }

    pub fn tracing_level(&self) -> Level {
        match self.debug {
            0 => Level::ERROR,
            1 => Level::WARN,
            2 => Level::INFO,
            3 => Level::DEBUG,
            _ => Level::TRACE,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run the [`App`], listening to ABCI requests from Tendermint.
    Run(RunArgs),
    /// Generate a new Secp256k1 key pair and export them to files in base64 format.
    Keygen(KeygenArgs),
}

#[derive(Args, Debug)]
pub struct RunArgs;

#[derive(Args, Debug)]
pub struct KeygenArgs {
    /// Name used to distinguish the files from other exported keys.
    #[arg(short, long)]
    pub name: String,
    /// Directory to export the key files to; it must exist.
    #[arg(short, long, default_value = ".")]
    pub out_dir: PathBuf,
}
