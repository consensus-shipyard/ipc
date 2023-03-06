// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

mod commands;

use crate::config::Config;
pub use commands::cli;

const DEFAULT_CONFIG_PATH: &str = ".ipc_agent/config.toml";

/// The trait that represents the abstraction of a command line handler. To implement a new command
/// line operation, implement this trait and register it.
///
/// Note that this trait does not support a stateful implementation as we assume CLI commands are all
/// constructed from scratch.
#[async_trait]
pub trait CommandLineHandler {
    /// Abstraction for command line operations arguments.
    ///
    /// NOTE that this parameter is used to generate the command line arguments.
    /// Currently we are directly integrating with `clap` crate. In the future we can use our own
    /// implementation to abstract away external crates. But this should be good for now.
    type Arguments: std::fmt::Debug + Args;

    /// Handles the request with the provided arguments. Dev should handle the content to print and how
    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()>;
}

/// The global arguments that will be shared by all cli commands.
#[derive(Debug, Args, Clone)]
pub struct GlobalArguments {
    #[arg(
        help = "The toml config file path for IPC Agent, default to ${HOME}/.ipc_agent/config.toml"
    )]
    config_path: Option<String>,
}

impl GlobalArguments {
    pub fn config_path(&self) -> String {
        self.config_path.clone().unwrap_or_else(|| {
            let home = match std::env::var("HOME") {
                Ok(home) => home,
                Err(_) => panic!("cannot get home"),
            };
            format!("{home:}/{:}", DEFAULT_CONFIG_PATH)
        })
    }

    pub fn config(&self) -> Result<Config> {
        let config_path = self.config_path();
        Config::from_file(config_path)
    }
}
