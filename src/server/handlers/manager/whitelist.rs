// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! WhitelistPropagator operation in the gateway actor

use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::{check_subnet, parse_from, JsonRPCRequestHandler};
use anyhow::anyhow;
use async_trait::async_trait;
use cid::Cid;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct WhitelistPropagatorParams {
    pub subnet: String,
    pub from: Option<String>,
    pub postbox_msg_cid: Cid,
    pub to_add: Vec<String>,
}

/// The WhitelistPropagator json rpc method handler.
pub(crate) struct WhitelistPropagatorHandler {
    pool: Arc<SubnetManagerPool>,
}

impl WhitelistPropagatorHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for WhitelistPropagatorHandler {
    type Request = WhitelistPropagatorParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet = SubnetID::from_str(&request.subnet)?;
        let conn = match self.pool.get(&subnet) {
            None => return Err(anyhow!("target subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let to_add = request
            .to_add
            .iter()
            .map(|s| Address::from_str(s))
            .collect::<Result<Vec<_>, _>>()?;
        let from = parse_from(subnet_config, request.from)?;

        conn.manager()
            .whitelist_propagator(
                subnet,
                subnet_config.gateway_addr(),
                request.postbox_msg_cid,
                from,
                to_add,
            )
            .await
    }
}
