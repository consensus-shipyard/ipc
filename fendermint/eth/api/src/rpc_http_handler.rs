// Copyright 2022-2023 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

// Based on https://github.com/ChainSafe/forest/blob/v0.8.2/node/rpc/src/rpc_http_handler.rs

use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use jsonrpc_v2::{Id, RequestObject as JsonRpcRequestObject};

use crate::JsonRpcServer;

/// Handle JSON-RPC calls.
pub async fn handle(
    _headers: HeaderMap,
    axum::extract::State(server): axum::extract::State<JsonRpcServer>,
    axum::Json(request): axum::Json<JsonRpcRequestObject>,
) -> impl IntoResponse {
    let response_headers = [("content-type", "application/json-rpc;charset=utf-8")];

    // NOTE: Any authorization can come here.

    tracing::debug!("RPC request: {request:?}");

    let id = request.id_ref().map(id_to_string).unwrap_or_default();
    let method = request.method_ref().to_owned();

    match call_rpc_str(server.clone(), request).await {
        Ok(result) => {
            tracing::debug!(method, id, result, "RPC call success");
            (StatusCode::OK, response_headers, result)
        }
        Err(err) => {
            let msg = err.to_string();
            tracing::error!(method, id, msg, "RPC call failure");
            (StatusCode::INTERNAL_SERVER_ERROR, response_headers, msg)
        }
    }
}

// Calls an RPC method and returns the full response as a string.
pub async fn call_rpc_str(
    server: JsonRpcServer,
    request: JsonRpcRequestObject,
) -> anyhow::Result<String> {
    let response = server.handle(request).await;
    Ok(serde_json::to_string(&response)?)
}

fn id_to_string(id: &jsonrpc_v2::Id) -> String {
    match id {
        Id::Null => "null".to_owned(),
        Id::Str(s) => (**s).to_owned(),
        Id::Num(n) => n.to_string(),
    }
}
