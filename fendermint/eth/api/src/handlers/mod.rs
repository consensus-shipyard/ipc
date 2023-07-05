// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::JsonRpcServer;

pub mod http;
pub mod ws;

// Calls an RPC method and returns the full response as a string.
async fn call_rpc_str(
    server: &JsonRpcServer,
    request: jsonrpc_v2::RequestObject,
) -> anyhow::Result<String> {
    let response = server.handle(request).await;
    Ok(serde_json::to_string(&response)?)
}
