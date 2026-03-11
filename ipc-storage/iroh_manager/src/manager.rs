// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::Path;

use anyhow::Result;
use iroh_blobs::rpc::proto::RpcService;
use n0_future::task::AbortOnDropHandle;
use quic_rpc::client::QuinnConnector;
use tracing::info;

use crate::{BlobsClient, IrohNode};

#[derive(Debug)]
pub struct IrohManager {
    client: IrohNode,
    server_key: Vec<u8>,
    rpc_addr: SocketAddr,
    _rpc_task: AbortOnDropHandle<()>,
}

impl IrohManager {
    pub async fn new(
        v4_addr: Option<SocketAddrV4>,
        v6_addr: Option<SocketAddrV6>,
        path: impl AsRef<Path>,
        rpc_addr: Option<SocketAddr>,
    ) -> Result<Self> {
        let storage_path = path.as_ref().to_path_buf();
        let client = IrohNode::persistent(v4_addr, v6_addr, &storage_path).await?;

        // setup an RPC listener
        let rpc_addr = rpc_addr.unwrap_or_else(|| "127.0.0.1:0".parse().unwrap());

        let (config, server_key) = quic_rpc::transport::quinn::configure_server()?;
        let endpoint = iroh_quinn::Endpoint::server(config, rpc_addr)?;
        let local_addr = endpoint.local_addr()?;

        info!("Iroh RPC listening on {} ({})", local_addr, rpc_addr);
        let rpc_server = quic_rpc::transport::quinn::QuinnListener::new(endpoint)?;
        let rpc_server = quic_rpc::RpcServer::<RpcService, _>::new(rpc_server);
        let blobs = client.blobs.clone();
        let rpc_task = rpc_server
            .spawn_accept_loop(move |msg, chan| blobs.clone().handle_rpc_request(msg, chan));

        Ok(Self {
            client,
            server_key,
            rpc_addr: local_addr,
            _rpc_task: rpc_task,
        })
    }

    /// Retrives a blob client, and starts the node if it has not started yet.
    pub fn blobs_client(&self) -> BlobsClient {
        self.client.blobs_client().boxed()
    }

    /// Returns the key for the RPC client.
    pub fn rpc_key(&self) -> &[u8] {
        &self.server_key
    }

    pub fn rpc_addr(&self) -> SocketAddr {
        self.rpc_addr
    }
}

pub type BlobsRpcClient = iroh_blobs::rpc::client::blobs::Client<QuinnConnector<RpcService>>;

/// Connect to the given rpc listening on this address, with this key.
pub async fn connect(remote_addr: SocketAddr) -> Result<BlobsClient> {
    info!("iroh RPC connecting to {}", remote_addr);
    let bind_addr: SocketAddr = "0.0.0.0:0".parse()?;
    let client = quic_rpc::transport::quinn::make_insecure_client_endpoint(bind_addr)?;
    let client = QuinnConnector::<RpcService>::new(client, remote_addr, "localhost".to_string());
    let client = quic_rpc::RpcClient::<RpcService, _>::new(client);
    let client = iroh_blobs::rpc::client::blobs::Client::new(client);
    Ok(client.boxed())
}

#[cfg(test)]
mod tests {
    use n0_future::StreamExt;

    use super::*;

    #[tokio::test]
    async fn test_append_delete() -> Result<()> {
        tracing_subscriber::fmt().init();
        let dir = tempfile::tempdir()?;

        let iroh = IrohManager::new(None, None, dir.path(), None).await?;

        let tags: Vec<_> = (0..10).map(|i| format!("tag-{i}")).collect();

        for tag in &tags {
            iroh.blobs_client()
                .add_bytes_named(format!("content-for-{tag}"), tag.as_bytes())
                .await?;
        }

        let existing_tags: Vec<_> = iroh
            .blobs_client()
            .tags()
            .list()
            .await?
            .try_collect()
            .await?;
        assert_eq!(existing_tags.len(), 10);

        let t = tags.clone();
        let rpc_addr = iroh.rpc_addr();
        let task = tokio::task::spawn(async move {
            let client = connect(rpc_addr).await?;

            for tag in t {
                client.tags().delete(tag).await?;
            }

            anyhow::Ok(())
        });

        task.await??;

        let existing_tags: Vec<_> = iroh
            .blobs_client()
            .tags()
            .list()
            .await?
            .try_collect()
            .await?;
        dbg!(&existing_tags);
        assert_eq!(existing_tags.len(), 0);

        Ok(())
    }
}
