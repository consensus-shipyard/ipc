// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Release operation in the gateway actor

use crate::manager::SubnetManager;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::{check_subnet, parse_from, JsonRPCRequestHandler};
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseParams {
    pub subnet: String,
    pub from: Option<String>,
    pub amount: u64,
}

/// The Release json rpc method handler.
pub(crate) struct ReleaseHandler {
    pool: Arc<SubnetManagerPool>,
}

impl ReleaseHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for ReleaseHandler {
    type Request = ReleaseParams;
    type Response = ChainEpoch;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet = SubnetID::from_str(&request.subnet)?;
        let conn = match self.pool.get(&subnet) {
            None => return Err(anyhow!("target subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let amount = TokenAmount::from_whole(request.amount);
        let from = parse_from(subnet_config, request.from)?;

        conn.manager()
            .release(subnet, subnet_config.gateway_addr, from, amount)
            .await
    }
}
