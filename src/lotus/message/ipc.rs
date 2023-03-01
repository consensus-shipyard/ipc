// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use serde::Deserialize;

use crate::lotus::message::CIDMap;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct IPCGetPrevCheckpointForChildResponse {
    #[serde(rename = "CID")]
    pub cid: CIDMap,
}
