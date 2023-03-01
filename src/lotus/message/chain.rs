// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use serde::Deserialize;
use serde_json::Value;

use crate::lotus::message::CIDMap;

/// A simplified struct representing a `ChainHead` response that does not decode the `blocks` field.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChainHeadResponse {
    #[allow(dead_code)]
    pub cids: Vec<CIDMap>,
    #[allow(dead_code)]
    pub blocks: Vec<Value>,
    #[allow(dead_code)]
    pub height: u64,
}
