// Copyright 2022-2023 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

// Based on https://github.com/ChainSafe/forest/blob/v0.8.2/node/rpc/src/rpc_ws_handler.rs

use anyhow::Context;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    http::HeaderMap,
    response::IntoResponse,
};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use fvm_shared::error::ExitCode;

use crate::{handlers::call_rpc_str, JsonRpcServer};

pub async fn handle(
    _headers: HeaderMap,
    axum::extract::State(server): axum::extract::State<JsonRpcServer>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async { rpc_ws_handler_inner(server, socket).await })
}

/// Handle requests in a loop, interpreting each message as a JSON-RPC request.
///
/// Messages are evaluated one by one. We could spawn tasks like Forest,
/// but there should be some rate limiting applied to avoid DoS attacks.
async fn rpc_ws_handler_inner(server: JsonRpcServer, socket: WebSocket) {
    tracing::debug!("Accepted WS connection!");
    let (mut sender, mut receiver) = socket.split();
    while let Some(Ok(message)) = receiver.next().await {
        tracing::debug!("Received new WS RPC message: {:?}", message);

        if let Message::Text(request_text) = message {
            tracing::debug!("WS RPC Request: {}", request_text);

            if !request_text.is_empty() {
                tracing::debug!("RPC Request Received: {:?}", &request_text);

                match serde_json::from_str(&request_text)
                    as Result<jsonrpc_v2::RequestObject, serde_json::Error>
                {
                    Ok(req) => match rpc_ws_call(&server, &mut sender, req).await {
                        Ok(()) => {
                            tracing::debug!("WS RPC task success.");
                        }
                        Err(e) => {
                            tracing::warn!("failed to send response to WS: {e}");
                        }
                    },
                    Err(e) => {
                        let msg = format!("Error deserializing WS request payload: {e}");
                        tracing::error!("{}", msg);
                        if let Err(e) = sender
                            .send(Message::Text(error_str(
                                ExitCode::USR_SERIALIZATION.value() as i64,
                                msg,
                            )))
                            .await
                        {
                            tracing::warn!("failed to send error response to WS: {e}");
                        }
                    }
                }
            }
        }
    }
}

/// Call the RPC method and respond through the Web Socket.
async fn rpc_ws_call(
    server: &JsonRpcServer,
    sender: &mut SplitSink<WebSocket, Message>,
    request: jsonrpc_v2::RequestObject,
) -> anyhow::Result<()> {
    let method = request.method_ref();

    tracing::debug!("RPC WS called method: {}", method);

    match call_rpc_str(server, request).await {
        Ok(response) => sender
            .send(Message::Text(response))
            .await
            .context("failed to send success result to WS"),
        Err(e) => {
            tracing::error!("RPC call failed: {}", e);
            sender
                .send(Message::Text(error_str(
                    ExitCode::USR_UNSPECIFIED.value() as i64,
                    e.to_string(),
                )))
                .await
                .context("failed to send error result to WS")
        }
    }
}

pub fn error_res(code: i64, message: String) -> jsonrpc_v2::ResponseObject {
    jsonrpc_v2::ResponseObject::Error {
        jsonrpc: jsonrpc_v2::V2,
        error: jsonrpc_v2::Error::Full {
            code,
            message,
            data: None,
        },
        id: jsonrpc_v2::Id::Null,
    }
}

pub fn error_str(code: i64, message: String) -> String {
    match serde_json::to_string(&error_res(code, message)) {
        Ok(err_str) => err_str,
        Err(err) => format!("Failed to serialize error data. Error was: {err}"),
    }
}
