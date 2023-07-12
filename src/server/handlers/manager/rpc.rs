// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! RPC subnet handler and parameters

use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct RPCSubnetParams {
    pub subnet: String,
}

/// The create subnet json rpc method handler.
pub(crate) struct RPCSubnetHandler {
    pool: Arc<SubnetManagerPool>,
}

impl RPCSubnetHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for RPCSubnetHandler {
    type Request = RPCSubnetParams;
    type Response = String;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet = SubnetID::from_str(&request.subnet)?;
        let conn = match self.pool.get(&subnet) {
            None => return Err(anyhow!("target subnet not found")),
            Some(conn) => conn,
        };

        Ok(conn.subnet().rpc_http().to_string())
    }
}
