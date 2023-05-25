// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::anyhow;
use std::net::ToSocketAddrs;

/// Start listening to JSON-RPC requests.
pub async fn listen<A: ToSocketAddrs>(listen_addr: A) -> anyhow::Result<()> {
    if let Some(listen_addr) = listen_addr.to_socket_addrs()?.next() {
        let router = axum::Router::new();

        let server = axum::Server::try_bind(&listen_addr)?.serve(router.into_make_service());

        tracing::info!(?listen_addr, "bound Ethereum API");
        server.await?;
        Ok(())
    } else {
        Err(anyhow!("failed to convert to any socket address"))
    }
}
