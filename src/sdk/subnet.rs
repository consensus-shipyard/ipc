// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::config::json_rpc_methods;
use crate::jsonrpc::JsonRpcClient;
use crate::sdk::IpcAgentClient;
use crate::server::create::{CreateSubnetParams, CreateSubnetResponse};
use crate::server::join::JoinSubnetParams;
use crate::server::kill::KillSubnetParams;
use crate::server::leave::LeaveSubnetParams;

impl<T: JsonRpcClient> IpcAgentClient<T> {
    pub async fn create_subnet(&self, params: CreateSubnetParams) -> anyhow::Result<String> {
        Ok(self
            .json_rpc_client
            .request::<CreateSubnetResponse>(
                json_rpc_methods::CREATE_SUBNET,
                serde_json::to_value(params)?,
            )
            .await?
            .address)
    }

    pub async fn join_subnet(&self, params: JoinSubnetParams) -> anyhow::Result<()> {
        self.json_rpc_client
            .request::<()>(json_rpc_methods::JOIN_SUBNET, serde_json::to_value(params)?)
            .await
    }

    pub async fn leave_subnet(&self, params: LeaveSubnetParams) -> anyhow::Result<()> {
        self.json_rpc_client
            .request::<()>(
                json_rpc_methods::LEAVE_SUBNET,
                serde_json::to_value(params)?,
            )
            .await
    }

    pub async fn kill_subnet(&self, params: KillSubnetParams) -> anyhow::Result<()> {
        self.json_rpc_client
            .request::<()>(json_rpc_methods::KILL_SUBNET, serde_json::to_value(params)?)
            .await
    }
}
