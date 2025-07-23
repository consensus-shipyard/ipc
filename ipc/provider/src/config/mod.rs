// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Provides a simple way of reading configuration files.
//!
//! Reads a TOML config file for the IPC Agent and deserializes it in a type-safe way into a
//! [`Config`] struct.

pub mod deserialize;
pub mod subnet;

pub mod serialize;
#[cfg(test)]
mod tests;

use fs_err as fs;
use std::collections::HashMap;
use std::path::Path;

use crate::config::deserialize::eth_addr_str_to_address;
use anyhow::{Context, Result};
use deserialize::deserialize_subnets_from_vec;
use ipc_api::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use serialize::serialize_subnets_to_str;
pub use subnet::{EVMSubnet, Subnet, SubnetConfig};

pub const JSON_RPC_VERSION: &str = "2.0";

const DEFAULT_KEYSTORE: &str = "~/.ipc";
const CALIBRATION_ID: &str = "/r314159";
const CALIBRATION_RPC: &str = "https://api.calibration.node.glif.io/rpc/v1";
const CALIBRATION_GATEWAY: &str = "0x1AEe8A878a22280fc2753b3C63571C8F895D2FE3";
const CALIBRATION_REGISTRY: &str = "0x0b4e239FF21b40120cDa817fba77bD1B366c1bcD";

/// The top-level struct representing the config. Calls to [`Config::from_file`] deserialize into
/// this struct.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Config {
    /// Directory of the keystore that wants to be made available by the provider.
    pub keystore_path: Option<String>,
    #[serde(deserialize_with = "deserialize_subnets_from_vec", default)]
    #[serde(serialize_with = "serialize_subnets_to_str")]
    pub subnets: HashMap<SubnetID, Subnet>,
}

impl Config {
    /// Returns an empty config to be populated further
    pub fn new() -> Self {
        Config {
            keystore_path: None,
            subnets: Default::default(),
        }
    }

    /// Reads a TOML configuration in the `s` string and returns a [`Config`] struct.
    pub fn from_toml_str(s: &str) -> Result<Self> {
        let config = toml::from_str(s)?;
        Ok(config)
    }

    /// Reads a TOML configuration file specified in the `path` and returns a [`Config`] struct.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let contents = fs::read_to_string(&path).with_context(|| {
            format!(
                "failed to read config from {}",
                path.as_ref().to_string_lossy()
            )
        })?;

        let config: Config =
            Config::from_toml_str(contents.as_str()).context("failed to parse config TOML")?;

        Ok(config)
    }

    /// Reads a TOML configuration file specified in the `path` and returns a [`Config`] struct.
    pub async fn from_file_async(path: impl AsRef<Path>) -> Result<Self> {
        let contents = tokio::fs::read_to_string(path).await?;
        Config::from_toml_str(contents.as_str())
    }

    pub async fn write_to_file_async(&self, path: impl AsRef<Path>) -> Result<()> {
        // Ensure parent directories exist
        let path_ref = path.as_ref();
        if let Some(parent) = path_ref.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Serialize and write
        let content = toml::to_string(self)?;
        tokio::fs::write(path_ref, content.into_bytes()).await?;
        Ok(())
    }

    pub fn add_subnet(&mut self, subnet: Subnet) {
        self.subnets.insert(subnet.id.clone(), subnet);
    }

    pub fn remove_subnet(&mut self, subnet_id: &SubnetID) {
        self.subnets.remove(subnet_id);
    }
}

impl Default for Config {
    fn default() -> Self {
        let subnet_id: SubnetID = CALIBRATION_ID.parse().expect("hard-coded ID must parse");

        let provider_http = CALIBRATION_RPC.parse().expect("RPC URL is valid");

        let gateway_addr =
            eth_addr_str_to_address(CALIBRATION_GATEWAY).expect("invalid gateway address");
        let registry_addr =
            eth_addr_str_to_address(CALIBRATION_REGISTRY).expect("invalid registry address");

        let fevm = EVMSubnet {
            provider_http,
            gateway_addr,
            registry_addr,
            provider_timeout: None,
            auth_token: None,
        };

        let mut subnets = HashMap::new();
        subnets.insert(
            subnet_id.clone(),
            Subnet {
                id: subnet_id,
                config: SubnetConfig::Fevm(fevm),
            },
        );

        Config {
            keystore_path: Some(DEFAULT_KEYSTORE.to_string()),
            subnets,
        }
    }
}
