// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Epoch of the last top-down checkpoint executed

use std::str::FromStr;
use std::sync::Arc;

use crate::manager::SubnetManager;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};

use crate::server::handlers::manager::check_subnet;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;

#[derive(Debug, Serialize, Deserialize)]
pub struct LastTopDownExecParams {
    pub subnet_id: String,
}

/// The epoch of the latest top-down checkpoint executed
pub(crate) struct LastTopDownExecHandler {
    pool: Arc<SubnetManagerPool>,
}

impl LastTopDownExecHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for LastTopDownExecHandler {
    type Request = LastTopDownExecParams;
    type Response = ChainEpoch;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let child_subnet_id = SubnetID::from_str(request.subnet_id.as_str())?;
        let conn = match self.pool.get(&child_subnet_id) {
            None => return Err(anyhow!("target subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        Ok(conn.manager().last_topdown_executed().await?)
    }
}
