// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
// Copyright 2022-2023 Protocol Labs
//! Ipc agent sdk, contains the json rpc client to interact with the IPC agent rpc server.

mod checkpoint;
mod config;
mod crossnet;
mod subnet;
mod wallet;

use crate::jsonrpc::JsonRpcClientImpl;
pub use wallet::LotusJsonKeyType;

use url::Url;

/// The ipc agent client
pub struct IpcAgentClient<T> {
    json_rpc_client: T,
}

impl<T> IpcAgentClient<T> {
    pub fn new(json_rpc_client: T) -> Self {
        Self { json_rpc_client }
    }
}

impl IpcAgentClient<JsonRpcClientImpl> {
    pub fn default_from_url(url: Url) -> Self {
        IpcAgentClient::new(JsonRpcClientImpl::new(url, None))
    }
}
