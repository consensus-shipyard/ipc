// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Contains the chain related RPC request and response messages

use crate::message::CIDMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A simplified struct representing a `ChainHead` response that does not decode the `blocks` field.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChainHeadResponse {
    pub cids: Vec<CIDMap>,
    pub blocks: Vec<Value>,
    pub height: u64,
}
