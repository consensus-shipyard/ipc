// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! List checkpoints in subnet actor

use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};

use crate::lotus::message::ipc::CheckpointResponse;
use crate::manager::SubnetManager;
use crate::server::handlers::manager::check_subnet;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListCheckpointsParams {
    pub subnet_id: String,
    pub from_epoch: ChainEpoch,
    pub to_epoch: ChainEpoch,
}

/// The list checkpoints json rpc method handler.
pub(crate) struct ListCheckpointsHandler {
    pool: Arc<SubnetManagerPool>,
}

impl ListCheckpointsHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for ListCheckpointsHandler {
    type Request = ListCheckpointsParams;
    type Response = Vec<CheckpointResponse>;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let child_subnet_id = SubnetID::from_str(request.subnet_id.as_str())?;
        let parent_subnet_id = child_subnet_id
            .parent()
            .ok_or_else(|| anyhow!("subnet id does not have a parent"))?;

        let conn = match self.pool.get(&parent_subnet_id) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let checkpoints: Vec<CheckpointResponse> = conn
            .manager()
            .list_checkpoints(child_subnet_id, request.from_epoch, request.to_epoch)
            .await?;
        Ok(checkpoints)
    }
}
