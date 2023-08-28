// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Set the subnet actor validator worker addr

use crate::server::subnet::SubnetManagerPool;
use crate::server::{check_subnet, parse_from, JsonRPCRequestHandler};
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct SetValidatorWorkerAddrParams {
    pub subnet: String,
    pub from: Option<String>,
    pub validator_worker_addr: String,
}

/// Sets a new worker address to an existing validator
pub(crate) struct SetValidatorWorkerAddrHandler {
    pool: Arc<SubnetManagerPool>,
}

impl SetValidatorWorkerAddrHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for SetValidatorWorkerAddrHandler {
    type Request = SetValidatorWorkerAddrParams;
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

        let worker_addr = Address::from_str(&request.validator_worker_addr)?;

        conn.manager()
            .set_validator_worker_addr(subnet, from, worker_addr)
            .await
    }
}
