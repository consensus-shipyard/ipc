// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Send cross network message operation in the gateway actor

use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::{check_subnet, parse_from, JsonRPCRequestHandler};
use anyhow::anyhow;
use async_trait::async_trait;
use ipc_sdk::cross::CrossMsg;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct SendCrossMsgParams {
    pub subnet: String,
    pub from: Option<String>,
    pub cross_message: CrossMsg,
}

/// The WhitelistPropagator json rpc method handler.
pub(crate) struct SendCrossMsgHandler {
    pool: Arc<SubnetManagerPool>,
}

impl SendCrossMsgHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for SendCrossMsgHandler {
    type Request = SendCrossMsgParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet = SubnetID::from_str(&request.subnet)?;
        let conn = match self.pool.get(&subnet) {
            None => return Err(anyhow!("target subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let from = parse_from(subnet_config, request.from)?;

        conn.manager()
            .send_cross_message(subnet_config.gateway_addr(), from, request.cross_message)
            .await
    }
}
