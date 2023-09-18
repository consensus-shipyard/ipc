// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The json rpc request param. It is the standard form our json-rpc and follows a structure similar
/// to the one of the Ethereum RPC: https://ethereum.org/en/developers/docs/apis/json-rpc/#curl-examples
#[derive(Serialize, Deserialize, Debug)]
pub struct JSONRPCRequest {
    pub id: u64,
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
}
