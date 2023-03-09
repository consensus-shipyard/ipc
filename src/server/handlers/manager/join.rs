// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Create subnet handler and parameters

use crate::manager::SubnetManager;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::JoinParams;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinSubnetParams {
    pub subnet: String,
    pub from: Option<String>,
    pub collateral: u64,
    pub validator_net_addr: String,
}

/// The create subnet json rpc method handler.
pub(crate) struct JoinSubnetHandler {
    pool: Arc<SubnetManagerPool>,
}

impl JoinSubnetHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for JoinSubnetHandler {
    type Request = JoinSubnetParams;
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

        let join_params = JoinParams {
            validator_net_addr: request.validator_net_addr,
        };
        let collateral = TokenAmount::from_atto(request.collateral);
        let from = match request.from {
            Some(addr) => Address::from_str(&addr)?,
            None => conn.subnet().accounts[0],
        };

        conn.manager()
            .join_subnet(subnet, from, collateral, join_params)
            .await
    }
}
