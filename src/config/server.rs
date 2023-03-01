// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use serde::Deserialize;
use std::net::SocketAddr;

pub const JSON_RPC_ENDPOINT: &str = "json_rpc";

#[derive(Deserialize, Clone)]
pub struct Server {
    pub json_rpc_address: SocketAddr,
}
