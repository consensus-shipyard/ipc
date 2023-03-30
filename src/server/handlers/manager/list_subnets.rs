// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! List subnets in gateway actor

use crate::lotus::message::ipc::SubnetInfo;
use crate::manager::SubnetManager;
use crate::server::handlers::manager::check_subnet;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListSubnetsParams {
    pub gateway_address: String,
    pub subnet_id: String,
}

/// The create subnet json rpc method handler.
pub(crate) struct ListSubnetsHandler {
    pool: Arc<SubnetManagerPool>,
}

impl ListSubnetsHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for ListSubnetsHandler {
    type Request = ListSubnetsParams;
    type Response = HashMap<String, SubnetInfo>;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet = SubnetID::from_str(&request.subnet_id)?;
        let conn = match self.pool.get(&subnet) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let gateway_addr = Address::from_str(&request.gateway_address)?;
        let subnet_map = conn.manager().list_child_subnets(gateway_addr).await?;
        Ok(subnet_map
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect())
    }
}
