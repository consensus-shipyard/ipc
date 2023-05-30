// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Cross net related sdk functions

use crate::config::json_rpc_methods;
use crate::jsonrpc::JsonRpcClient;
use crate::sdk::IpcAgentClient;
use crate::server::fund::FundParams;
use crate::server::release::ReleaseParams;
use fvm_shared::clock::ChainEpoch;

impl<T: JsonRpcClient> IpcAgentClient<T> {
    pub async fn fund(
        &self,
        subnet: &str,
        from: Option<String>,
        amount: f64,
    ) -> anyhow::Result<ChainEpoch> {
        let params = FundParams {
            subnet: subnet.to_string(),
            from,
            amount,
        };

        self.json_rpc_client
            .request::<ChainEpoch>(json_rpc_methods::FUND, serde_json::to_value(params)?)
            .await
    }

    pub async fn release(
        &self,
        subnet: &str,
        from: Option<String>,
        amount: f64,
    ) -> anyhow::Result<ChainEpoch> {
        let params = ReleaseParams {
            subnet: subnet.to_string(),
            from,
            amount,
        };
        self.json_rpc_client
            .request::<ChainEpoch>(json_rpc_methods::RELEASE, serde_json::to_value(params)?)
            .await
    }
}
