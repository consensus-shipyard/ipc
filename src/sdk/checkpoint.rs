// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Checkpoint related sdk functions

use crate::config::json_rpc_methods;
use crate::jsonrpc::JsonRpcClient;
use crate::sdk::IpcAgentClient;
use crate::server::list_checkpoints::ListBottomUpCheckpointsParams;
use crate::server::topdown_executed::LastTopDownExecParams;
use fvm_shared::clock::ChainEpoch;

impl<T: JsonRpcClient> IpcAgentClient<T> {
    pub async fn last_top_down_executed(&self, subnet: &str) -> anyhow::Result<ChainEpoch> {
        let params = LastTopDownExecParams {
            subnet_id: subnet.to_string(),
        };

        self.json_rpc_client
            .request::<ChainEpoch>(
                json_rpc_methods::LAST_TOPDOWN_EXECUTED,
                serde_json::to_value(params)?,
            )
            .await
    }

    pub async fn list_bottom_up_checkpoints(
        &self,
        subnet: &str,
        start: ChainEpoch,
        end: ChainEpoch,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let params = ListBottomUpCheckpointsParams {
            subnet_id: subnet.to_string(),
            from_epoch: start,
            to_epoch: end,
        };

        self.json_rpc_client
            .request::<Vec<serde_json::Value>>(
                json_rpc_methods::LIST_BOTTOMUP_CHECKPOINTS,
                serde_json::to_value(params)?,
            )
            .await
    }
}
