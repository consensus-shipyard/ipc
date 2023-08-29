// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! List top down messages

use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::cross::CrossMsg;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};

use crate::server::handlers::manager::check_subnet;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;

#[derive(Debug, Serialize, Deserialize)]
pub struct TopDownMsgsParams {
    pub subnet_id: String,
    pub start_epoch: ChainEpoch,
    pub end_epoch: ChainEpoch,
}

/// The epoch of the latest top-down checkpoint executed
pub(crate) struct TopDownMsgsHandler {
    pool: Arc<SubnetManagerPool>,
}

impl TopDownMsgsHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for TopDownMsgsHandler {
    type Request = TopDownMsgsParams;
    type Response = Vec<CrossMsg>;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let child_subnet_id = SubnetID::from_str(request.subnet_id.as_str())?;
        let parent = child_subnet_id
            .parent()
            .ok_or_else(|| anyhow!("root does not have parent"))?;

        let conn = match self.pool.get(&parent) {
            None => return Err(anyhow!("target subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        Ok(conn
            .manager()
            .get_top_down_msgs(&child_subnet_id, request.start_epoch, request.end_epoch)
            .await?)
    }
}
