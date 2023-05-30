// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub const JSON_RPC_ENDPOINT: &str = "json_rpc";

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Server {
    pub json_rpc_address: SocketAddr,
}

pub mod json_rpc_methods {
    pub const CREATE_SUBNET: &str = "ipc_createSubnet";
    pub const JOIN_SUBNET: &str = "ipc_joinSubnet";
    pub const RPC_SUBNET: &str = "ipc_rpcSubnet";
    pub const LEAVE_SUBNET: &str = "ipc_leaveSubnet";
    pub const KILL_SUBNET: &str = "ipc_killSubnet";
    pub const FUND: &str = "ipc_fund";
    pub const RELEASE: &str = "ipc_release";
    pub const PROPAGATE: &str = "ipc_propagate";
    pub const WHITELIST_PROPAGATOR: &str = "ipc_whitelistPropagator";
    pub const LIST_CHILD_SUBNETS: &str = "ipc_listChildSubnets";
    pub const RELOAD_CONFIG: &str = "ipc_reloadConfig";
    pub const QUERY_VALIDATOR_SET: &str = "ipc_queryValidatorSet";
    pub const SET_VALIDATOR_NET_ADDR: &str = "ipc_setValidatorNetAddr";
    pub const SEND_VALUE: &str = "ipc_sendValue";
    pub const WALLET_NEW: &str = "ipc_walletNew";
    pub const WALLET_IMPORT: &str = "ipc_walletImport";
    pub const WALLET_EXPORT: &str = "ipc_walletExport";
    pub const WALLET_BALANCES: &str = "ipc_walletBalances";
    pub const LIST_BOTTOMUP_CHECKPOINTS: &str = "ipc_listBottomUpCheckpoints";
    pub const LAST_TOPDOWN_EXECUTED: &str = "ipc_lastTopDownCheckpointExecuted";
}
