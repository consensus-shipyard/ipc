// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Service resources for storage-node plugin.
//!
//! This module defines the resources that the storage plugin exposes
//! to other components through the ModuleResources API.

use crate::resolver::ResolvePool;
use crate::storage_env::{BlobPoolItem, ReadRequestPoolItem};
use std::sync::Arc;

/// Resources provided by the storage-node plugin.
///
/// These can be accessed by other components through the generic
/// ModuleResources API without hardcoding storage-specific types.
#[derive(Clone)]
pub struct StorageServiceResources {
    /// Pool for managing blob resolution requests
    pub blob_pool: Arc<ResolvePool<BlobPoolItem>>,

    /// Pool for managing read request resolution
    pub read_request_pool: Arc<ResolvePool<ReadRequestPoolItem>>,
}

impl StorageServiceResources {
    pub fn new(
        blob_pool: Arc<ResolvePool<BlobPoolItem>>,
        read_request_pool: Arc<ResolvePool<ReadRequestPoolItem>>,
    ) -> Self {
        Self {
            blob_pool,
            read_request_pool,
        }
    }
}

/// Settings structure that the plugin expects in ServiceContext.
///
/// The app layer should populate ServiceContext with these settings.
#[derive(Clone)]
pub struct StorageServiceSettings {
    /// Whether the storage services are enabled
    pub enabled: bool,

    /// Retry delay for failed resolutions (in seconds)
    pub retry_delay: u64,

    /// IPC subnet ID
    pub subnet_id: ipc_api::subnet_id::SubnetID,

    /// Vote interval (in seconds)
    pub vote_interval: std::time::Duration,

    /// Vote timeout (in seconds)
    pub vote_timeout: std::time::Duration,
}

/// Extra context data that the plugin needs from the app.
///
/// This should be provided via ServiceContext.with_extra()
pub struct StorageServiceContext {
    /// IPLD resolver client for network communication
    pub resolver_client: ipc_ipld_resolver::Client<fendermint_vm_topdown::voting::VoteTally>,

    /// Vote tally for parent finality
    pub vote_tally: fendermint_vm_topdown::voting::VoteTally,
}
