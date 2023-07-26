// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Expose the subnet actor validator set

use crate::lotus::message::ipc::QueryValidatorSetResponse;
use crate::server::subnet::SubnetManagerPool;
use crate::server::{check_subnet, JsonRPCRequestHandler};
use anyhow::anyhow;
use async_trait::async_trait;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryValidatorSetParams {
    pub subnet: String,
}

/// The create subnet json rpc method handler.
pub(crate) struct QueryValidatorSetHandler {
    pool: Arc<SubnetManagerPool>,
}

impl QueryValidatorSetHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for QueryValidatorSetHandler {
    type Request = QueryValidatorSetParams;
    type Response = QueryValidatorSetResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet_id = SubnetID::from_str(&request.subnet)?;
        let parent = subnet_id
            .parent()
            .ok_or_else(|| anyhow!("cannot get for root"))?;

        let conn = match self.pool.get(&parent) {
            None => return Err(anyhow!("target subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        conn.manager()
            .get_validator_set(&subnet_id, Some(subnet_config.gateway_addr()))
            .await
    }
}
