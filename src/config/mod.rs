// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Provides a simple way of reading configuration files.
//!
//! Reads a TOML config file for the IPC Agent and deserializes it in a type-safe way into a
//! [`Config`] struct.

mod deserialize;
mod server;
mod subnet;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
pub use server::Server;
pub use server::JSON_RPC_ENDPOINT;
pub use subnet::Subnet;

pub const JSON_RPC_VERSION: &str = "2.0";
pub const IPC_GATEWAY_ADDR: u64 = 64;

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
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::str::FromStr;

    use fvm_shared::address::Address;
    use indoc::formatdoc;
    use ipc_sdk::subnet_id::{SubnetID, ROOTNET_ID};
    use url::Url;

    use crate::config::Config;

    // Arguments for the config's fields
    const SERVER_JSON_RPC_ADDR: &str = "127.0.0.1:3030";
    const ROOT_ID: &str = "/root";
    const CHILD_ID: &str = "/root/f0100";
    const ROOT_AUTH_TOKEN: &str = "ROOT_AUTH_TOKEN";
    const CHILD_AUTH_TOKEN: &str = "CHILD_AUTH_TOKEN";
    const JSONRPC_API_HTTP: &str = "https://example.org/rpc/v0";
    const JSONRPC_API_WS: &str = "ws://example.org/rpc/v0";
    const ACCOUNT_ADDRESS: &str =
        "f3thgjtvoi65yzdcoifgqh6utjbaod3ukidxrx34heu34d6avx6z7r5766t5jqt42a44ehzcnw3u5ehz47n42a";

    #[test]
    fn check_server_config() {
        let config = read_config().server;
        assert_eq!(
            config.json_rpc_address,
            SocketAddr::from_str(SERVER_JSON_RPC_ADDR).unwrap(),
            "invalid server rpc address"
        );
    }

    #[test]
    fn check_subnets_config() {
        let config = read_config().subnets;

        let root = &config["root"];
        assert_eq!(root.id, *ROOTNET_ID);
        assert_eq!(
            root.jsonrpc_api_http,
            Url::from_str(JSONRPC_API_HTTP).unwrap()
        );
        assert_eq!(
            root.jsonrpc_api_ws.as_ref().unwrap(),
            &Url::from_str(JSONRPC_API_WS).unwrap()
        );
        assert_eq!(root.auth_token.as_ref().unwrap(), ROOT_AUTH_TOKEN);

        let child = &config["child"];
        assert_eq!(child.id, SubnetID::from_str(CHILD_ID).unwrap(),);
        assert_eq!(
            child.jsonrpc_api_http,
            Url::from_str(JSONRPC_API_HTTP).unwrap(),
        );
        assert_eq!(child.auth_token.as_ref().unwrap(), CHILD_AUTH_TOKEN,);
        assert_eq!(
            child.accounts.as_ref(),
            vec![Address::from_str(ACCOUNT_ADDRESS).unwrap()],
        );
    }

    fn read_config() -> Config {
        let config_str = formatdoc!(
            r#"
            [server]
            json_rpc_address = "{SERVER_JSON_RPC_ADDR}"

            [subnets]

            [subnets.root]
            id = "{ROOT_ID}"
            jsonrpc_api_http = "{JSONRPC_API_HTTP}"
            jsonrpc_api_ws = "{JSONRPC_API_WS}"
            auth_token = "{ROOT_AUTH_TOKEN}"

            [subnets.child]
            id = "{CHILD_ID}"
            jsonrpc_api_http = "{JSONRPC_API_HTTP}"
            auth_token = "{CHILD_AUTH_TOKEN}"
            accounts = ["{ACCOUNT_ADDRESS}"]
        "#
        );

        Config::from_toml_str(config_str.as_str()).unwrap()
    }
}
