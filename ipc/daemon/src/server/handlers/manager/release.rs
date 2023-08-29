// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Release operation in the gateway actor

use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::{check_subnet, handlers, parse_from, JsonRPCRequestHandler};
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseParams {
    pub subnet: String,
    pub from: Option<String>,
    pub to: Option<String>,
    /// In whole FIL
    pub amount: f64,
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

        let amount = handlers::f64_to_token_amount(request.amount)?;
        let from = parse_from(subnet_config, request.from)?;
        let to = request
            .to
            .map(|r| Address::from_str(&r))
            .transpose()?
            .unwrap_or(from);

        conn.manager()
            .release(subnet, subnet_config.gateway_addr(), from, to, amount)
            .await
    }
}
