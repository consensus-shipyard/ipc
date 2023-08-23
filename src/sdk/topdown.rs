// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::config::json_rpc_methods;
use crate::jsonrpc::JsonRpcClient;
use crate::lotus::message::chain::ChainHeadResponse;
use crate::lotus::message::ipc::ValidatorSet;
use crate::sdk::IpcAgentClient;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::CrossMsg;
use ipc_sdk::subnet_id::SubnetID;

impl<T: JsonRpcClient> IpcAgentClient<T> {
    pub async fn get_top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
        nonce: u64,
    ) -> anyhow::Result<Vec<CrossMsg>> {
        self.json_rpc_client
            .request::<Vec<CrossMsg>>(
                json_rpc_methods::LIST_TOPDOWN_MSGS,
                serde_json::json!({
                    "subnet_id": subnet_id.to_string(),
                    "epoch": epoch,
                    "nonce": nonce,
                }),
            )
            .await
    }

    pub async fn get_chain_head(&self, subnet_id: String) -> anyhow::Result<ChainHeadResponse> {
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
        height: ChainEpoch,
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
