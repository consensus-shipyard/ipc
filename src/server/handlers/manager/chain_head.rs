// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Get the chain head of the subnet

use crate::lotus::message::chain::ChainHeadResponse;
use crate::server::handlers::manager::check_subnet;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainHeadParams {
    pub subnet_id: String,
}

/// The create subnet json rpc method handler.
pub(crate) struct ChainHeadHandler {
    pool: Arc<SubnetManagerPool>,
}

impl ChainHeadHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for ChainHeadHandler {
    type Request = ChainHeadParams;
    type Response = ChainHeadResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet = SubnetID::from_str(&request.subnet_id)?;
        let conn = match self.pool.get(&subnet) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        conn.manager().chain_head().await
    }
}
