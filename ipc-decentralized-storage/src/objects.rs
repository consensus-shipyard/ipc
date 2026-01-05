// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Objects API service for handling object upload and download
//!
//! This module provides HTTP endpoints for:
//! - Uploading objects to Iroh storage with entanglement
//! - Downloading objects from buckets
//! - Downloading blobs directly

use std::{
    convert::Infallible, net::SocketAddr, num::ParseIntError, path::Path, str::FromStr,
    time::Instant,
};

use anyhow::{anyhow, Context, Result};
use bytes::Buf;
use entangler::{ChunkRange, Config, EntanglementResult, Entangler};
use entangler_storage::iroh::IrohStorage as EntanglerIrohStorage;
use fendermint_actor_bucket::{GetParams, Object};
use fendermint_rpc::{client::FendermintClient, message::GasParams, QueryClient};
use fendermint_vm_message::query::FvmQueryHeight;
use futures_util::{StreamExt, TryStreamExt};
use fvm_shared::address::{Address, Error as NetworkError, Network};
use fvm_shared::econ::TokenAmount;
use ipc_api::ethers_address_to_fil_address;
use iroh::NodeAddr;
use iroh_blobs::{hashseq::HashSeq, rpc::client::blobs::BlobStatus, util::SetTagOption, Hash};
use iroh_manager::{get_blob_hash_and_size, BlobsClient, IrohNode};
use lazy_static::lazy_static;
use mime_guess::get_mime_extensions_str;
use prometheus::{register_histogram, register_int_counter, Histogram, IntCounter};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info};
use uuid::Uuid;
use warp::path::Tail;
use warp::{
    filters::multipart::Part,
    http::{HeaderMap, HeaderValue, StatusCode},
    hyper::body::Body,
    Filter, Rejection, Reply,
};

/// The alpha parameter for alpha entanglement determines the number of parity blobs to generate
/// for the original blob.
const ENTANGLER_ALPHA: u8 = 3;
/// The s parameter for alpha entanglement determines the number of horizontal strands in the grid.
const ENTANGLER_S: u8 = 5;
/// Chunk size used by the entangler.
const CHUNK_SIZE: u64 = 1024;

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
    let health = warp::path!("health").and(warp::get()).and_then(handle_health);
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
        .and_then(handle_object_upload);

    let objects_download = warp::path!("v1" / "objects" / String / ..)
        .and(warp::path::tail())
        .and(
            warp::get()
                .map(|| "GET".to_string())
                .or(warp::head().map(|| "HEAD".to_string()))
                .unify(),
        )
        .and(warp::header::optional::<String>("Range"))
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
        .and(warp::header::optional::<String>("Range"))
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
    let health = warp::path!("health").and(warp::get()).and_then(handle_health);
    let node_addr = warp::path!("v1" / "node")
        .and(warp::get())
        .and(with_iroh(iroh_node.clone()))
        .and_then(handle_node_addr);

    let objects_upload = warp::path!("v1" / "objects")
        .and(warp::post())
        .and(with_iroh(iroh_node.clone()))
        .and(warp::multipart::form().max_length(max_object_size + 1024 * 1024))
        .and(with_max_size(max_object_size))
        .and_then(handle_object_upload);

    let objects_download = warp::path!("v1" / "objects" / String / ..)
        .and(warp::path::tail())
        .and(
            warp::get()
                .map(|| "GET".to_string())
                .or(warp::head().map(|| "HEAD".to_string()))
                .unify(),
        )
        .and(warp::header::optional::<String>("Range"))
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
        .and(warp::header::optional::<String>("Range"))
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

#[derive(Debug, Error)]
enum ObjectsError {
    #[error("error parsing range header: `{0}`")]
    RangeHeaderParseError(ParseIntError),
    #[error("invalid range header")]
    RangeHeaderInvalid,
}

impl From<ParseIntError> for ObjectsError {
    fn from(err: ParseIntError) -> Self {
        ObjectsError::RangeHeaderParseError(err)
    }
}

#[derive(Default)]
struct ObjectParser {
    hash: Option<Hash>,
    size: Option<u64>,
    source: Option<NodeAddr>,
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

    async fn read_hash(&mut self, form_part: Part) -> anyhow::Result<()> {
        let value = self.read_part(form_part).await?;
        let text = String::from_utf8(value).map_err(|_| anyhow!("cannot parse hash"))?;
        let hash: Hash = text.parse().map_err(|_| anyhow!("cannot parse hash"))?;
        self.hash = Some(hash);
        Ok(())
    }

    async fn read_size(&mut self, form_part: Part) -> anyhow::Result<()> {
        let value = self.read_part(form_part).await?;
        let text = String::from_utf8(value).map_err(|_| anyhow!("cannot parse size"))?;
        let size: u64 = text.parse().map_err(|_| anyhow!("cannot parse size"))?;
        self.size = Some(size);
        Ok(())
    }

    async fn read_source(&mut self, form_part: Part) -> anyhow::Result<()> {
        let value = self.read_part(form_part).await?;
        let text = String::from_utf8(value).map_err(|_| anyhow!("cannot parse source"))?;
        let source: NodeAddr =
            serde_json::from_str(&text).map_err(|_| anyhow!("cannot parse source"))?;
        self.source = Some(source);
        Ok(())
    }

    async fn read_form(mut form_data: warp::multipart::FormData) -> anyhow::Result<Self> {
        let mut object_parser = ObjectParser::default();
        while let Some(part) = form_data.next().await {
            let part = part.map_err(|e| anyhow!("cannot read form data: {}", e))?;
            match part.name() {
                "hash" => {
                    object_parser.read_hash(part).await?;
                }
                "size" => {
                    object_parser.read_size(part).await?;
                }
                "source" => {
                    object_parser.read_source(part).await?;
                }
                "data" => {
                    object_parser.data_part = Some(part);
                    // This early return was added to avoid the "failed to lock multipart state" error.
                    // It implies that the data field must be the last one sent in the multipart form.
                    return Ok(object_parser);
                }
                // Ignore but accept signature-related fields for backward compatibility
                "chain_id" | "msg" => {
                    // Read and discard the data
                    let _ = object_parser.read_part(part).await?;
                }
                _ => {
                    return Err(anyhow!("unknown form field"));
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
    hash: String,      // Hash sequence hash (for bucket storage)
    orig_hash: String, // Original blob content hash (for addBlob)
    metadata_hash: String,
}

async fn handle_object_upload(
    iroh: IrohNode,
    form_data: warp::multipart::FormData,
    max_size: u64,
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

    let upload_id = Uuid::new_v4();

    // Handle the two upload cases
    let hash = match (parser.source, parser.data_part) {
        // Case 1: Source node provided - download from the source
        (Some(source), None) => {
            let hash = match parser.hash {
                Some(hash) => hash,
                None => {
                    return Err(Rejection::from(BadRequest {
                        message: "missing hash in form".to_string(),
                    }))
                }
            };

            let tag = iroh_blobs::Tag(format!("temp-{hash}-{upload_id}").into());
            let progress = iroh
                .blobs_client()
                .download_with_opts(
                    hash,
                    iroh_blobs::rpc::client::blobs::DownloadOptions {
                        format: iroh_blobs::BlobFormat::Raw,
                        nodes: vec![source],
                        tag: SetTagOption::Named(tag),
                        mode: iroh_blobs::rpc::client::blobs::DownloadMode::Queued,
                    },
                )
                .await
                .map_err(|e| {
                    Rejection::from(BadRequest {
                        message: format!("failed to fetch blob {}: {}", hash, e),
                    })
                })?;
            let outcome = progress.finish().await.map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to fetch blob {}: {}", hash, e),
                })
            })?;
            let outcome_size = outcome.local_size + outcome.downloaded_size;
            if outcome_size != size {
                return Err(Rejection::from(BadRequest {
                    message: format!(
                        "blob size and given size do not match (expected {}, got {})",
                        size, outcome_size
                    ),
                }));
            }

            debug!(
                "downloaded blob {} in {:?} (size: {}; local_size: {}; downloaded_size: {})",
                hash, outcome.stats.elapsed, size, outcome.local_size, outcome.downloaded_size,
            );
            COUNTER_BYTES_UPLOADED.inc_by(outcome.downloaded_size);
            hash
        }

        // Case 2: Direct upload - store the provided data
        (None, Some(data_part)) => {
            let stream = data_part.stream().map(|result| {
                result
                    .map(|mut buf| buf.copy_to_bytes(buf.remaining()))
                    .map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::Other, format!("Warp error: {}", e))
                    })
            });

            let batch = iroh.blobs_client().batch().await.map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to store blob: {}", e),
                })
            })?;
            let temp_tag = batch.add_stream(stream).await.map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to store blob: {}", e),
                })
            })?;

            let hash = *temp_tag.hash();
            let new_tag = iroh_blobs::Tag(format!("temp-{hash}-{upload_id}").into());
            batch.persist_to(temp_tag, new_tag).await.map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to persist blob: {}", e),
                })
            })?;

            drop(batch);

            let status = iroh.blobs_client().status(hash).await.map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to check blob status: {}", e),
                })
            })?;
            let BlobStatus::Complete { size } = status else {
                return Err(Rejection::from(BadRequest {
                    message: "failed to store data".to_string(),
                }));
            };
            COUNTER_BYTES_UPLOADED.inc_by(size);
            debug!("stored uploaded blob {} (size: {})", hash, size);

            hash
        }

        (Some(_), Some(_)) => {
            return Err(Rejection::from(BadRequest {
                message: "cannot provide both source and data".to_string(),
            }));
        }

        (None, None) => {
            return Err(Rejection::from(BadRequest {
                message: "must provide either source or data".to_string(),
            }));
        }
    };

    debug!("raw uploaded hash: {}", hash);

    let ent = new_entangler(iroh.blobs_client()).map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to create entangler: {}", e),
        })
    })?;
    let ent_result = ent.entangle_uploaded(hash.to_string()).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to entangle uploaded data: {}", e),
        })
    })?;

    debug!(
        "entanglement result: orig_hash={}, metadata_hash={}, upload_results_count={}",
        ent_result.orig_hash,
        ent_result.metadata_hash,
        ent_result.upload_results.len()
    );

    let hash_seq_hash = tag_entangled_data(&iroh, &ent_result, upload_id)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("failed to tag entangled data: {}", e),
            })
        })?;

    debug!("hash_seq_hash: {}", hash_seq_hash);

    COUNTER_BLOBS_UPLOADED.inc();
    HISTOGRAM_UPLOAD_TIME.observe(start_time.elapsed().as_secs_f64());

    let response = UploadResponse {
        hash: hash_seq_hash.to_string(),
        orig_hash: ent_result.orig_hash.clone(),
        metadata_hash: ent_result.metadata_hash,
    };
    Ok(warp::reply::json(&response))
}

async fn tag_entangled_data(
    iroh: &IrohNode,
    ent_result: &EntanglementResult,
    upload_id: Uuid,
) -> Result<Hash, anyhow::Error> {
    let orig_hash = Hash::from_str(ent_result.orig_hash.as_str())?;
    let metadata_hash = Hash::from_str(ent_result.metadata_hash.as_str())?;

    // collect all hashes related to the blob, but ignore the metadata hash, as we want to make
    // sure that the metadata hash is the second hash in the sequence after the original hash
    let upload_hashes = ent_result
        .upload_results
        .iter()
        .map(|r| Hash::from_str(&r.hash))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|h| h != &metadata_hash)
        .collect::<Vec<_>>();

    let mut hashes = vec![orig_hash, metadata_hash];
    hashes.extend(upload_hashes);

    let hashes_str = hashes
        .iter()
        .map(|h| h.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let batch = iroh.blobs_client().batch().await?;

    // make a hash sequence object from the hashes and upload it to iroh
    let hash_seq = hashes.into_iter().collect::<HashSeq>();

    let temp_tag = batch
        .add_bytes_with_opts(hash_seq, iroh_blobs::BlobFormat::HashSeq)
        .await?;
    let hash_seq_hash = *temp_tag.hash();

    debug!(
        "storing hash sequence: {} ({})",
        hash_seq_hash.to_string(),
        hashes_str
    );

    // this tag will be replaced later by the validator to "stored-seq-{hash_seq_hash}"
    let hash_seq_tag = iroh_blobs::Tag(format!("temp-seq-{hash_seq_hash}").into());
    batch.persist_to(temp_tag, hash_seq_tag).await?;

    drop(batch);

    // delete all tags returned by the entangler
    for ent_upload_result in &ent_result.upload_results {
        let tag_value = ent_upload_result
            .info
            .get("tag")
            .ok_or_else(|| anyhow!("Missing tag in entanglement upload result"))?;
        let tag = iroh_blobs::Tag::from(tag_value.clone());
        iroh.blobs_client().tags().delete(tag).await?;
    }

    // remove upload tags
    let orig_tag = iroh_blobs::Tag(format!("temp-{orig_hash}-{upload_id}").into());
    iroh.blobs_client().tags().delete(orig_tag).await?;

    Ok(hash_seq_hash)
}

fn new_entangler(iroh: &BlobsClient) -> Result<Entangler<EntanglerIrohStorage>, entangler::Error> {
    Entangler::new(
        EntanglerIrohStorage::from_client(iroh.clone()),
        Config::new(ENTANGLER_ALPHA, ENTANGLER_S),
    )
}

fn get_range_params(range: String, size: u64) -> Result<(u64, u64), ObjectsError> {
    let range: Vec<String> = range
        .replace("bytes=", "")
        .split('-')
        .map(|n| n.to_string())
        .collect();
    if range.len() != 2 {
        return Err(ObjectsError::RangeHeaderInvalid);
    }
    let (first, mut last): (u64, u64) = match (!range[0].is_empty(), !range[1].is_empty()) {
        (true, true) => (range[0].parse::<u64>()?, range[1].parse::<u64>()?),
        (true, false) => (range[0].parse::<u64>()?, size - 1),
        (false, true) => {
            let last = range[1].parse::<u64>()?;
            if last > size {
                (0, size - 1)
            } else {
                (size - last, size - 1)
            }
        }
        (false, false) => (0, size - 1),
    };
    if first > last || first >= size {
        return Err(ObjectsError::RangeHeaderInvalid);
    }
    if last >= size {
        last = size - 1;
    }
    Ok((first, last))
}

struct ObjectRange {
    start: u64,
    end: u64,
    len: u64,
    size: u64,
    body: Body,
}

async fn handle_object_download<F: QueryClient + Send + Sync>(
    address: String,
    tail: Tail,
    method: String,
    range: Option<String>,
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
    let maybe_object = os_get(client, address, GetParams(key.clone()), height)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("bucket get error: {}", e),
            })
        })?;

    match maybe_object {
        Some(object) => {
            let seq_hash = Hash::from_bytes(object.hash.0);
            let (hash, size) = get_blob_hash_and_size(&iroh, seq_hash).await.map_err(|e| {
                Rejection::from(BadRequest {
                    message: e.to_string(),
                })
            })?;

            let ent = new_entangler(&iroh).map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to create entangler: {}", e),
                })
            })?;
            let recovery_hash = Hash::from_bytes(object.recovery_hash.0);

            let object_range = match range {
                Some(range) => {
                    let (first_byte, last_byte) = get_range_params(range, size).map_err(|e| {
                        Rejection::from(BadRequest {
                            message: e.to_string(),
                        })
                    })?;
                    let len = (last_byte - first_byte) + 1;

                    let first_chunk = first_byte / CHUNK_SIZE;
                    let last_chunk = last_byte / CHUNK_SIZE;

                    let bytes_stream = ent
                        .download_range(
                            &hash.to_string(),
                            ChunkRange::Between(first_chunk, last_chunk),
                            Some(recovery_hash.to_string()),
                        )
                        .await
                        .map_err(|e| {
                            Rejection::from(BadRequest {
                                message: format!("failed to download object: {} {}", hash, e),
                            })
                        })?;

                    let offset = (first_byte % CHUNK_SIZE) as usize;
                    let end_offset = (last_byte % CHUNK_SIZE + 1) as usize;

                    let bytes_stream = bytes_stream.enumerate().map(move |(i, chunk)| {
                        let chunk = chunk?;
                        let result = if first_chunk == last_chunk {
                            // Single chunk case - slice with both offsets
                            chunk.slice(offset..end_offset)
                        } else if i == 0 {
                            // First of multiple chunks
                            chunk.slice(offset..)
                        } else if i == (last_chunk - first_chunk) as usize {
                            // Last of multiple chunks
                            chunk.slice(..end_offset)
                        } else {
                            // Middle chunks
                            chunk
                        };
                        Ok::<_, anyhow::Error>(result)
                    });

                    let body = Body::wrap_stream(bytes_stream);
                    ObjectRange {
                        start: first_byte,
                        end: last_byte,
                        len,
                        size,
                        body,
                    }
                }
                None => {
                    let bytes_stream = ent
                        .download(&hash.to_string(), Some(&recovery_hash.to_string()))
                        .await
                        .map_err(|e| {
                            Rejection::from(BadRequest {
                                message: format!("failed to download object: {} {}", hash, e),
                            })
                        })?;
                    let body = Body::wrap_stream(bytes_stream.map_err(|e| anyhow::anyhow!(e)));
                    ObjectRange {
                        start: 0,
                        end: size - 1,
                        len: size,
                        size,
                        body,
                    }
                }
            };

            // If it is a HEAD request, we don't need to send the body,
            // but we still need to send the Content-Length header
            if method == "HEAD" {
                let mut response = warp::reply::Response::new(Body::empty());
                let mut header_map = HeaderMap::new();
                header_map.insert("Content-Length", HeaderValue::from(object_range.len));
                let headers = response.headers_mut();
                headers.extend(header_map);
                return Ok(response);
            }

            let mut response = warp::reply::Response::new(object_range.body);
            let mut header_map = HeaderMap::new();
            if object_range.len < object_range.size {
                *response.status_mut() = StatusCode::PARTIAL_CONTENT;
                header_map.insert(
                    "Content-Range",
                    HeaderValue::from_str(&format!(
                        "bytes {}-{}/{}",
                        object_range.start, object_range.end, object_range.size
                    ))
                    .unwrap(),
                );
            } else {
                header_map.insert("Accept-Ranges", HeaderValue::from_str("bytes").unwrap());
            }
            header_map.insert("Content-Length", HeaderValue::from(object_range.len));

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
            COUNTER_BYTES_DOWNLOADED.inc_by(object_range.len);
            HISTOGRAM_DOWNLOAD_TIME.observe(start_time.elapsed().as_secs_f64());

            Ok(response)
        }
        None => Err(Rejection::from(NotFound)),
    }
}

/// Handle direct blob download by querying the blobs actor.
async fn handle_blob_download<F: QueryClient + Send + Sync>(
    blob_hash_str: String,
    method: String,
    range: Option<String>,
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
    let maybe_blob = blob_get(client, blob_hash, height).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("blobs actor query error: {}", e),
        })
    })?;

    match maybe_blob {
        Some(blob) => {
            // The blob hash from blobs actor is the hash sequence hash
            // We need to parse it to get the original content hash
            let hash_seq_hash = Hash::from_bytes(blob_hash.0);
            let size = blob.size;

            debug!(
                "blob download: hash_seq_hash={}, size={}",
                hash_seq_hash, size
            );

            // Read the hash sequence to get the original content hash
            let hash_seq_bytes = iroh.read_to_bytes(hash_seq_hash).await.map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to read hash sequence: {} {}", hash_seq_hash, e),
                })
            })?;

            let hash_seq = HashSeq::try_from(hash_seq_bytes).map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to parse hash sequence: {}", e),
                })
            })?;

            // First hash in the sequence is the original content
            let orig_hash = hash_seq.iter().next().ok_or_else(|| {
                Rejection::from(BadRequest {
                    message: "hash sequence is empty".to_string(),
                })
            })?;

            debug!("parsed orig_hash from hash sequence: {}", orig_hash);

            let object_range = match range {
                Some(range) => {
                    let (first_byte, last_byte) = get_range_params(range, size).map_err(|e| {
                        Rejection::from(BadRequest {
                            message: e.to_string(),
                        })
                    })?;
                    let len = (last_byte - first_byte) + 1;

                    // Use read_at for range requests on the original content
                    use iroh_blobs::rpc::client::blobs::ReadAtLen;
                    let read_len = ReadAtLen::AtMost(len);
                    let bytes = iroh
                        .read_at_to_bytes(orig_hash, first_byte, read_len)
                        .await
                        .map_err(|e| {
                            Rejection::from(BadRequest {
                                message: format!(
                                    "failed to read blob at range: {} {}",
                                    orig_hash, e
                                ),
                            })
                        })?;

                    let body = Body::from(bytes);
                    ObjectRange {
                        start: first_byte,
                        end: last_byte,
                        len,
                        size,
                        body,
                    }
                }
                None => {
                    // Read the entire original content blob directly from Iroh
                    debug!("reading original content with hash: {}", orig_hash);

                    let reader = iroh.read(orig_hash).await.map_err(|e| {
                        Rejection::from(BadRequest {
                            message: format!("failed to read blob: {} {}", orig_hash, e),
                        })
                    })?;

                    let bytes_stream = reader.map(move |chunk_result: Result<bytes::Bytes, _>| {
                        chunk_result.map_err(|e: std::io::Error| anyhow::anyhow!(e))
                    });

                    let body = Body::wrap_stream(bytes_stream);
                    ObjectRange {
                        start: 0,
                        end: size - 1,
                        len: size,
                        size,
                        body,
                    }
                }
            };

            // If it is a HEAD request, we don't need to send the body
            if method == "HEAD" {
                let mut response = warp::reply::Response::new(Body::empty());
                let mut header_map = HeaderMap::new();
                header_map.insert("Content-Length", HeaderValue::from(object_range.len));
                let headers = response.headers_mut();
                headers.extend(header_map);
                return Ok(response);
            }

            let mut response = warp::reply::Response::new(object_range.body);
            let mut header_map = HeaderMap::new();
            if object_range.len < object_range.size {
                *response.status_mut() = StatusCode::PARTIAL_CONTENT;
                header_map.insert(
                    "Content-Range",
                    HeaderValue::from_str(&format!(
                        "bytes {}-{}/{}",
                        object_range.start, object_range.end, object_range.size
                    ))
                    .unwrap(),
                );
            } else {
                header_map.insert("Accept-Ranges", HeaderValue::from_str("bytes").unwrap());
            }
            header_map.insert("Content-Length", HeaderValue::from(object_range.len));
            header_map.insert(
                "Content-Type",
                HeaderValue::from_str("application/octet-stream").unwrap(),
            );

            let headers = response.headers_mut();
            headers.extend(header_map);

            COUNTER_BLOBS_DOWNLOADED.inc();
            COUNTER_BYTES_DOWNLOADED.inc_by(object_range.len);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_range_params() {
        // bad formats
        let _ = get_range_params("bytes=0,50".into(), 100).is_err();
        let _ = get_range_params("bytes=-0-50".into(), 100).is_err();
        let _ = get_range_params("bytes=-50-".into(), 100).is_err();
        // first > last
        let _ = get_range_params("bytes=50-0".into(), 100).is_err();
        // first >= size
        let _ = get_range_params("bytes=100-".into(), 100).is_err();
        // first == last
        let (first, last) = get_range_params("bytes=0-0".into(), 100).unwrap();
        assert_eq!(first, 0);
        assert_eq!(last, 0);
        // exact range given
        let (first, last) = get_range_params("bytes=0-50".into(), 100).unwrap();
        assert_eq!(first, 0);
        assert_eq!(last, 50);
        // only end given, this means "give me last 50 bytes"
        let (first, last) = get_range_params("bytes=-50".into(), 100).unwrap();
        assert_eq!(first, 50);
        assert_eq!(last, 99);
        // only start given, this means "give me everything but the first 50 bytes"
        let (first, last) = get_range_params("bytes=50-".into(), 100).unwrap();
        assert_eq!(first, 50);
        assert_eq!(last, 99);
        // neither given, this means "give me everything"
        let (first, last) = get_range_params("bytes=-".into(), 100).unwrap();
        assert_eq!(first, 0);
        assert_eq!(last, 99);
        // last >= size
        let (first, last) = get_range_params("bytes=50-100".into(), 100).unwrap();
        assert_eq!(first, 50);
        assert_eq!(last, 99);
    }
}
