// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use anyhow::{anyhow, Result};
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::header::HeaderValue;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::{connect_async, WebSocketStream};
use url::Url;

#[cfg(test)]
mod tests;

const DEFAULT_JSON_RPC_VERSION: &str = "2.0";
const DEFAULT_JSON_RPC_ID: u8 = 1;

/// A convenience constant that represents empty params in a JSON-RPC request.
pub const NO_PARAMS: Value = json!([]);

/// A simple async JSON-RPC client that can send one-shot request via HTTP/HTTPS
/// and subscribe to push-based notifications via Websockets. The returned
/// results are of type [`Value`] from the [`serde_json`] crate.
#[async_trait]
pub trait JsonRpcClient {
    /// Sends a JSON-RPC request with `method` and `params` via HTTP/HTTPS.
    async fn request<T: DeserializeOwned>(&self, method: &str, params: Value) -> Result<T>;

    /// Subscribes to notifications via a Websocket. This returns a [`Receiver`]
    /// channel that is used to receive the messages sent by the server.
    /// TODO: https://github.com/consensus-shipyard/ipc-agent/issues/7.
    async fn subscribe(&self, method: &str) -> Result<Receiver<Value>>;
}

/// The implementation of [`JsonRpcClient`].
pub struct JsonRpcClientImpl {
    http_client: Client,
    url: Url,
    bearer_token: Option<String>,
}

impl JsonRpcClientImpl {
    /// Creates a client that sends all requests to `url`.
    pub fn new(url: Url, bearer_token: Option<&str>) -> Self {
        Self {
            http_client: Client::default(),
            url,
            bearer_token: bearer_token.map(String::from),
        }
    }
}

#[async_trait]
impl JsonRpcClient for JsonRpcClientImpl {
    async fn request<T: DeserializeOwned>(&self, method: &str, params: Value) -> Result<T> {
        let request_body = build_jsonrpc_request(method, params)?;
        let mut builder = self.http_client.post(self.url.as_str()).json(&request_body);

        // Add the authorization bearer token if present
        if self.bearer_token.is_some() {
            builder = builder.bearer_auth(self.bearer_token.as_ref().unwrap());
        }

        let response = builder.send().await?;

        let response_body = response.text().await?;
        log::debug!("received raw response body: {:?}", response_body);

        let value = serde_json::from_str::<JsonRpcResponse<T>>(response_body.as_ref())?;

        if value.id != DEFAULT_JSON_RPC_ID || value.jsonrpc != DEFAULT_JSON_RPC_VERSION {
            return Err(anyhow!("json_rpc id or version not matching."));
        }

        Result::from(value)
    }

    async fn subscribe(&self, method: &str) -> Result<Receiver<Value>> {
        let mut request = self.url.as_str().into_client_request()?;

        // Add the authorization bearer token if present
        if self.bearer_token.is_some() {
            let token_string = format!("Bearer {}", self.bearer_token.as_ref().unwrap());
            let header_value = HeaderValue::from_str(token_string.as_str())?;
            request.headers_mut().insert("Authorization", header_value);
        }

        let (mut ws_stream, _) = connect_async(request).await?;
        let request_body = build_jsonrpc_request(method, NO_PARAMS)?;
        ws_stream
            .send(Message::text(request_body.to_string()))
            .await?;

        let (send_chan, recv_chan) = async_channel::unbounded::<Value>();
        spawn(handle_stream(ws_stream, send_chan));

        Ok(recv_chan)
    }
}

/// JsonRpcResponse wraps the json rpc response.
/// We could have encountered success or error, this struct handles the error and result and convert
/// them into Result.
#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    id: u8,
    jsonrpc: String,

    result: Option<T>,
    error: Option<Value>,
}

impl<T> From<JsonRpcResponse<T>> for Result<T> {
    fn from(j: JsonRpcResponse<T>) -> Self {
        if j.error.is_some() {
            Err(anyhow!("json_rpc error: {:}", j.error.unwrap()))
        } else {
            Ok(j.result.unwrap())
        }
    }
}

// Processes a websocket stream by reading messages from the stream `ws_stream` and sending
// them to an output channel `chan`.
async fn handle_stream(
    mut ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    chan: Sender<Value>,
) {
    loop {
        match ws_stream.next().await {
            None => {
                log::trace!("No message in websocket stream. The stream was closed.");
                break;
            }
            Some(result) => match result {
                Ok(msg) => {
                    println!("{}", msg);
                    log::trace!("Read message from websocket stream: {}", msg);
                    let value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
                    chan.send(value).await.unwrap();
                }
                Err(err) => {
                    log::error!("Error reading message from websocket stream: {:?}", err);
                    break;
                }
            },
        };
    }
    chan.close();
}

// A convenience function to build a JSON-RPC request.
fn build_jsonrpc_request(method: &str, params: Value) -> Result<Value> {
    let has_params = if params.is_array() {
        let array_params = params.as_array().unwrap();
        !array_params.is_empty()
    } else if params.is_object() {
        let object_params = params.as_object().unwrap();
        !object_params.is_empty()
    } else if params.is_null() {
        false
    } else {
        return Err(anyhow!("params is not an array nor an object"));
    };

    let request_value = if has_params {
        json!({
            "jsonrpc": DEFAULT_JSON_RPC_VERSION,
            "id": DEFAULT_JSON_RPC_ID,
            "method": method,
            "params": params,
        })
    } else {
        json!({
            "jsonrpc": DEFAULT_JSON_RPC_VERSION,
            "id": DEFAULT_JSON_RPC_ID,
            "method": method,
        })
    };
    Ok(request_value)
}
