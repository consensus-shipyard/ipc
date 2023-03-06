// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::config::deserialize::deserialize_network;
use fvm_shared::address::Network;
use serde::Deserialize;
use std::net::SocketAddr;

pub const JSON_RPC_ENDPOINT: &str = "json_rpc";

#[derive(Deserialize, Clone)]
pub struct Server {
    pub json_rpc_address: SocketAddr,
    #[serde(deserialize_with = "deserialize_network")]
    pub network: Network,
}

pub mod json_rpc_methods {
    pub const CREATE_SUBNET: &str = "ipc_createSubnet";
    pub const RELOAD_CONFIG: &str = "ipc_reloadConfig";
}
