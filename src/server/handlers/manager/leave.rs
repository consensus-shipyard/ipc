// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Leave subnet handler and parameters

use crate::manager::SubnetManager;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::handlers::manager::{check_subnet, parse_from};
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaveSubnetParams {
    pub subnet: String,
    pub from: Option<String>,
}

/// The create subnet json rpc method handler.
pub(crate) struct LeaveSubnetHandler {
    pool: Arc<SubnetManagerPool>,
}

impl LeaveSubnetHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for LeaveSubnetHandler {
    type Request = LeaveSubnetParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet = SubnetID::from_str(&request.subnet)?;
        let parent = subnet.parent().ok_or_else(|| anyhow!("no parent found"))?;
        let conn = match self.pool.get(&parent) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let from = parse_from(subnet_config, request.from)?;

        conn.manager().leave_subnet(subnet, from).await
    }
}
