// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Query the validator set in subnet actor

use crate::lotus::message::ipc::QueryValidatorSetResponse;

use crate::server::handlers::manager::check_subnet;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryValidatorSetParams {
    pub gateway_address: String,
    pub subnet: String,
}

/// The list validators json rpc method handler.
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
        let subnet = SubnetID::from_str(&request.subnet)?;
        let conn = match self.pool.get(&subnet) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let gateway_addr = Address::from_str(&request.gateway_address);
        let val_set = conn
            .manager()
            .get_validator_set(&subnet, gateway_addr.ok())
            .await?;
        log::debug!("list of validators: {val_set:?}");
        Ok(val_set)
    }
}
