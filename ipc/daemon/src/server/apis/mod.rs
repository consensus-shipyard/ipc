// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

mod topdown;

use crate::jsonrpc::JsonRpcClientImpl;

use url::Url;

pub mod json_rpc_methods {
    pub const CHAIN_HEAD_HEIGHT: &str = "ipc_chainHeadHeight";
    pub const CREATE_SUBNET: &str = "ipc_createSubnet";
    pub const JOIN_SUBNET: &str = "ipc_joinSubnet";
    pub const RPC_SUBNET: &str = "ipc_rpcSubnet";
    pub const LEAVE_SUBNET: &str = "ipc_leaveSubnet";
    pub const KILL_SUBNET: &str = "ipc_killSubnet";
    pub const FUND: &str = "ipc_fund";
    pub const RELEASE: &str = "ipc_release";
    pub const PROPAGATE: &str = "ipc_propagate";
    pub const SEND_CROSS_MSG: &str = "ipc_sendCrossMessage";
    pub const LIST_CHILD_SUBNETS: &str = "ipc_listChildSubnets";
    pub const RELOAD_CONFIG: &str = "ipc_reloadConfig";
    pub const QUERY_VALIDATOR_SET: &str = "ipc_queryValidatorSet";
    pub const SET_VALIDATOR_NET_ADDR: &str = "ipc_setValidatorNetAddr";
    pub const SEND_VALUE: &str = "ipc_sendValue";
    pub const WALLET_NEW: &str = "ipc_walletNew";
    pub const WALLET_REMOVE: &str = "ipc_walletRemove";
    pub const WALLET_IMPORT: &str = "ipc_walletImport";
    pub const WALLET_EXPORT: &str = "ipc_walletExport";
    pub const WALLET_BALANCES: &str = "ipc_walletBalances";
    pub const LIST_BOTTOMUP_CHECKPOINTS: &str = "ipc_listBottomUpCheckpoints";
    pub const LAST_TOPDOWN_EXECUTED: &str = "ipc_lastTopDownCheckpointExecuted";
    pub const LIST_TOPDOWN_MSGS: &str = "ipc_listTopDownMsgs";
    pub const GET_BLOCK_HASH: &str = "ipc_getBlockHash";
}

/// The ipc agent client
pub struct IpcAgentClient<T> {
    json_rpc_client: T,
}

impl<T> IpcAgentClient<T> {
    pub fn new(json_rpc_client: T) -> Self {
        Self { json_rpc_client }
    }
}

impl IpcAgentClient<JsonRpcClientImpl> {
    pub fn default_from_url(url: Url) -> Self {
        IpcAgentClient::new(JsonRpcClientImpl::new(url, None))
    }
}
