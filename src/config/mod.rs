// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Provides a simple way of reading configuration files.
//!
//! Reads a TOML config file for the IPC Agent and deserializes it in a type-safe way into a
//! [`Config`] struct.

mod deserialize;
mod reload;
mod server;
pub mod subnet;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
pub use reload::ReloadableConfig;
use serde::Deserialize;
pub use server::JSON_RPC_ENDPOINT;
pub use server::{json_rpc_methods, Server};
pub use subnet::Subnet;

pub const JSON_RPC_VERSION: &str = "2.0";
pub const DEFAULT_IPC_GATEWAY_ADDR: u64 = 64;

/// The top-level struct representing the config. Calls to [`Config::from_file`] deserialize into
/// this struct.
#[derive(Deserialize)]
pub struct Config {
    pub server: Server,
    pub subnets: HashMap<String, Subnet>,
}

impl Config {
    /// Reads a TOML configuration in the `s` string and returns a [`Config`] struct.
    pub fn from_toml_str(s: &str) -> Result<Self> {
        let config = toml::from_str(s)?;
        Ok(config)
    }

    /// Reads a TOML configuration file specified in the `path` and returns a [`Config`] struct.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = Config::from_toml_str(contents.as_str())?;
        Ok(config)
    }

    /// Reads a TOML configuration file specified in the `path` and returns a [`Config`] struct.
    pub async fn from_file_async(path: impl AsRef<Path>) -> Result<Self> {
        let contents = tokio::fs::read_to_string(path).await?;
        Config::from_toml_str(contents.as_str())
    }
}
