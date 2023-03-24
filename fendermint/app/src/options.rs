// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::Level;

#[derive(Parser)]
#[command(version)]
pub struct Options {
    /// Set a custom directory for configuration files.
    ///
    /// By default the application will try to find where the config directory is.
    #[arg(short, long, value_name = "FILE")]
    config_dir: Option<PathBuf>,

    /// Turn debugging information on.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: Option<Command>,
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

#[derive(Subcommand)]
pub enum Command {
    /// Run the [`App`], listening to ABCI requests from Tendermint.
    Run {
        /// Optionally override the default configuration.
        #[arg(short, long, default_value = "dev")]
        mode: String,
    },
}
