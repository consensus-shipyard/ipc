// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! RPC server for the decentralized storage node
//!
//! Provides HTTP endpoints for:
//! - Signature queries
//! - Blob metadata queries
//! - Blob content retrieval

use std::convert::Infallible;
use std::net::SocketAddr;

use anyhow::Result;
use fendermint_actor_blobs_shared::bytes::B256;
use fendermint_rpc::message::GasParams;
use fendermint_rpc::QueryClient;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_shared::econ::TokenAmount;
use iroh_blobs::Hash;
use iroh_manager::IrohNode;
use tracing::info;
use warp::Filter;

use super::{SharedFendermintClient, SignatureStorage};

/// Start the RPC server for signature queries and blob queries
pub async fn start_rpc_server(
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
) -> impl Filter<Extract = (SignatureStorage,), Error = Infallible> + Clone {
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
    use std::str::FromStr;

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
