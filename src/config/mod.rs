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
use deserialize::deserialize_subnets_from_vec;
use ipc_sdk::subnet_id::SubnetID;
pub use reload::ReloadableConfig;
use serde::Deserialize;
pub use server::JSON_RPC_ENDPOINT;
pub use server::{json_rpc_methods, Server};
pub use subnet::Subnet;

pub const JSON_RPC_VERSION: &str = "2.0";

/// Default config template
pub const DEFAULT_CONFIG_TEMPLATE: &str = r#"
[server]
json_rpc_address = "127.0.0.1:3030"

[[subnets]]
id = "/root"
gateway_addr = "t064"
network_name = "root"
jsonrpc_api_http = "http://127.0.0.1:1234/rpc/v1"
jsonrpc_api_ws = "wss://example.org/rpc/v0"
auth_token = "YOUR TOKEN"
accounts = ["t01"]

[[subnets]]
id = "/root/t01"
gateway_addr = "t064"
network_name = "child"
jsonrpc_api_http = "http://127.0.0.1:1250/rpc/v1"
auth_token = "YOUR TOKEN"
accounts = ["t01"]
"#;

/// The top-level struct representing the config. Calls to [`Config::from_file`] deserialize into
/// this struct.
#[derive(Deserialize, Debug)]
pub struct Config {
    pub server: Server,
    #[serde(deserialize_with = "deserialize_subnets_from_vec", default)]
    pub subnets: HashMap<SubnetID, Subnet>,
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
