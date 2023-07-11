// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::anyhow;
use axum::routing::{get, post};
use jsonrpc_v2::Data;
use std::{net::ToSocketAddrs, sync::Arc, time::Duration};
use tendermint_rpc::WebSocketClient;

mod apis;
mod conv;
mod error;
mod filters;
mod gas;
mod handlers;
mod state;

use error::{error, JsonRpcError};
use state::JsonRpcState;

type JsonRpcData<C> = Data<JsonRpcState<C>>;
type JsonRpcServer = Arc<jsonrpc_v2::Server<jsonrpc_v2::MapRouter>>;
type JsonRpcResult<T> = Result<T, JsonRpcError>;

/// Start listening to JSON-RPC requests.
pub async fn listen<A: ToSocketAddrs>(
    listen_addr: A,
    client: WebSocketClient,
    filter_timeout: Duration,
) -> anyhow::Result<()> {
    if let Some(listen_addr) = listen_addr.to_socket_addrs()?.next() {
        let state = JsonRpcState::new(client, filter_timeout);
        let server = make_server(state);
        let router = make_router(server);
        let server = axum::Server::try_bind(&listen_addr)?.serve(router.into_make_service());

        tracing::info!(?listen_addr, "bound Ethereum API");
        server.await?;
        Ok(())
    } else {
        Err(anyhow!("failed to convert to any socket address"))
    }
}

/// Register method handlers with the JSON-RPC server construct.
fn make_server(state: JsonRpcState<WebSocketClient>) -> JsonRpcServer {
    let server = jsonrpc_v2::Server::new().with_data(Data(Arc::new(state)));
    let server = apis::register_methods(server);
    server.finish()
}

/// Register routes in the `axum` router to handle JSON-RPC and WebSocket calls.
fn make_router(server: JsonRpcServer) -> axum::Router {
    axum::Router::new()
        .route("/", post(handlers::http::handle))
        .route("/", get(handlers::ws::handle))
        .with_state(server)
}
