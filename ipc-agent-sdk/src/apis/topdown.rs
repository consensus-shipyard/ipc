// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::apis::{json_rpc_methods, IpcAgentClient};
use crate::jsonrpc::JsonRpcClient;
use crate::message::chain::ChainHeadResponse;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::cross::CrossMsg;
use ipc_sdk::subnet_id::SubnetID;
use ipc_sdk::ValidatorSet;

impl<T: JsonRpcClient> IpcAgentClient<T> {
    pub async fn get_top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        start: ChainEpoch,
        to: ChainEpoch,
    ) -> anyhow::Result<Vec<CrossMsg>> {
        self.json_rpc_client
            .request::<Vec<CrossMsg>>(
                json_rpc_methods::LIST_TOPDOWN_MSGS,
                serde_json::json!({
                    "subnet_id": subnet_id.to_string(),
                    "start": start,
                    "to": to,
                }),
            )
            .await
    }

    pub async fn get_chain_head(&self, subnet_id: &SubnetID) -> anyhow::Result<ChainHeadResponse> {
        self.json_rpc_client
            .request::<ChainHeadResponse>(
                json_rpc_methods::CHAIN_HEAD,
                serde_json::json!({
                    "subnet_id": subnet_id.to_string()
                }),
            )
            .await
    }

    pub async fn get_block_hash(
        &self,
        subnet_id: &str,
        height: ChainEpoch,
    ) -> anyhow::Result<Vec<u8>> {
        self.json_rpc_client
            .request::<Vec<u8>>(
                json_rpc_methods::GET_BLOCK_HASH,
                serde_json::json!({
                    "subnet_id": subnet_id.to_string(),
                    "height": height
                }),
            )
            .await
    }

    pub async fn get_validator_set(
        &self,
        subnet_id: &str,
        height: Option<ChainEpoch>,
    ) -> anyhow::Result<ValidatorSet> {
        self.json_rpc_client
            .request::<ValidatorSet>(
                json_rpc_methods::QUERY_VALIDATOR_SET,
                serde_json::json!({
                    "subnet": subnet_id.to_string(),
                    "epoch": height
                }),
            )
            .await
    }
}
