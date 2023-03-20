// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! SendValue subnet handler and parameters

use crate::manager::SubnetManager;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::handlers::manager::{check_subnet, parse_from};
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct SendValueParams {
    pub subnet: String,
    pub from: Option<String>,
    pub to: String,
    pub amount: u64,
}

/// Send value between two addresses within a subnet
pub(crate) struct SendValueHandler {
    pool: Arc<SubnetManagerPool>,
}

impl SendValueHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for SendValueHandler {
    type Request = SendValueParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let conn = match self.pool.get(&request.subnet) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let amount = TokenAmount::from_whole(request.amount); // In FIL, not atto

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let from = parse_from(subnet_config, request.from)?;
        let to = Address::from_str(&request.to)?;

        conn.manager().send_value(from, to, amount).await?;
        Ok(())
    }
}
