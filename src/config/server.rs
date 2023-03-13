// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use serde::Deserialize;
use std::net::SocketAddr;

pub const JSON_RPC_ENDPOINT: &str = "json_rpc";

#[derive(Deserialize, Clone, Debug)]
pub struct Server {
    pub json_rpc_address: SocketAddr,
}

pub mod json_rpc_methods {
    pub const CREATE_SUBNET: &str = "ipc_createSubnet";
    pub const JOIN_SUBNET: &str = "ipc_joinSubnet";
    pub const LEAVE_SUBNET: &str = "ipc_leaveSubnet";
    pub const KILL_SUBNET: &str = "ipc_killSubnet";
    pub const LIST_CHILD_SUBNETS: &str = "ipc_listChildSubnets";
    pub const RELOAD_CONFIG: &str = "ipc_reloadConfig";
    pub const QUERY_VALIDATOR_SET: &str = "ipc_queryValidatorSet";
}
