// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Set the subnet actor validator net addr

use crate::manager::SubnetManager;
use crate::server::subnet::SubnetManagerPool;
use crate::server::{check_subnet, parse_from, JsonRPCRequestHandler};
use anyhow::anyhow;
use async_trait::async_trait;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct SetValidatorNetAddrParams {
    pub subnet: String,
    pub from: Option<String>,
    pub validator_net_addr: String,
}

/// Sets a new net address to an existing validator
pub(crate) struct SetValidatorNetAddrHandler {
    pool: Arc<SubnetManagerPool>,
}

impl SetValidatorNetAddrHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for SetValidatorNetAddrHandler {
    type Request = SetValidatorNetAddrParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet = SubnetID::from_str(&request.subnet)?;
        let parent = subnet
            .parent()
            .ok_or_else(|| anyhow!("no parent found"))?
            .to_string();
        let conn = match self.pool.get(&parent) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let from = parse_from(subnet_config, request.from)?;

        conn.manager()
            .set_validator_net_addr(subnet, from, request.validator_net_addr)
            .await
    }
}
