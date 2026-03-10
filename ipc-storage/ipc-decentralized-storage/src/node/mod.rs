// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Node module for running a decentralized storage node
//!
//! This module provides functionality to run a complete storage node that:
//! - Starts an Iroh instance for P2P storage
//! - Polls the chain for newly added blobs
//! - Resolves blobs by downloading them from the source nodes

mod resolver;
mod rpc;
pub mod shard_verifier;
pub mod store;

use anyhow::{Context, Result};
use bls_signatures::{PrivateKey as BlsPrivateKey, Serialize as BlsSerialize};
use erasure_encoding::NodeId;
use ethers::types::Address;
use fendermint_actor_blobs_shared::bytes::B256;
use fendermint_rpc::FendermintClient;
use iroh_blobs::Hash;
use iroh_manager::IrohNode;
use std::collections::HashMap;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tendermint_rpc::Url;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::distribution::NodeRpcDirectory;
use crate::gateway::BlobGateway;
use crate::objects::build_node_directories;
use resolver::EventPollerConfig;
use store::InMemoryStore;

/// Default encoding parameters (must match actor defaults).
const DEFAULT_DATA_SHARDS: usize = 4;
const DEFAULT_PARITY_SHARDS: usize = 2;

/// Cached operator directory info for the resolution loop.
struct OperatorDirectoryCache {
    nodes: Vec<NodeId>,
    node_rpc_directory: NodeRpcDirectory,
    last_refresh: Instant,
}

/// Configuration for the storage node
#[derive(Clone)]
pub struct NodeConfig {
    /// Path to store Iroh data
    pub iroh_path: std::path::PathBuf,
    /// IPv4 bind address for Iroh (optional, uses default if None)
    pub iroh_v4_addr: Option<SocketAddrV4>,
    /// IPv6 bind address for Iroh (optional, uses default if None)
    pub iroh_v6_addr: Option<SocketAddrV6>,
    /// Tendermint RPC URL
    pub rpc_url: Url,
    /// Ethereum JSON-RPC URL (Fendermint ETH API endpoint)
    pub eth_rpc_url: String,
    /// Number of blobs to fetch per query
    pub batch_size: u32,
    /// Polling interval for querying added blobs
    pub poll_interval: Duration,
    /// Maximum concurrent blob downloads
    pub max_concurrent_downloads: usize,
    /// BLS private key for signing blob hashes
    pub bls_private_key: BlsPrivateKey,
    /// Address to bind the RPC server for signature queries
    pub rpc_bind_addr: SocketAddr,
    /// Blobs actor address for event filtering
    pub blobs_actor_address: Address,
}

impl std::fmt::Debug for NodeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeConfig")
            .field("iroh_path", &self.iroh_path)
            .field("iroh_v4_addr", &self.iroh_v4_addr)
            .field("iroh_v6_addr", &self.iroh_v6_addr)
            .field("rpc_url", &self.rpc_url)
            .field("eth_rpc_url", &self.eth_rpc_url)
            .field("batch_size", &self.batch_size)
            .field("poll_interval", &self.poll_interval)
            .field("max_concurrent_downloads", &self.max_concurrent_downloads)
            .field("bls_private_key", &"<redacted>")
            .field("rpc_bind_addr", &self.rpc_bind_addr)
            .field("blobs_actor_address", &self.blobs_actor_address)
            .finish()
    }
}

/// Storage for BLS signatures of resolved blobs
/// Maps blob hash -> BLS signature
pub type SignatureStorage = Arc<RwLock<HashMap<Hash, Vec<u8>>>>;

/// Shared Fendermint client wrapped in Arc<Mutex> for async access
pub type SharedFendermintClient = Arc<Mutex<FendermintClient>>;

impl NodeConfig {
    /// Create a new NodeConfig with a generated BLS key
    pub fn new_with_generated_key() -> Self {
        let bls_private_key = BlsPrivateKey::generate(&mut rand::thread_rng());
        Self {
            iroh_path: std::env::current_dir().unwrap().join("iroh_data"),
            iroh_v4_addr: None,
            iroh_v6_addr: None,
            rpc_url: Url::from_str("http://localhost:26657").unwrap(),
            eth_rpc_url: "http://localhost:8545".to_string(),
            batch_size: 10,
            poll_interval: Duration::from_secs(5),
            max_concurrent_downloads: 10,
            bls_private_key,
            rpc_bind_addr: "127.0.0.1:8080".parse().unwrap(),
            blobs_actor_address: Address::zero(), // Should be configured
        }
    }
}

/// Launch a storage node that polls for added blobs and downloads them
///
/// This function:
/// 1. Starts an Iroh node for P2P storage
/// 2. Creates an RPC client to query the chain
/// 3. Polls for newly added blobs
/// 4. Downloads blobs from their source nodes using Iroh
/// 5. Polls for blob finalized/deleted events
pub async fn launch(config: NodeConfig) -> Result<()> {
    info!("Starting decentralized storage node");
    info!("Iroh path: {}", config.iroh_path.display());
    info!("RPC URL: {}", config.rpc_url);
    info!("ETH RPC URL: {}", config.eth_rpc_url);
    info!("Poll interval: {:?}", config.poll_interval);

    // Start Iroh node
    info!("Starting Iroh node...");
    let iroh_node =
        IrohNode::persistent(config.iroh_v4_addr, config.iroh_v6_addr, &config.iroh_path)
            .await
            .context("failed to start Iroh node")?;

    let node_addr = iroh_node.endpoint().node_addr().await?;
    info!("Iroh node started: {}", node_addr.node_id);

    // Create RPC client
    info!("Connecting to Fendermint RPC...");
    let client = FendermintClient::new_http(config.rpc_url.clone(), None)
        .context("failed to create Fendermint client")?;

    // Create gateway
    let gateway = BlobGateway::new(client, config.batch_size, config.poll_interval);

    // Track blobs currently being downloaded (keyed by B256 hash from chain)
    let mut in_progress: HashMap<B256, tokio::task::JoinHandle<Result<()>>> = HashMap::new();
    // Track blobs that have been downloaded but not yet finalized on-chain
    let mut downloaded: HashMap<B256, std::time::Instant> = HashMap::new();

    // Storage for BLS signatures of downloaded blobs
    let signatures: SignatureStorage = Arc::new(RwLock::new(HashMap::new()));

    // Create in-memory store for tracking polled heights
    let store = Arc::new(InMemoryStore::new());

    // Create a separate client for RPC server queries
    let rpc_client = FendermintClient::new_http(config.rpc_url.clone(), None)
        .context("failed to create RPC server Fendermint client")?;
    let rpc_client = Arc::new(Mutex::new(rpc_client));

    // Start RPC server for signature queries, blob downloads, and shard pulls
    let signatures_for_rpc = signatures.clone();
    let rpc_bind_addr = config.rpc_bind_addr;
    let rpc_client_for_server = rpc_client.clone();
    let iroh_for_rpc = iroh_node.clone();
    let bls_key_for_rpc = config.bls_private_key;
    let rpc_url_for_server = config.rpc_url.clone();
    tokio::spawn(async move {
        if let Err(e) = rpc::start_rpc_server(
            rpc_bind_addr,
            signatures_for_rpc,
            rpc_client_for_server,
            iroh_for_rpc,
            bls_key_for_rpc,
            rpc_url_for_server,
        )
        .await
        {
            error!("RPC server error: {}", e);
        }
    });

    // Start event poller for blob finalization and deletion
    let signatures_for_events = signatures.clone();
    let store_for_events = store.clone();
    let iroh_for_events = iroh_node.clone();
    let event_poller_config = EventPollerConfig {
        eth_rpc_url: config.eth_rpc_url.clone(),
        poll_interval: config.poll_interval,
        blobs_actor_address: config.blobs_actor_address,
    };
    tokio::spawn(async move {
        if let Err(e) = resolver::poll_for_blob_events(
            event_poller_config,
            signatures_for_events,
            store_for_events,
            iroh_for_events,
        )
        .await
        {
            error!("Event poller error: {}", e);
        }
    });

    // Determine this node's NodeId from its Iroh identity
    let our_node_id = NodeId(node_addr.node_id.as_bytes().clone());
    info!("Our NodeId: {:?}", hex::encode(our_node_id.0));

    // Operator directory cache (refreshed periodically)
    let mut op_cache: Option<OperatorDirectoryCache> = None;
    let cache_refresh_interval = Duration::from_secs(300);

    info!("Starting blob resolution loop");
    info!(
        "BLS public key: {:?}",
        hex::encode(config.bls_private_key.public_key().as_bytes())
    );
    info!("RPC server listening on: {}", config.rpc_bind_addr);

    loop {
        // Check completed downloads and move them to the downloaded set
        let mut finished = Vec::new();
        in_progress.retain(|hash, handle| {
            if handle.is_finished() {
                finished.push(*hash);
                false
            } else {
                true
            }
        });

        for hash in finished {
            info!(
                "Blob {} resolution completed, waiting for finalization",
                hash
            );
            downloaded.insert(hash, Instant::now());
        }

        // Clean up old downloaded entries
        if !downloaded.is_empty() {
            debug!("Blobs waiting for finalization: {}", downloaded.len());
            let cutoff = Instant::now() - Duration::from_secs(300);
            downloaded.retain(|hash, timestamp| {
                if *timestamp < cutoff {
                    warn!("Blob {} has been waiting for finalization for >5 minutes, removing from tracking", hash);
                    false
                } else {
                    true
                }
            });
        }

        // Refresh operator directory cache if stale or missing
        let cache_stale = op_cache
            .as_ref()
            .map_or(true, |c| c.last_refresh.elapsed() > cache_refresh_interval);

        if cache_stale {
            match build_node_directories(&gateway).await {
                Ok((nodes, _node_directory, node_rpc_directory)) => {
                    info!("Refreshed operator directory: {} nodes", nodes.len());
                    op_cache = Some(OperatorDirectoryCache {
                        nodes,
                        node_rpc_directory,
                        last_refresh: Instant::now(),
                    });
                }
                Err(e) => {
                    warn!("Failed to refresh operator directory: {}", e);
                }
            }
        }

        // Query for added blobs
        match gateway.query_added_blobs().await {
            Ok(blobs) => {
                if !blobs.is_empty() {
                    info!("Found {} added blobs to resolve", blobs.len());

                    for blob_item in blobs {
                        let (hash, size, _sources) = blob_item;

                        // Skip if already in progress or downloaded
                        if in_progress.contains_key(&hash) {
                            debug!("Blob {} already in progress, skipping", hash);
                            continue;
                        }
                        if in_progress.len() >= config.max_concurrent_downloads {
                            warn!(
                                "Max concurrent downloads ({}) reached, deferring blob {}",
                                config.max_concurrent_downloads, hash
                            );
                            continue;
                        }
                        if downloaded.contains_key(&hash) {
                            debug!("Blob {} already downloaded, waiting for finalization", hash);
                            continue;
                        }

                        // Need operator directory to resolve
                        let Some(cache) = &op_cache else {
                            warn!("No operator directory available, deferring blob {}", hash);
                            continue;
                        };

                        if cache.nodes.is_empty() {
                            warn!("No nodes in operator directory, deferring blob {}", hash);
                            continue;
                        }

                        // Spawn shard-based resolution
                        let iroh_clone = iroh_node.clone();
                        let nodes_clone = cache.nodes.clone();
                        let rpc_dir_clone = cache.node_rpc_directory.clone();
                        let our_id = our_node_id;
                        let bls_key = config.bls_private_key;
                        let sigs = signatures.clone();

                        info!(
                            "Spawning shard resolution for blob {} (size: {})",
                            hash, size
                        );

                        let handle = tokio::spawn(async move {
                            resolver::resolve_blob_shards(
                                iroh_clone,
                                hash,
                                size,
                                DEFAULT_DATA_SHARDS,
                                DEFAULT_PARITY_SHARDS,
                                nodes_clone,
                                rpc_dir_clone,
                                our_id,
                                bls_key,
                                sigs,
                            )
                            .await
                        });

                        in_progress.insert(hash, handle);
                    }
                }
            }
            Err(e) => {
                error!("Failed to query added blobs: {}", e);
            }
        }

        // Wait before the next poll
        sleep(config.poll_interval).await;
    }
}
