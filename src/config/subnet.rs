// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use serde::Deserialize;
use url::Url;

use crate::config::deserialize::{deserialize_accounts, deserialize_subnet_id};

/// Represents a subnet declaration in the config.
#[derive(Deserialize, Clone, Debug)]
pub struct Subnet {
    #[serde(deserialize_with = "deserialize_subnet_id")]
    pub id: SubnetID,
    pub jsonrpc_api_http: Url,
    pub jsonrpc_api_ws: Option<Url>,
    pub auth_token: Option<String>,
    #[serde(deserialize_with = "deserialize_accounts", default)]
    pub accounts: Vec<Address>,
}
