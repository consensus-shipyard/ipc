// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::str::FromStr;
use std::time::Instant;
use std::{convert::Infallible, net::ToSocketAddrs, num::ParseIntError};

use anyhow::anyhow;
use anyhow::Context;
use bytes::Buf;
use entangler::{ChunkRange, Config, Entangler};
use entangler_storage::iroh::IrohStorage as EntanglerIrohStorage;
use fendermint_actor_bucket::{GetParams, Object};
use fendermint_app_settings::objects::ObjectsSettings;
use fendermint_rpc::{client::FendermintClient, message::GasParams, QueryClient};
use fendermint_vm_message::query::FvmQueryHeight;
use futures_util::{StreamExt, TryStreamExt};
use fvm_shared::{
    address::{Address, Error as NetworkError, Network},
    econ::TokenAmount,
};
use ipc_api::ethers_address_to_fil_address;
use iroh::{
    blobs::{provider::AddProgress, util::SetTagOption, Hash},
    client::blobs::BlobStatus,
    net::NodeAddr,
};
use lazy_static::lazy_static;
use num_traits::Zero;
use prometheus::{register_histogram, register_int_counter, Histogram, IntCounter};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;
use warp::{
    filters::multipart::Part,
    http::{HeaderMap, HeaderValue, StatusCode},
    hyper::body::Body,
    path::Tail,
    Filter, Rejection, Reply,
};

use crate::cmd;
use crate::options::objects::{ObjectsArgs, ObjectsCommands};

/// The alpha parameter for alpha entanglement determines the number of parity blobs to generate
/// for the original blob.
const ENTANGLER_ALPHA: u8 = 3;
/// The s parameter for alpha entanglement determines the number of horizontal strands in the grid.
const ENTANGLER_S: u8 = 5;
/// The p parameter for alpha entanglement determines the number of helical strands in the grid.
const ENTANGLER_P: u8 = 5;
/// Chunk size used by the entangler.
const CHUNK_SIZE: u64 = 1024;

cmd! {
    ObjectsArgs(self, settings: ObjectsSettings) {
        match self.command.clone() {
            ObjectsCommands::Run { tendermint_url, iroh_addr} => {
                if settings.metrics.enabled {
                    info!(
                        listen_addr = settings.metrics.listen.to_string(),
                        "serving metrics"
                    );
                    let builder = prometheus_exporter::Builder::new(settings.metrics.listen.try_into()?);
                    let _ = builder.start().context("failed to start metrics server")?;
                } else {
                    info!("metrics disabled");
                }

                let client = FendermintClient::new_http(tendermint_url, None)?;

                let iroh_addr = iroh_addr
                    .to_socket_addrs()?
                    .next()
                    .ok_or(anyhow!("failed to convert iroh_addr to a socket address"))?;
                let iroh_client = iroh::client::Iroh::connect_addr(iroh_addr).await?;

                // Admin routes
                let health = warp::path!("health")
                    .and(warp::get()).and_then(handle_health);
                let node_addr = warp::path!("v1" / "node" )
                .and(warp::get())
                .and(with_iroh(iroh_client.clone()))
                .and_then(handle_node_addr);

                // Objects routes
                let objects_upload = warp::path!("v1" / "objects" )
                .and(warp::post())
                .and(with_iroh(iroh_client.clone()))
                .and(warp::multipart::form().max_length(settings.max_object_size + 1024 * 1024)) // max_object_size + 1MB for form overhead
                .and(with_max_size(settings.max_object_size))
                .and_then(handle_object_upload);

                let objects_download = warp::path!("v1" / "objects" / String / ..)
                .and(warp::path::tail())
                .and(
                    warp::get().map(|| "GET".to_string()).or(warp::head().map(|| "HEAD".to_string())).unify()
                )
                .and(warp::header::optional::<String>("Range"))
                .and(warp::query::<HeightQuery>())
                .and(with_client(client.clone()))
                .and(with_iroh(iroh_client.clone()))
                .and_then(handle_object_download);

                let router = health
                    .or(node_addr)
                    .or(objects_upload)
                    .or(objects_download)
                    .with(warp::cors().allow_any_origin()
                        .allow_headers(vec!["Content-Type"])
                        .allow_methods(vec!["POST", "DEL", "GET", "HEAD"]))
                    .recover(handle_rejection);

                if let Some(listen_addr) = settings.listen.to_socket_addrs()?.next() {
                    warp::serve(router).run(listen_addr).await;
                    Ok(())
                } else {
                    Err(anyhow!("failed to convert to a socket address"))
                }
            },
        }
    }
}

fn with_client(
    client: FendermintClient,
) -> impl Filter<Extract = (FendermintClient,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_iroh(
    client: iroh::client::Iroh,
) -> impl Filter<Extract = (iroh::client::Iroh,), Error = Infallible> + Clone {
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
                    // Just read and discard the data
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

async fn handle_node_addr(iroh: iroh::client::Iroh) -> Result<impl Reply, Rejection> {
    let node_addr = iroh.net().node_addr().await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to get iroh node address info: {}", e),
        })
    })?;
    Ok(warp::reply::json(&node_addr))
}

#[derive(Serialize)]
struct UploadResponse {
    hash: String,
    metadata_hash: String,
}

async fn handle_object_upload(
    iroh: iroh::client::Iroh,
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
            let tag = iroh::blobs::Tag(format!("temp-{hash}").into());
            let progress = iroh
                .blobs()
                .download_with_opts(
                    hash,
                    iroh::client::blobs::DownloadOptions {
                        format: iroh::blobs::BlobFormat::Raw,
                        nodes: vec![source],
                        tag: SetTagOption::Named(tag),
                        mode: iroh::client::blobs::DownloadMode::Queued,
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

            info!(
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

            let mut progress = iroh
                .blobs()
                .add_stream(stream, SetTagOption::Auto)
                .await
                .map_err(|e| {
                    Rejection::from(BadRequest {
                        message: format!("failed to store blob: {}", e),
                    })
                })?;

            let uploaded_hash = loop {
                let Some(event) = progress.next().await else {
                    return Err(Rejection::from(BadRequest {
                        message: "Unexpected end while ingesting data".to_string(),
                    }));
                };
                match event.map_err(|e| {
                    Rejection::from(BadRequest {
                        message: format!("failed to make progress: {}", e),
                    })
                })? {
                    AddProgress::AllDone { hash, .. } => {
                        break hash;
                    }
                    AddProgress::Abort(err) => {
                        return Err(Rejection::from(BadRequest {
                            message: format!("upload aborted: {}", err),
                        }));
                    }
                    _ => continue,
                }
            };
            info!("stored uploaded blob {} (size: {})", uploaded_hash, size);
            COUNTER_BYTES_UPLOADED.inc_by(size);

            uploaded_hash
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

    let ent = new_entangler(iroh).map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to create entangler: {}", e),
        })
    })?;
    let metadata_hash = ent.entangle_uploaded(hash.to_string()).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to entangle uploaded data: {}", e),
        })
    })?;

    COUNTER_BLOBS_UPLOADED.inc();
    HISTOGRAM_UPLOAD_TIME.observe(start_time.elapsed().as_secs_f64());

    let response = UploadResponse {
        hash: hash.to_string(),
        metadata_hash,
    };
    Ok(warp::reply::json(&response))
}

fn new_entangler(
    iroh: iroh::client::Iroh,
) -> Result<Entangler<EntanglerIrohStorage>, entangler::Error> {
    Entangler::new(
        EntanglerIrohStorage::from_client(iroh),
        Config::new(ENTANGLER_ALPHA, ENTANGLER_S, ENTANGLER_P),
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

pub(crate) struct ObjectRange {
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
    iroh: iroh::client::Iroh,
) -> Result<impl Reply, Rejection> {
    let address = parse_address(&address).map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("invalid address {}: {}", address, e),
        })
    })?;
    let height = height_query
        .height
        .unwrap_or(FvmQueryHeight::Committed.into());
    let path = tail.as_str();
    let key: Vec<u8> = path.into();
    let start_time = Instant::now();
    let maybe_object = os_get(client, address, GetParams(key), height)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("bucket get error: {}", e),
            })
        })?;

    match maybe_object {
        Some(object) => {
            let hash = Hash::from_bytes(object.hash.0);
            let status = iroh.blobs().status(hash).await.map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to read object: {} {}", hash, e),
                })
            })?;
            let BlobStatus::Complete { size } = status else {
                return Err(Rejection::from(BadRequest {
                    message: format!("object {} is not available", hash),
                }));
            };
            // Sanity check size
            if size.is_zero() {
                return Err(Rejection::from(BadRequest {
                    message: format!("object {} has zero size", hash),
                }));
            }

            let ent = new_entangler(iroh).map_err(|e| {
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
            header_map.insert(
                "Content-Type",
                HeaderValue::from_str(
                    object
                        .metadata
                        .get("content-type")
                        .unwrap_or(&"application/octet-stream".to_string()),
                )
                .unwrap(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use fendermint_rpc::FendermintClient;
    use rand_chacha::rand_core::{RngCore, SeedableRng};
    use rand_chacha::ChaCha8Rng;
    use tendermint_rpc::{Method, MockClient, MockRequestMethodMatcher};

    // Used to mocking Actor State
    const ABCI_QUERY_RESPONSE_DOWNLOAD: &str = r#"{
        "jsonrpc": "2.0",
        "id": "",
        "result": {
         "response": {
            "code": 0,
            "log": "",
            "info": "",
            "index": "0",
            "key": "",
            "value": "mQEIEhizARiFGJgYIBgYGNcYGBhJGBgYgRgYGO8YGBinCgwYGBiICxgYGI0YGBiMGBgYGRgYGIUYGBjQGBgYdRgYGNsYGBjLGBgY9hgYGHkYGBi5GBgYmhgYGF8YGBiZFBgYGOUYGBiqGBgY+RgYGGsYGBiDGBgYGhgYGJ4YGBgkGJgYIBgYGOwYGBhRGBgYoBgYGNEYGBhyGBgY2RgYGOcYGBhlGBgYpxgYGDMYGBjKExgYGOsYGBgYGBgY8hgYGN0YGBjNGBgYbRgYGMwYGBhOGBgYKhgYGPcYGBgoGBgYixgYGBgYGBjJGBgYXRgYGDQYGBiJABgYGIcYGBj3CxgZFRg2GKIYYxhmGG8YbxhjGGIYYRhyGGwYYxhvGG4YdBhlGG4YdBgtGHQYeRhwGGUYeBgYGGEYcBhwGGwYaRhjGGEYdBhpGG8YbhgvGG8YYxh0GGUYdBgtGHMYdBhyGGUYYRhtGDAYzBjgGL8CGDoYSwoHGG0YZRhzGHMYYRhnGGUSDQoEGGYYchhvGG0SAxh0GDAYMBgYARIYMQoCGHQYbxIYKRh0GDIYZxhvGGMYcBhuGGwYcRhpGGwYeBh5GG0Ydhh3GDQYdhhuGGwYaBg0GG4YcBhuGGQYeRh4GGoYYxhsGDMYMxh5GGgYdRhjGHQYaRhjGGkYGAE=",
            "proof": null,
            "height": "6017",
            "codespace": ""
            }
        }
     }"#;

    fn setup_logs() {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        use tracing_subscriber::EnvFilter;

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(tracing_subscriber::fmt::format().with_line_number(true))
                    .with_writer(std::io::stdout),
            )
            .with(EnvFilter::from_default_env())
            .try_init()
            .ok();
    }

    #[tokio::test]
    async fn test_handle_object_upload() {
        setup_logs();

        let iroh = iroh::node::Node::memory().spawn().await.unwrap();
        // client iroh node
        let client_iroh = iroh::node::Node::memory().spawn().await.unwrap();
        let hash = client_iroh
            .blobs()
            .add_bytes(&b"hello world"[..])
            .await
            .unwrap()
            .hash;
        let client_node_addr = client_iroh.net().node_addr().await.unwrap();
        let size = 11;

        // Create multipart form for source-based upload
        let boundary = "--abcdef1234--";
        let mut body = Vec::new();
        let form_data = format!(
            "\
            --{0}\r\n\
            content-disposition: form-data; name=\"hash\"\r\n\r\n\
            {1}\r\n\
            --{0}\r\n\
            content-disposition: form-data; name=\"size\"\r\n\r\n\
            {2}\r\n\
            --{0}\r\n\
            content-disposition: form-data; name=\"source\"\r\n\r\n\
            {3}\r\n\
            --{0}--\r\n\
            ",
            boundary,
            hash,
            size,
            serde_json::to_string(&client_node_addr).unwrap(),
        );
        body.extend_from_slice(form_data.as_bytes());

        let form_data = warp::test::request()
            .method("POST")
            .header("content-length", body.len())
            .header(
                "content-type",
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(body)
            .filter(&warp::multipart::form())
            .await
            .unwrap();

        let reply = handle_object_upload(iroh.client().clone(), form_data, 1000)
            .await
            .unwrap();
        let response = reply.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_object_upload_direct() {
        setup_logs();

        let iroh = iroh::node::Node::memory().spawn().await.unwrap();

        // Create a 10MB random file
        const FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let mut test_data = vec![0u8; FILE_SIZE];
        rng.fill_bytes(&mut test_data);

        let size = test_data.len() as u64;
        let hash = Hash::new(&test_data);

        // Create multipart form with direct data upload
        let boundary = "------------------------abcdef1234567890"; // Use a longer boundary
        let mut body = Vec::with_capacity(FILE_SIZE + 1024); // Pre-allocate with some extra space for headers

        // Write form fields
        body.extend_from_slice(
            format!(
                "\
            --{boundary}\r\n\
            Content-Disposition: form-data; name=\"hash\"\r\n\r\n\
            {hash}\r\n\
            --{boundary}\r\n\
            Content-Disposition: form-data; name=\"size\"\r\n\r\n\
            {size}\r\n\
            --{boundary}\r\n\
            Content-Disposition: form-data; name=\"data\"\r\n\
            Content-Type: application/octet-stream\r\n\r\n",
            )
            .as_bytes(),
        );

        // Write file data
        body.extend_from_slice(&test_data);

        // Write final boundary
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let form_data = warp::test::request()
            .method("POST")
            .header("content-length", body.len())
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body)
            .filter(&warp::multipart::form().max_length(11 * 1024 * 1024))
            .await
            .unwrap();

        // Test with a larger max_size to accommodate our test file
        let reply = handle_object_upload(iroh.client().clone(), form_data, FILE_SIZE as u64 * 2)
            .await
            .unwrap();
        let response = reply.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        // Verify the blob was stored in iroh
        let status = iroh.blobs().status(hash).await.unwrap();
        match status {
            BlobStatus::Complete { size: stored_size } => {
                assert_eq!(stored_size, size);
            }
            _ => panic!("Expected blob to be stored completely"),
        }
    }

    #[tokio::test]
    async fn test_handle_object_download_get() {
        let matcher = MockRequestMethodMatcher::default().map(
            Method::AbciQuery,
            Ok(ABCI_QUERY_RESPONSE_DOWNLOAD.to_string()),
        );
        let client = FendermintClient::new(MockClient::new(matcher).0);
        let iroh = iroh::node::Node::memory().spawn().await.unwrap();
        let _hash = iroh
            .blobs()
            .add_bytes(&b"hello world"[..])
            .await
            .unwrap()
            .hash;

        let result = handle_object_download(
            "t2mnd5jkuvmsaf457ympnf3monalh3vothdd5njoy".into(),
            warp::test::request()
                .path("/foo/bar")
                .filter(&warp::path::tail())
                .await
                .unwrap(),
            "GET".to_string(),
            None,
            HeightQuery { height: Some(1) },
            client,
            iroh.client().clone(),
        )
        .await;
        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("Content-Type")
                .unwrap()
                .to_str()
                .unwrap(),
            "application/octet-stream"
        );
        let body = warp::hyper::body::to_bytes(response.into_body())
            .await
            .unwrap();
        assert_eq!(body, "hello world".as_bytes());
    }

    #[tokio::test]
    async fn test_handle_object_download_with_range() {
        let matcher = MockRequestMethodMatcher::default().map(
            Method::AbciQuery,
            Ok(ABCI_QUERY_RESPONSE_DOWNLOAD.to_string()),
        );
        let client = FendermintClient::new(MockClient::new(matcher).0);
        let iroh = iroh::node::Node::memory().spawn().await.unwrap();
        let _hash = iroh
            .blobs()
            .add_bytes(&b"hello world"[..])
            .await
            .unwrap()
            .hash;

        let result = handle_object_download(
            "t2mnd5jkuvmsaf457ympnf3monalh3vothdd5njoy".into(),
            warp::test::request()
                .path("/foo/bar")
                .filter(&warp::path::tail())
                .await
                .unwrap(),
            "GET".to_string(),
            Some("bytes=0-4".to_string()),
            HeightQuery { height: Some(1) },
            client,
            iroh.client().clone(),
        )
        .await;
        assert!(result.is_ok(), "{:#?}", result.err());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        let body = warp::hyper::body::to_bytes(response.into_body())
            .await
            .unwrap();
        assert_eq!(body, "hello".as_bytes());
    }

    #[tokio::test]
    async fn test_handle_object_download_head() {
        let matcher = MockRequestMethodMatcher::default().map(
            Method::AbciQuery,
            Ok(ABCI_QUERY_RESPONSE_DOWNLOAD.to_string()),
        );
        let client = FendermintClient::new(MockClient::new(matcher).0);
        let iroh = iroh::node::Node::memory().spawn().await.unwrap();
        let _hash = iroh
            .blobs()
            .add_bytes(&b"hello world"[..])
            .await
            .unwrap()
            .hash;

        let result = handle_object_download(
            "t2mnd5jkuvmsaf457ympnf3monalh3vothdd5njoy".into(),
            warp::test::request()
                .path("/foo/bar")
                .filter(&warp::path::tail())
                .await
                .unwrap(),
            "HEAD".to_string(),
            None,
            HeightQuery { height: Some(1) },
            client,
            iroh.client().clone(),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("Content-Length").unwrap(), "11");
    }

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
