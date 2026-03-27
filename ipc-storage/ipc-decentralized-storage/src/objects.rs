// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Objects API service for handling object upload and download
//!
//! This module provides HTTP endpoints for:
//! - Uploading objects to Iroh storage with erasure encoding + distribution
//! - Downloading objects from buckets
//! - Downloading blobs directly

use std::{convert::Infallible, net::SocketAddr, path::Path, str::FromStr, time::Instant};

use anyhow::{anyhow, Context, Result};
use bytes::Buf;
use erasure_encoding::{BlobId, NodeId};
use fendermint_actor_bucket::{GetParams, Object};
use fendermint_rpc::{client::FendermintClient, message::GasParams, QueryClient};
use fendermint_vm_message::query::FvmQueryHeight;
use futures_util::StreamExt;
use fvm_shared::address::{Address, Error as NetworkError, Network};
use fvm_shared::econ::TokenAmount;
use ipc_api::ethers_address_to_fil_address;
use iroh::NodeAddr;
use iroh_blobs::Hash;
use iroh_manager::{BlobsClient, IrohNode};
use lazy_static::lazy_static;
use mime_guess::get_mime_extensions_str;
use prometheus::{register_histogram, register_int_counter, Histogram, IntCounter};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;
use warp::path::Tail;
use warp::{
    filters::multipart::Part,
    http::{HeaderMap, HeaderValue, StatusCode},
    hyper::body::Body,
    Filter, Rejection, Reply,
};

use crate::distribution::{distribute, DistributeParams, NodeDirectory, NodeRpcDirectory};
use crate::gateway::BlobGateway;
use crate::retrieval::{retrieve, BlobRetrievalParams};

/// Number of data shards per chunk for erasure encoding (k).
const DEFAULT_DATA_SHARDS: usize = 4;
/// Number of parity shards per chunk for erasure encoding (m).
const DEFAULT_PARITY_SHARDS: usize = 2;

/// Configuration for the objects service
#[derive(Clone, Debug)]
pub struct ObjectsConfig {
    /// Listen address for the HTTP server
    pub listen_addr: SocketAddr,
    /// Tendermint RPC URL for FendermintClient
    pub tendermint_url: tendermint_rpc::Url,
    /// Maximum object size in bytes
    pub max_object_size: u64,
    /// Enable metrics
    pub metrics_enabled: bool,
    /// Metrics listen address
    pub metrics_listen: Option<SocketAddr>,
}

impl Default for ObjectsConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:8080".parse().unwrap(),
            tendermint_url: "http://localhost:26657".parse().unwrap(),
            max_object_size: 100 * 1024 * 1024, // 100MB
            metrics_enabled: false,
            metrics_listen: None,
        }
    }
}

/// Run the objects service
///
/// This starts an HTTP server with endpoints for object upload/download.
pub async fn run_objects_service(
    config: ObjectsConfig,
    iroh_node: IrohNode,
    iroh_resolver_blobs: BlobsClient,
) -> Result<()> {
    if config.metrics_enabled {
        if let Some(metrics_listen) = config.metrics_listen {
            info!(listen_addr = %metrics_listen, "serving metrics");
            let builder = prometheus_exporter::Builder::new(metrics_listen);
            let _ = builder.start().context("failed to start metrics server")?;
        }
    } else {
        info!("metrics disabled");
    }

    let client = FendermintClient::new_http(config.tendermint_url, None)?;

    // Admin routes
    let health = warp::path!("health")
        .and(warp::get())
        .and_then(handle_health);
    let node_addr = warp::path!("v1" / "node")
        .and(warp::get())
        .and(with_iroh(iroh_node.clone()))
        .and_then(handle_node_addr);

    // Objects routes
    let objects_upload = warp::path!("v1" / "objects")
        .and(warp::post())
        .and(with_iroh(iroh_node.clone()))
        .and(warp::multipart::form().max_length(config.max_object_size + 1024 * 1024))
        .and(with_max_size(config.max_object_size))
        .and(with_client(client.clone()))
        .and_then(handle_object_upload);

    let objects_download = warp::path!("v1" / "objects" / String / ..)
        .and(warp::path::tail())
        .and(
            warp::get()
                .map(|| "GET".to_string())
                .or(warp::head().map(|| "HEAD".to_string()))
                .unify(),
        )
        .and(warp::query::<HeightQuery>())
        .and(with_client(client.clone()))
        .and(with_iroh_blobs(iroh_resolver_blobs.clone()))
        .and_then(handle_object_download);

    let blobs_download = warp::path!("v1" / "blobs" / String)
        .and(
            warp::get()
                .map(|| "GET".to_string())
                .or(warp::head().map(|| "HEAD".to_string()))
                .unify(),
        )
        .and(warp::query::<HeightQuery>())
        .and(with_client(client.clone()))
        .and(with_iroh_blobs(iroh_resolver_blobs.clone()))
        .and_then(handle_blob_download);

    let router = health
        .or(node_addr)
        .or(objects_upload)
        .or(blobs_download)
        .or(objects_download)
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_headers(vec!["Content-Type"])
                .allow_methods(vec!["POST", "DEL", "GET", "HEAD"]),
        )
        .recover(handle_rejection);

    info!(listen_addr = %config.listen_addr, "starting objects service");
    warp::serve(router).run(config.listen_addr).await;

    Ok(())
}

/// Create the objects service routes (for integration into existing servers)
pub fn objects_routes(
    client: FendermintClient,
    iroh_node: IrohNode,
    iroh_resolver_blobs: BlobsClient,
    max_object_size: u64,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let health = warp::path!("health")
        .and(warp::get())
        .and_then(handle_health);
    let node_addr = warp::path!("v1" / "node")
        .and(warp::get())
        .and(with_iroh(iroh_node.clone()))
        .and_then(handle_node_addr);

    let objects_upload = warp::path!("v1" / "objects")
        .and(warp::post())
        .and(with_iroh(iroh_node.clone()))
        .and(warp::multipart::form().max_length(max_object_size + 1024 * 1024))
        .and(with_max_size(max_object_size))
        .and(with_client(client.clone()))
        .and_then(handle_object_upload);

    let objects_download = warp::path!("v1" / "objects" / String / ..)
        .and(warp::path::tail())
        .and(
            warp::get()
                .map(|| "GET".to_string())
                .or(warp::head().map(|| "HEAD".to_string()))
                .unify(),
        )
        .and(warp::query::<HeightQuery>())
        .and(with_client(client.clone()))
        .and(with_iroh_blobs(iroh_resolver_blobs.clone()))
        .and_then(handle_object_download);

    let blobs_download = warp::path!("v1" / "blobs" / String)
        .and(
            warp::get()
                .map(|| "GET".to_string())
                .or(warp::head().map(|| "HEAD".to_string()))
                .unify(),
        )
        .and(warp::query::<HeightQuery>())
        .and(with_client(client.clone()))
        .and(with_iroh_blobs(iroh_resolver_blobs.clone()))
        .and_then(handle_blob_download);

    health
        .or(node_addr)
        .or(objects_upload)
        .or(blobs_download)
        .or(objects_download)
}

fn with_client(
    client: FendermintClient,
) -> impl Filter<Extract = (FendermintClient,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_iroh(client: IrohNode) -> impl Filter<Extract = (IrohNode,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_iroh_blobs(
    client: BlobsClient,
) -> impl Filter<Extract = (BlobsClient,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_max_size(max_size: u64) -> impl Filter<Extract = (u64,), Error = Infallible> + Clone {
    warp::any().map(move || max_size)
}

#[derive(Serialize, Deserialize)]
struct HeightQuery {
    pub height: Option<u64>,
}

#[derive(Default)]
struct ObjectParser {
    size: Option<u64>,
    data_part: Option<Part>,
}

impl ObjectParser {
    async fn read_part(&mut self, part: Part) -> anyhow::Result<Vec<u8>> {
        let value = part
            .stream()
            .fold(Vec::new(), |mut vec, data| async move {
                if let Ok(data) = data {
                    vec.extend_from_slice(data.chunk());
                }
                vec
            })
            .await;
        Ok(value)
    }

    async fn read_size(&mut self, form_part: Part) -> anyhow::Result<()> {
        let value = self.read_part(form_part).await?;
        let text = String::from_utf8(value).map_err(|_| anyhow!("cannot parse size"))?;
        let size: u64 = text.parse().map_err(|_| anyhow!("cannot parse size"))?;
        self.size = Some(size);
        Ok(())
    }

    async fn read_form(mut form_data: warp::multipart::FormData) -> anyhow::Result<Self> {
        let mut object_parser = ObjectParser::default();
        while let Some(part) = form_data.next().await {
            let part = part.map_err(|e| anyhow!("cannot read form data: {}", e))?;
            match part.name() {
                "size" => {
                    object_parser.read_size(part).await?;
                }
                "data" => {
                    object_parser.data_part = Some(part);
                    // This early return was added to avoid the "failed to lock multipart state" error.
                    // It implies that the data field must be the last one sent in the multipart form.
                    return Ok(object_parser);
                }
                _ => {
                    // Ignore unknown fields for forward compatibility
                    let _ = object_parser.read_part(part).await?;
                }
            }
        }
        Ok(object_parser)
    }
}

lazy_static! {
    static ref COUNTER_BLOBS_UPLOADED: IntCounter = register_int_counter!(
        "objects_blobs_uploaded_total",
        "Number of successfully uploaded blobs"
    )
    .unwrap();
    static ref COUNTER_BYTES_UPLOADED: IntCounter = register_int_counter!(
        "objects_bytes_uploaded_total",
        "Number of successfully uploaded bytes"
    )
    .unwrap();
    static ref HISTOGRAM_UPLOAD_TIME: Histogram = register_histogram!(
        "objects_upload_time_seconds",
        "Time spent uploading an object in seconds"
    )
    .unwrap();
    static ref COUNTER_BLOBS_DOWNLOADED: IntCounter = register_int_counter!(
        "objects_blobs_downloaded_total",
        "Number of successfully downloaded blobs"
    )
    .unwrap();
    static ref COUNTER_BYTES_DOWNLOADED: IntCounter = register_int_counter!(
        "objects_bytes_downloaded_total",
        "Number of successfully downloaded bytes"
    )
    .unwrap();
    static ref HISTOGRAM_DOWNLOAD_TIME: Histogram = register_histogram!(
        "objects_download_time_seconds",
        "Time spent downloading an object in seconds"
    )
    .unwrap();
}

async fn handle_health() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::reply())
}

async fn handle_node_addr(iroh: IrohNode) -> Result<impl Reply, Rejection> {
    let node_addr = iroh.endpoint().node_addr().await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to get iroh node address info: {}", e),
        })
    })?;
    Ok(warp::reply::json(&node_addr))
}

#[derive(Serialize)]
struct UploadResponse {
    /// Original blob content hash.
    hash: String,
    /// Number of chunks the data was split into.
    num_chunks: usize,
    /// Number of data shards per chunk (k).
    data_shards: usize,
    /// Number of parity shards per chunk (m).
    parity_shards: usize,
    /// Original data length in bytes.
    original_len: usize,
}

async fn handle_object_upload(
    iroh: IrohNode,
    form_data: warp::multipart::FormData,
    max_size: u64,
    client: FendermintClient,
) -> Result<impl Reply, Rejection> {
    let start_time = Instant::now();
    let parser = ObjectParser::read_form(form_data).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to read form: {}", e),
        })
    })?;

    let size = match parser.size {
        Some(size) => size,
        None => {
            return Err(Rejection::from(BadRequest {
                message: "missing size in form".to_string(),
            }))
        }
    };
    if size > max_size {
        return Err(Rejection::from(BadRequest {
            message: format!("blob size exceeds maximum of {}", max_size),
        }));
    }

    let _upload_id = Uuid::new_v4();

    // Collect upload data into memory
    let data_part = match parser.data_part {
        Some(part) => part,
        None => {
            return Err(Rejection::from(BadRequest {
                message: "missing data in form".to_string(),
            }))
        }
    };

    let mut data = Vec::new();
    let mut stream = data_part.stream();
    while let Some(result) = stream.next().await {
        let mut buf = result.map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("failed to read upload stream: {}", e),
            })
        })?;
        data.extend_from_slice(&buf.copy_to_bytes(buf.remaining()));
    }

    if data.len() as u64 != size {
        return Err(Rejection::from(BadRequest {
            message: format!(
                "data size and given size do not match (expected {}, got {})",
                size,
                data.len()
            ),
        }));
    }

    // Compute content hash (blake3) from the uploaded data
    let hash = Hash::from(*blake3::hash(&data).as_bytes());

    COUNTER_BYTES_UPLOADED.inc_by(data.len() as u64);
    debug!("uploaded blob {} (size: {})", hash, data.len());

    // Build node directories from on-chain state
    let gateway = BlobGateway::new(client, 10, std::time::Duration::from_secs(5));
    let (nodes, node_directory, node_rpc_directory) =
        build_node_directories(&gateway).await.map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("failed to build node directories: {}", e),
            })
        })?;

    // Get local node address for P2P shard distribution
    let local_node_addr = iroh.endpoint().node_addr().await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to get local node address: {}", e),
        })
    })?;

    // Erasure-encode and distribute shards to assigned nodes
    let blob_id = BlobId(*hash.as_bytes());
    info!(
        "Distributing blob: hash={}, data_len={}, blob_id={}, nodes={}",
        hash,
        data.len(),
        hex::encode(blob_id.0),
        nodes.len()
    );
    let dist_result = distribute(
        DistributeParams {
            blob_id,
            data,
            data_shards: DEFAULT_DATA_SHARDS,
            parity_shards: DEFAULT_PARITY_SHARDS,
            nodes,
            node_directory,
            node_rpc_directory,
        },
        iroh.blobs_client(),
        &local_node_addr,
    )
    .await
    .map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to distribute shards: {}", e),
        })
    })?;

    info!(
        "Distribution complete: num_chunks={}, k={}, m={}, original_len={}, failures={}, total_shards={}",
        dist_result.metadata.num_chunks,
        dist_result.metadata.data_shards,
        dist_result.metadata.parity_shards,
        dist_result.metadata.original_len,
        dist_result.failure_count(),
        dist_result.shard_results.len()
    );
    for sr in &dist_result.shard_results {
        info!(
            "  Shard result {}/{}: hash={:?}, success={}, error={:?}",
            sr.chunk_index, sr.shard_index, sr.iroh_hash, sr.success, sr.error
        );
    }

    COUNTER_BLOBS_UPLOADED.inc();
    HISTOGRAM_UPLOAD_TIME.observe(start_time.elapsed().as_secs_f64());

    let response = UploadResponse {
        hash: hash.to_string(),
        num_chunks: dist_result.metadata.num_chunks,
        data_shards: dist_result.metadata.data_shards,
        parity_shards: dist_result.metadata.parity_shards,
        original_len: dist_result.metadata.original_len,
    };
    Ok(warp::reply::json(&response))
}

/// Query on-chain operator state and build node directories for shard distribution.
pub async fn build_node_directories<C: QueryClient + Send + Sync>(
    gateway: &BlobGateway<C>,
) -> Result<(Vec<NodeId>, NodeDirectory, NodeRpcDirectory)> {
    let operators = gateway
        .query_active_operators()
        .await
        .context("failed to query active operators")?;

    if operators.is_empty() {
        anyhow::bail!("no active operators found");
    }

    let mut nodes = Vec::new();
    let mut node_directory = NodeDirectory::new();
    let mut node_rpc_directory = NodeRpcDirectory::new();

    let http_client = reqwest::Client::new();

    for operator_addr in &operators {
        let info = gateway
            .get_operator_info(*operator_addr)
            .await
            .context("failed to get operator info")?;

        // Query the operator's RPC endpoint to get their Iroh NodeAddr
        let url = format!("{}/v1/node", info.rpc_url.trim_end_matches('/'));
        let resp = http_client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("failed to query node addr from {}", url))?;

        if !resp.status().is_success() {
            anyhow::bail!("node {} returned {}", url, resp.status());
        }

        let node_addr: NodeAddr = resp
            .json()
            .await
            .with_context(|| format!("failed to parse node addr from {}", url))?;

        let node_id = NodeId(*node_addr.node_id.as_bytes());

        nodes.push(node_id);
        node_directory.insert(node_id, node_addr);
        node_rpc_directory.insert(node_id, info.rpc_url.clone());
    }

    Ok((nodes, node_directory, node_rpc_directory))
}

async fn handle_object_download<F: QueryClient + Send + Sync + Clone>(
    address: String,
    tail: Tail,
    method: String,
    height_query: HeightQuery,
    client: F,
    iroh: BlobsClient,
) -> Result<impl Reply, Rejection> {
    let address = parse_address(&address).map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("invalid address {}: {}", address, e),
        })
    })?;
    let height = height_query
        .height
        .unwrap_or(FvmQueryHeight::Committed.into());

    let path = urlencoding::decode(tail.as_str())
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("invalid address {}: {}", address, e),
            })
        })?
        .to_string();

    let key: Vec<u8> = path.into();
    let start_time = Instant::now();
    let maybe_object = os_get(client.clone(), address, GetParams(key.clone()), height)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("bucket get error: {}", e),
            })
        })?;

    match maybe_object {
        Some(object) => {
            let size = object.size;

            // Retrieve blob by fetching shards from assigned nodes and RS-decoding
            let gateway = BlobGateway::new(client, 10, std::time::Duration::from_secs(5));
            let (nodes, node_directory, node_rpc_directory) =
                build_node_directories(&gateway).await.map_err(|e| {
                    Rejection::from(BadRequest {
                        message: format!("failed to build node directories: {}", e),
                    })
                })?;

            let blob_id = BlobId(object.hash.0);
            let retrieved_data = retrieve(
                &BlobRetrievalParams {
                    blob_id,
                    original_len: size as usize,
                    data_shards: DEFAULT_DATA_SHARDS,
                    parity_shards: DEFAULT_PARITY_SHARDS,
                    nodes,
                    node_directory,
                    node_rpc_directory,
                },
                &iroh,
            )
            .await
            .map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to retrieve blob: {}", e),
                })
            })?;

            // HEAD request: return headers only
            if method == "HEAD" {
                let mut response = warp::reply::Response::new(Body::empty());
                let mut header_map = HeaderMap::new();
                header_map.insert("Content-Length", HeaderValue::from(size));
                let headers = response.headers_mut();
                headers.extend(header_map);
                return Ok(response);
            }

            let mut response = warp::reply::Response::new(Body::from(retrieved_data));
            let mut header_map = HeaderMap::new();
            header_map.insert("Content-Length", HeaderValue::from(size));

            let content_type = object
                .metadata
                .get("content-type")
                .cloned()
                .unwrap_or_else(|| "application/octet-stream".to_string());
            header_map.insert(
                "Content-Type",
                HeaderValue::from_str(&content_type).unwrap(),
            );

            let key_str = String::from_utf8_lossy(&key);
            if let Some(val) = get_filename_with_extension(&key_str, &content_type) {
                let disposition = format!("attachment; filename=\"{}\"", val);
                header_map.insert(
                    "Content-Disposition",
                    HeaderValue::from_str(&disposition).unwrap(),
                );
            }

            let headers = response.headers_mut();
            headers.extend(header_map);

            COUNTER_BLOBS_DOWNLOADED.inc();
            COUNTER_BYTES_DOWNLOADED.inc_by(size);
            HISTOGRAM_DOWNLOAD_TIME.observe(start_time.elapsed().as_secs_f64());

            Ok(response)
        }
        None => Err(Rejection::from(NotFound)),
    }
}

/// Handle direct blob download by querying the blobs actor.
async fn handle_blob_download<F: QueryClient + Send + Sync + Clone>(
    blob_hash_str: String,
    method: String,
    height_query: HeightQuery,
    client: F,
    iroh: BlobsClient,
) -> Result<impl Reply, Rejection> {
    // Strip 0x prefix if present
    let blob_hash_hex = blob_hash_str.strip_prefix("0x").unwrap_or(&blob_hash_str);

    let blob_hash_bytes = hex::decode(blob_hash_hex).map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("invalid blob hash {}: {}", blob_hash_str, e),
        })
    })?;

    if blob_hash_bytes.len() != 32 {
        return Err(Rejection::from(BadRequest {
            message: format!("blob hash must be 32 bytes, got {}", blob_hash_bytes.len()),
        }));
    }

    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&blob_hash_bytes);
    let blob_hash = fendermint_actor_blobs_shared::bytes::B256(hash_array);

    let height = height_query
        .height
        .unwrap_or(FvmQueryHeight::Committed.into());

    let start_time = Instant::now();

    // Query the blobs actor to get blob info
    let maybe_blob = blob_get(client.clone(), blob_hash, height)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("blobs actor query error: {}", e),
            })
        })?;

    match maybe_blob {
        Some(blob) => {
            let size = blob.size;

            debug!("blob download: hash={}, size={}", blob_hash, size);

            // Retrieve blob by fetching shards from assigned nodes and RS-decoding
            let gateway = BlobGateway::new(client, 10, std::time::Duration::from_secs(5));
            let (nodes, node_directory, node_rpc_directory) =
                build_node_directories(&gateway).await.map_err(|e| {
                    Rejection::from(BadRequest {
                        message: format!("failed to build node directories: {}", e),
                    })
                })?;

            let blob_id = BlobId(blob_hash.0);
            let retrieved_data = retrieve(
                &BlobRetrievalParams {
                    blob_id,
                    original_len: size as usize,
                    data_shards: DEFAULT_DATA_SHARDS,
                    parity_shards: DEFAULT_PARITY_SHARDS,
                    nodes,
                    node_directory,
                    node_rpc_directory,
                },
                &iroh,
            )
            .await
            .map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to retrieve blob: {}", e),
                })
            })?;

            // HEAD request: return headers only
            if method == "HEAD" {
                let mut response = warp::reply::Response::new(Body::empty());
                let mut header_map = HeaderMap::new();
                header_map.insert("Content-Length", HeaderValue::from(size));
                let headers = response.headers_mut();
                headers.extend(header_map);
                return Ok(response);
            }

            let mut response = warp::reply::Response::new(Body::from(retrieved_data));
            let mut header_map = HeaderMap::new();
            header_map.insert("Content-Length", HeaderValue::from(size));
            header_map.insert(
                "Content-Type",
                HeaderValue::from_str("application/octet-stream").unwrap(),
            );

            let headers = response.headers_mut();
            headers.extend(header_map);

            COUNTER_BLOBS_DOWNLOADED.inc();
            COUNTER_BYTES_DOWNLOADED.inc_by(size);
            HISTOGRAM_DOWNLOAD_TIME.observe(start_time.elapsed().as_secs_f64());

            Ok(response)
        }
        None => Err(Rejection::from(NotFound)),
    }
}

/// Parse an f/eth-address from string.
pub fn parse_address(s: &str) -> anyhow::Result<Address> {
    let addr = Network::Mainnet
        .parse_address(s)
        .or_else(|e| match e {
            NetworkError::UnknownNetwork => Network::Testnet.parse_address(s),
            _ => Err(e),
        })
        .or_else(|_| {
            let addr = ethers::types::Address::from_str(s)?;
            ethers_address_to_fil_address(&addr)
        })?;
    Ok(addr)
}

// Rejection handlers

#[derive(Clone, Debug)]
struct BadRequest {
    message: String,
}

impl warp::reject::Reject for BadRequest {}

#[derive(Debug)]
struct NotFound;

impl warp::reject::Reject for NotFound {}

#[derive(Clone, Debug, Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() || err.find::<NotFound>().is_some() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(e) = err.find::<BadRequest>() {
        let err = e.to_owned();
        (StatusCode::BAD_REQUEST, err.message)
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            "Payload too large".to_string(),
        )
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
    };

    let reply = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });
    let reply = warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*");
    Ok(warp::reply::with_status(reply, code))
}

// RPC methods

async fn os_get<F: QueryClient + Send + Sync>(
    mut client: F,
    address: Address,
    params: GetParams,
    height: u64,
) -> anyhow::Result<Option<Object>> {
    let gas_params = GasParams {
        gas_limit: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };
    let h = FvmQueryHeight::from(height);

    let return_data = client
        .os_get_call(address, params, TokenAmount::default(), gas_params, h)
        .await?;

    Ok(return_data)
}

async fn blob_get<F: QueryClient + Send + Sync>(
    mut client: F,
    blob_hash: fendermint_actor_blobs_shared::bytes::B256,
    height: u64,
) -> anyhow::Result<Option<fendermint_actor_blobs_shared::blobs::Blob>> {
    let gas_params = GasParams {
        gas_limit: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };
    let h = FvmQueryHeight::from(height);

    let return_data = client
        .blob_get_call(blob_hash, TokenAmount::default(), gas_params, h)
        .await?;

    Ok(return_data)
}

fn get_filename_with_extension(filename: &str, content_type: &str) -> Option<String> {
    let path = Path::new(filename);

    // Checks if filename already has extension
    if path.extension().and_then(|ext| ext.to_str()).is_some() {
        return Some(filename.to_string());
    }

    get_mime_extensions_str(content_type)?
        .first()
        .map(|ext| format!("{}.{}", filename, ext))
}
