// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::config::json_rpc_methods;
use crate::jsonrpc::JsonRpcClient;
use crate::sdk::IpcAgentClient;
use crate::server::ReloadConfigParams;

impl<T: JsonRpcClient> IpcAgentClient<T> {
    pub async fn reload_config(&self, path: Option<String>) -> anyhow::Result<()> {
        let params = ReloadConfigParams { path };
        self.json_rpc_client
            .request::<()>(
                json_rpc_methods::RELOAD_CONFIG,
                serde_json::to_value(params)?,
            )
            .await
    }
}
