// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Node module for running a decentralized storage node
//!
//! This module provides functionality to run a complete storage node that:
//! - Starts an Iroh instance for P2P storage
//! - Polls the chain for newly added blobs
//! - Resolves blobs by downloading them from the source nodes

use anyhow::{Context, Result};
use bls_signatures::{PrivateKey as BlsPrivateKey, Serialize as BlsSerialize};
use fendermint_actor_storage_blobs_shared::bytes::B256;
use fendermint_rpc::message::GasParams;
use fendermint_rpc::{FendermintClient, QueryClient};
use fendermint_vm_message::query::FvmQueryHeight;
use futures::StreamExt;
use fvm_shared::econ::TokenAmount;
use iroh_blobs::Hash;
use storage_node_iroh_manager::IrohNode;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tendermint_rpc::query::EventType;
use tendermint_rpc::{SubscriptionClient, Url, WebSocketClient};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use warp::Filter;

use crate::gateway::BlobGateway;

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
}

impl std::fmt::Debug for NodeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeConfig")
            .field("iroh_path", &self.iroh_path)
            .field("iroh_v4_addr", &self.iroh_v4_addr)
            .field("iroh_v6_addr", &self.iroh_v6_addr)
            .field("rpc_url", &self.rpc_url)
            .field("batch_size", &self.batch_size)
            .field("poll_interval", &self.poll_interval)
            .field("max_concurrent_downloads", &self.max_concurrent_downloads)
            .field("bls_private_key", &"<redacted>")
            .field("rpc_bind_addr", &self.rpc_bind_addr)
            .finish()
    }
}

/// Storage for BLS signatures of resolved blobs
/// Maps blob hash -> BLS signature
pub type SignatureStorage = Arc<RwLock<HashMap<Hash, Vec<u8>>>>;

impl NodeConfig {
    /// Create a new NodeConfig with a generated BLS key
    pub fn new_with_generated_key() -> Self {
        let bls_private_key = BlsPrivateKey::generate(&mut rand::thread_rng());
        Self {
            iroh_path: std::env::current_dir().unwrap().join("iroh_data"),
            iroh_v4_addr: None,
            iroh_v6_addr: None,
            rpc_url: Url::from_str("http://localhost:26657").unwrap(),
            batch_size: 10,
            poll_interval: Duration::from_secs(5),
            max_concurrent_downloads: 10,
            bls_private_key,
            rpc_bind_addr: "127.0.0.1:8080".parse().unwrap(),
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
pub async fn launch(config: NodeConfig) -> Result<()> {
    info!("Starting decentralized storage node");
    info!("Iroh path: {}", config.iroh_path.display());
    info!("RPC URL: {}", config.rpc_url);
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

    // Track blobs currently being downloaded
    let mut in_progress: HashMap<Hash, tokio::task::JoinHandle<Result<()>>> = HashMap::new();
    // Track blobs that have been downloaded but not yet finalized on-chain
    let mut downloaded: HashMap<Hash, std::time::Instant> = HashMap::new();

    // Storage for BLS signatures of downloaded blobs
    let signatures: SignatureStorage = Arc::new(RwLock::new(HashMap::new()));

    // Create a separate client for RPC server queries
    let rpc_client = FendermintClient::new_http(config.rpc_url.clone(), None)
        .context("failed to create RPC server Fendermint client")?;
    let rpc_client = Arc::new(Mutex::new(rpc_client));

    // Start RPC server for signature queries and blob downloads
    let signatures_for_rpc = signatures.clone();
    let rpc_bind_addr = config.rpc_bind_addr;
    let rpc_client_for_server = rpc_client.clone();
    let iroh_for_rpc = iroh_node.clone();
    tokio::spawn(async move {
        if let Err(e) = start_rpc_server(rpc_bind_addr, signatures_for_rpc, rpc_client_for_server, iroh_for_rpc).await {
            error!("RPC server error: {}", e);
        }
    });

    // Start event listener for blob finalization
    let signatures_for_events = signatures.clone();
    let event_url = config.rpc_url.clone();
    tokio::spawn(async move {
        if let Err(e) = listen_for_finalized_events(event_url, signatures_for_events).await {
            error!("Event listener error: {}", e);
        }
    });

    info!("Starting blob resolution loop");
    info!(
        "BLS public key: {:?}",
        hex::encode(config.bls_private_key.public_key().as_bytes())
    );
    info!("RPC server listening on: {}", config.rpc_bind_addr);

    loop {
        // Check completed downloads and move them to the downloaded set
        // Collect finished tasks to process
        let mut finished = Vec::new();
        in_progress.retain(|hash, handle| {
            if handle.is_finished() {
                finished.push(*hash);
                false // Remove from in_progress
            } else {
                true // Keep in in_progress
            }
        });

        // Process finished downloads
        for hash in finished {
            // Note: The task has finished, but we mark it as downloaded
            // The actual result checking would require more complex handling
            // For now, we assume successful completion if the task finished
            info!("Blob {} download completed, waiting for finalization", hash);
            downloaded.insert(hash, std::time::Instant::now());
        }

        // TODO: Query on-chain blob status to check if downloaded blobs are finalized
        // For now, just log the downloaded blobs waiting for finalization
        if !downloaded.is_empty() {
            debug!("Blobs waiting for finalization: {}", downloaded.len());
            // Clean up old entries (older than 5 minutes) to prevent memory leaks
            let cutoff = std::time::Instant::now() - Duration::from_secs(300);
            downloaded.retain(|hash, timestamp| {
                if *timestamp < cutoff {
                    warn!("Blob {} has been waiting for finalization for >5 minutes, removing from tracking", hash);
                    false
                } else {
                    true
                }
            });
        }

        // Query for added blobs
        match gateway.query_added_blobs().await {
            Ok(blobs) => {
                if !blobs.is_empty() {
                    info!("Found {} added blobs to resolve", blobs.len());

                    for blob_item in blobs {
                        let (hash, size, sources) = blob_item;

                        // Skip if already downloading
                        if in_progress.contains_key(&hash) {
                            debug!("Blob {} already in progress, skipping", hash);
                            continue;
                        }

                        // Check if we're at the concurrency limit
                        if in_progress.len() >= config.max_concurrent_downloads {
                            warn!(
                                "Max concurrent downloads ({}) reached, deferring blob {}",
                                config.max_concurrent_downloads, hash
                            );
                            continue;
                        }

                        // Skip if already downloaded and waiting for finalization
                        if downloaded.contains_key(&hash) {
                            debug!("Blob {} already downloaded, waiting for finalization", hash);
                            continue;
                        }

                        // Spawn a task to download this blob
                        let iroh_clone = iroh_node.clone();
                        let bls_key = config.bls_private_key;
                        let sigs = signatures.clone();
                        let handle = tokio::spawn(async move {
                            resolve_blob(iroh_clone, hash, size, sources, bls_key, sigs).await
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

/// Resolve a blob by downloading it from one of its sources
///
/// Downloads the hash sequence and all blobs referenced within it (including original content).
/// Returns Ok(()) if the blob was successfully downloaded, Err otherwise.
async fn resolve_blob(
    iroh: IrohNode,
    hash: Hash,
    size: u64,
    sources: std::collections::HashSet<(
        fvm_shared::address::Address,
        fendermint_actor_storage_blobs_shared::blobs::SubscriptionId,
        iroh::NodeId,
    )>,
    bls_private_key: BlsPrivateKey,
    signatures: SignatureStorage,
) -> Result<()> {
    use iroh_blobs::hashseq::HashSeq;

    info!("Resolving blob: {} (size: {})", hash, size);
    debug!("Sources: {} available", sources.len());

    // Try each source until one succeeds
    for (_subscriber, _id, source_node_id) in sources {
        debug!("Attempting download from source: {}", source_node_id);

        // Create a NodeAddr from the source
        let source_addr = iroh::NodeAddr::new(source_node_id);

        // Step 1: Download the hash sequence blob
        match iroh
            .blobs_client()
            .download_with_opts(
                hash,
                iroh_blobs::rpc::client::blobs::DownloadOptions {
                    format: iroh_blobs::BlobFormat::Raw,
                    nodes: vec![source_addr.clone()],
                    tag: iroh_blobs::util::SetTagOption::Named(iroh_blobs::Tag(
                        format!("blob-seq-{}", hash).into(),
                    )),
                    mode: iroh_blobs::rpc::client::blobs::DownloadMode::Queued,
                },
            )
            .await
        {
            Ok(progress) => {
                match progress.finish().await {
                    Ok(outcome) => {
                        let downloaded_size = outcome.local_size + outcome.downloaded_size;
                        info!(
                            "Downloaded hash sequence {} (downloaded: {} bytes, local: {} bytes)",
                            hash, outcome.downloaded_size, outcome.local_size
                        );

                        // Step 2: Read and parse the hash sequence to get all referenced blobs
                        let hash_seq_bytes = match iroh.blobs_client().read_to_bytes(hash).await {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                warn!("Failed to read hash sequence {}: {}", hash, e);
                                continue;
                            }
                        };

                        let hash_seq = match HashSeq::try_from(hash_seq_bytes) {
                            Ok(seq) => seq,
                            Err(e) => {
                                warn!("Failed to parse hash sequence {}: {}", hash, e);
                                continue;
                            }
                        };

                        let content_hashes: Vec<Hash> = hash_seq.iter().collect();
                        info!(
                            "Hash sequence {} contains {} blobs to download",
                            hash,
                            content_hashes.len()
                        );

                        // Step 3: Download all blobs in the hash sequence
                        let mut all_downloaded = true;
                        for (idx, content_hash) in content_hashes.iter().enumerate() {
                            let blob_type = if idx == 0 {
                                "original content"
                            } else if idx == 1 {
                                "metadata"
                            } else {
                                "parity"
                            };

                            debug!(
                                "Downloading {} blob {} ({}/{}): {}",
                                blob_type,
                                content_hash,
                                idx + 1,
                                content_hashes.len(),
                                content_hash
                            );

                            match iroh
                                .blobs_client()
                                .download_with_opts(
                                    *content_hash,
                                    iroh_blobs::rpc::client::blobs::DownloadOptions {
                                        format: iroh_blobs::BlobFormat::Raw,
                                        nodes: vec![source_addr.clone()],
                                        tag: iroh_blobs::util::SetTagOption::Named(iroh_blobs::Tag(
                                            format!("blob-{}-{}", hash, content_hash).into(),
                                        )),
                                        mode: iroh_blobs::rpc::client::blobs::DownloadMode::Queued,
                                    },
                                )
                                .await
                            {
                                Ok(content_progress) => {
                                    match content_progress.finish().await {
                                        Ok(content_outcome) => {
                                            debug!(
                                                "Downloaded {} blob {} (downloaded: {} bytes, local: {} bytes)",
                                                blob_type,
                                                content_hash,
                                                content_outcome.downloaded_size,
                                                content_outcome.local_size
                                            );
                                        }
                                        Err(e) => {
                                            warn!(
                                                "Failed to complete {} blob {} download: {}",
                                                blob_type, content_hash, e
                                            );
                                            all_downloaded = false;
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to start {} blob {} download: {}",
                                        blob_type, content_hash, e
                                    );
                                    all_downloaded = false;
                                }
                            }
                        }

                        if !all_downloaded {
                            warn!(
                                "Not all content blobs downloaded for {}, trying next source",
                                hash
                            );
                            continue;
                        }

                        info!(
                            "Successfully resolved blob {} with all {} content blobs (expected original size: {} bytes)",
                            hash, content_hashes.len(), size
                        );

                        // Generate BLS signature for the blob hash
                        let hash_bytes = hash.as_bytes();
                        let signature = bls_private_key.sign(hash_bytes);
                        let signature_bytes = signature.as_bytes();

                        // Store signature in memory
                        {
                            let mut sigs = signatures.write().unwrap();
                            sigs.insert(hash, signature_bytes.clone());
                        }

                        info!("Generated BLS signature for blob {}", hash);
                        debug!("Signature: {}", hex::encode(&signature_bytes));
                        debug!(
                            "Hash sequence blob size: {} bytes",
                            downloaded_size
                        );

                        // Blob downloaded successfully
                        // It will now wait for validator signatures before finalization
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("Failed to complete download from {}: {}", source_node_id, e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to start download from {}: {}", source_node_id, e);
            }
        }
    }

    anyhow::bail!("Failed to resolve blob {} from any source", hash)
}

/// Listen for BlobFinalized events and clean up signatures from memory
async fn listen_for_finalized_events(rpc_url: Url, signatures: SignatureStorage) -> Result<()> {
    info!("Starting event listener for BlobFinalized events");

    // Convert HTTP URL to WebSocket URL
    let ws_url = rpc_url
        .to_string()
        .replace("http://", "ws://")
        .replace("https://", "wss://");
    let ws_url = format!("{}/websocket", ws_url.trim_end_matches('/'));

    info!("Connecting to WebSocket: {}", ws_url);

    // Connect to WebSocket client
    let (client, driver) = WebSocketClient::new(ws_url.as_str())
        .await
        .context("failed to create WebSocket client")?;

    // Spawn the driver in the background
    tokio::spawn(async move {
        if let Err(e) = driver.run().await {
            error!("WebSocket driver error: {}", e);
        }
    });

    // Subscribe to all transactions (we'll filter for BlobFinalized events)
    let mut subscription = client
        .subscribe(EventType::Tx.into())
        .await
        .context("failed to subscribe to events")?;

    info!("Subscribed to transaction events, listening for BlobFinalized...");

    // Process events as they arrive
    while let Some(result) = subscription.next().await {
        match result {
            Ok(event) => {
                // Parse the event to extract BlobFinalized information
                if let Err(e) = process_event(&event, &signatures) {
                    debug!("Error processing event: {}", e);
                }
            }
            Err(e) => {
                warn!("Error receiving event: {}", e);
            }
        }
    }

    warn!("Event subscription ended");
    Ok(())
}

/// Process a Tendermint event and clean up signatures if it's a BlobFinalized event
fn process_event(
    event: &tendermint_rpc::event::Event,
    signatures: &SignatureStorage,
) -> Result<()> {
    // Look for BlobFinalized event in the transaction result
    if let tendermint_rpc::event::EventData::Tx { tx_result } = &event.data {
        // Search through events for BlobFinalized
        for tendermint_event in &tx_result.result.events {
            if tendermint_event.kind == "BlobFinalized" {
                // Extract the hash from event attributes
                for attr in &tendermint_event.attributes {
                    if attr.key == "hash" {
                        // The hash is in hex format (bytes32), we need to convert to Hash
                        let hash_hex = attr.value.trim_start_matches("0x");

                        match hex::decode(hash_hex) {
                            Ok(hash_bytes) if hash_bytes.len() == 32 => {
                                // Convert [u8; 32] to iroh Hash
                                let hash_array: [u8; 32] = hash_bytes.try_into().unwrap();
                                let hash = Hash::from(hash_array);

                                // Remove signature from memory
                                let mut sigs = signatures.write().unwrap();
                                if sigs.remove(&hash).is_some() {
                                    info!(
                                        "Removed signature for finalized blob {} from memory",
                                        hash
                                    );
                                } else {
                                    debug!(
                                        "Blob {} was finalized but no signature found in memory",
                                        hash
                                    );
                                }
                            }
                            Ok(_) => {
                                debug!("Invalid hash length in BlobFinalized event");
                            }
                            Err(e) => {
                                debug!("Failed to decode hash from event: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Shared Fendermint client wrapped in Arc<Mutex> for async access
pub type SharedFendermintClient = Arc<Mutex<FendermintClient>>;

/// Start the RPC server for signature queries and blob queries
async fn start_rpc_server(
    bind_addr: SocketAddr,
    signatures: SignatureStorage,
    client: SharedFendermintClient,
    iroh: IrohNode,
) -> Result<()> {
    // GET /signature/{hash}
    let get_signature = warp::path!("signature" / String)
        .and(warp::get())
        .and(with_signatures(signatures))
        .and_then(handle_get_signature);

    // GET /health
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({"status": "ok"})));

    // GET /v1/blobs/{hash} - returns blob metadata as JSON
    let client_for_meta = client.clone();
    let get_blob = warp::path!("v1" / "blobs" / String)
        .and(warp::get())
        .and(warp::query::<HeightQuery>())
        .and(with_client(client_for_meta))
        .and_then(handle_get_blob);

    // GET /v1/blobs/{hash}/content - returns blob content as binary stream
    let get_blob_content = warp::path!("v1" / "blobs" / String / "content")
        .and(warp::get())
        .and(warp::query::<HeightQuery>())
        .and(with_client(client))
        .and(with_iroh(iroh))
        .and_then(handle_get_blob_content);

    let routes = get_signature.or(health).or(get_blob_content).or(get_blob);

    info!("RPC server starting on {}", bind_addr);
    warp::serve(routes).run(bind_addr).await;
    Ok(())
}

/// Warp filter to inject signature storage
fn with_signatures(
    signatures: SignatureStorage,
) -> impl Filter<Extract = (SignatureStorage,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || signatures.clone())
}

/// Response for signature query
#[derive(serde::Serialize)]
struct SignatureResponse {
    hash: String,
    signature: String,
}

/// Handle GET /signature/{hash}
async fn handle_get_signature(
    hash_str: String,
    signatures: SignatureStorage,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Parse hash from hex string
    let hash = Hash::from_str(&hash_str).map_err(|_| warp::reject::not_found())?;

    // Look up signature
    let signature = {
        let sigs = signatures.read().unwrap();
        sigs.get(&hash).cloned()
    };

    match signature {
        Some(sig) => {
            let response = SignatureResponse {
                hash: hash_str,
                signature: hex::encode(&sig),
            };
            Ok(warp::reply::json(&response))
        }
        None => Err(warp::reject::not_found()),
    }
}

/// Query parameter for optional block height
#[derive(serde::Deserialize)]
struct HeightQuery {
    pub height: Option<u64>,
}

/// Warp filter to inject Fendermint client
fn with_client(
    client: SharedFendermintClient,
) -> impl Filter<Extract = (SharedFendermintClient,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

/// Response for blob query
#[derive(serde::Serialize)]
struct BlobResponse {
    hash: String,
    size: u64,
    metadata_hash: String,
    status: String,
    subscribers: Vec<BlobSubscriberInfo>,
}

/// Subscriber info for blob response
#[derive(serde::Serialize)]
struct BlobSubscriberInfo {
    subscription_id: String,
    expiry: i64,
}

/// Error response
#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
}

/// Handle GET /v1/blobs/{hash}
async fn handle_get_blob(
    hash_str: String,
    height_query: HeightQuery,
    client: SharedFendermintClient,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Parse blob hash - strip 0x prefix if present
    let blob_hash_hex = hash_str.strip_prefix("0x").unwrap_or(&hash_str);

    let blob_hash_bytes = match hex::decode(blob_hash_hex) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ErrorResponse {
                    error: "invalid hex string".to_string(),
                }),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    if blob_hash_bytes.len() != 32 {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse {
                error: format!("blob hash must be 32 bytes, got {}", blob_hash_bytes.len()),
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&blob_hash_bytes);
    let blob_hash = B256(hash_array);

    // Set query height
    let height = height_query
        .height
        .map(FvmQueryHeight::from)
        .unwrap_or(FvmQueryHeight::Committed);

    // Gas params for the query call
    let gas_params = GasParams {
        gas_limit: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    // Query the blob
    let maybe_blob = {
        let mut client_guard = client.lock().await;
        client_guard
            .blob_get_call(blob_hash, TokenAmount::default(), gas_params, height)
            .await
    };

    match maybe_blob {
        Ok(Some(blob)) => {
            let subscribers: Vec<BlobSubscriberInfo> = blob
                .subscribers
                .iter()
                .map(|(sub_id, expiry)| BlobSubscriberInfo {
                    subscription_id: sub_id.to_string(),
                    expiry: *expiry,
                })
                .collect();

            let response = BlobResponse {
                hash: format!("0x{}", hex::encode(blob_hash.0)),
                size: blob.size,
                metadata_hash: format!("0x{}", hex::encode(blob.metadata_hash.0)),
                status: format!("{:?}", blob.status),
                subscribers,
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::OK,
            ))
        }
        Ok(None) => Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse {
                error: "blob not found".to_string(),
            }),
            warp::http::StatusCode::NOT_FOUND,
        )),
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse {
                error: format!("query failed: {}", e),
            }),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// Warp filter to inject Iroh node
fn with_iroh(
    iroh: IrohNode,
) -> impl Filter<Extract = (IrohNode,), Error = Infallible> + Clone {
    warp::any().map(move || iroh.clone())
}

/// Handle GET /v1/blobs/{hash}/content - returns the actual blob content
async fn handle_get_blob_content(
    hash_str: String,
    height_query: HeightQuery,
    client: SharedFendermintClient,
    iroh: IrohNode,
) -> Result<impl warp::Reply, warp::Rejection> {
    use futures::TryStreamExt;
    use iroh_blobs::hashseq::HashSeq;
    use warp::hyper::Body;

    // Parse blob hash - strip 0x prefix if present
    let blob_hash_hex = hash_str.strip_prefix("0x").unwrap_or(&hash_str);

    let blob_hash_bytes = match hex::decode(blob_hash_hex) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::Response::new(Body::from(
                    serde_json::to_string(&ErrorResponse {
                        error: "invalid hex string".to_string(),
                    })
                    .unwrap(),
                )),
                warp::http::StatusCode::BAD_REQUEST,
            ));
        }
    };

    if blob_hash_bytes.len() != 32 {
        return Ok(warp::reply::with_status(
            warp::reply::Response::new(Body::from(
                serde_json::to_string(&ErrorResponse {
                    error: format!("blob hash must be 32 bytes, got {}", blob_hash_bytes.len()),
                })
                .unwrap(),
            )),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&blob_hash_bytes);
    let blob_hash = B256(hash_array);

    // Set query height
    let height = height_query
        .height
        .map(FvmQueryHeight::from)
        .unwrap_or(FvmQueryHeight::Committed);

    // Gas params for the query call
    let gas_params = GasParams {
        gas_limit: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    // First query the blobs actor to verify the blob exists
    let maybe_blob = {
        let mut client_guard = client.lock().await;
        client_guard
            .blob_get_call(blob_hash, TokenAmount::default(), gas_params, height)
            .await
    };

    match maybe_blob {
        Ok(Some(blob)) => {
            // The blob hash is actually a hash sequence hash
            let hash_seq_hash = Hash::from_bytes(blob_hash.0);
            let size = blob.size;

            // Read the hash sequence from Iroh to get the original content hash
            let hash_seq_bytes = match iroh.blobs_client().read_to_bytes(hash_seq_hash).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    return Ok(warp::reply::with_status(
                        warp::reply::Response::new(Body::from(
                            serde_json::to_string(&ErrorResponse {
                                error: format!("failed to read hash sequence: {}", e),
                            })
                            .unwrap(),
                        )),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                }
            };

            let hash_seq = match HashSeq::try_from(hash_seq_bytes) {
                Ok(seq) => seq,
                Err(e) => {
                    return Ok(warp::reply::with_status(
                        warp::reply::Response::new(Body::from(
                            serde_json::to_string(&ErrorResponse {
                                error: format!("failed to parse hash sequence: {}", e),
                            })
                            .unwrap(),
                        )),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                }
            };

            // First hash in the sequence is the original content
            let orig_hash = match hash_seq.iter().next() {
                Some(hash) => hash,
                None => {
                    return Ok(warp::reply::with_status(
                        warp::reply::Response::new(Body::from(
                            serde_json::to_string(&ErrorResponse {
                                error: "hash sequence is empty".to_string(),
                            })
                            .unwrap(),
                        )),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                }
            };

            // Read the actual content from Iroh
            let reader = match iroh.blobs_client().read(orig_hash).await {
                Ok(reader) => reader,
                Err(e) => {
                    return Ok(warp::reply::with_status(
                        warp::reply::Response::new(Body::from(
                            serde_json::to_string(&ErrorResponse {
                                error: format!("failed to read blob content: {}", e),
                            })
                            .unwrap(),
                        )),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                }
            };

            // Stream the content as the response body
            let bytes_stream = reader.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
            let body = Body::wrap_stream(bytes_stream);

            let mut response = warp::reply::Response::new(body);
            response.headers_mut().insert(
                "Content-Type",
                warp::http::HeaderValue::from_static("application/octet-stream"),
            );
            response.headers_mut().insert(
                "Content-Length",
                warp::http::HeaderValue::from(size),
            );

            Ok(warp::reply::with_status(response, warp::http::StatusCode::OK))
        }
        Ok(None) => Ok(warp::reply::with_status(
            warp::reply::Response::new(Body::from(
                serde_json::to_string(&ErrorResponse {
                    error: "blob not found".to_string(),
                })
                .unwrap(),
            )),
            warp::http::StatusCode::NOT_FOUND,
        )),
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::Response::new(Body::from(
                serde_json::to_string(&ErrorResponse {
                    error: format!("query failed: {}", e),
                })
                .unwrap(),
            )),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}
