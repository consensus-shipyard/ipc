// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Propagate operation in the gateway actor

use crate::manager::SubnetManager;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::{check_subnet, parse_from, JsonRPCRequestHandler};
use anyhow::anyhow;
use async_trait::async_trait;
use cid::Cid;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct PropagateParams {
    pub subnet: String,
    pub from: Option<String>,
    pub postbox_msg_cid: Cid,
}

/// The Propagate json rpc method handler.
pub(crate) struct PropagateHandler {
    pool: Arc<SubnetManagerPool>,
}

impl PropagateHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for PropagateHandler {
    type Request = PropagateParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let conn = match self.pool.get(&request.subnet) {
            None => return Err(anyhow!("target subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let from = parse_from(subnet_config, request.from)?;
        let subnet = SubnetID::from_str(&request.subnet)?;

        conn.manager()
            .propagate(subnet, from, request.postbox_msg_cid)
            .await
    }
}
