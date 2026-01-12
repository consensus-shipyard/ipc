// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Objects service integration for the gateway
//!
//! This module provides functionality to start the objects HTTP service
//! alongside the gateway's blob polling functionality.

use anyhow::Result;
use iroh_manager::{BlobsClient, IrohNode};
use std::net::SocketAddr;
use tracing::info;

use crate::objects::{self, ObjectsConfig};

/// Configuration for the gateway with objects service
#[derive(Clone, Debug, Default)]
pub struct GatewayWithObjectsConfig {
    /// Objects service configuration
    pub objects_config: ObjectsConfig,
}

/// Start the objects HTTP service in a background task
///
/// This spawns the objects service which handles:
/// - POST /v1/objects - Upload objects
/// - GET /v1/objects/{address}/{key} - Download objects from buckets
/// - GET /v1/blobs/{hash} - Download blobs directly
///
/// Returns a handle to the spawned task.
pub fn start_objects_service(
    config: ObjectsConfig,
    iroh_node: IrohNode,
    iroh_resolver_blobs: BlobsClient,
) -> tokio::task::JoinHandle<Result<()>> {
    let listen_addr = config.listen_addr;
    info!(listen_addr = %listen_addr, "starting objects service in background");

    tokio::spawn(async move {
        objects::run_objects_service(config, iroh_node, iroh_resolver_blobs).await
    })
}

/// Start only the objects HTTP service (blocking)
///
/// This is a convenience function that runs the objects service directly
/// without the gateway's blob polling functionality.
pub async fn run_objects_service_standalone(
    listen_addr: SocketAddr,
    tendermint_url: tendermint_rpc::Url,
    iroh_node: IrohNode,
    iroh_resolver_blobs: BlobsClient,
    max_object_size: u64,
) -> Result<()> {
    let config = ObjectsConfig {
        listen_addr,
        tendermint_url,
        max_object_size,
        metrics_enabled: false,
        metrics_listen: None,
    };

    objects::run_objects_service(config, iroh_node, iroh_resolver_blobs).await
}
