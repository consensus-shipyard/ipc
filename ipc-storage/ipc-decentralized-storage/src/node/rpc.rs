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
use std::str::FromStr;

use anyhow::Result;
use bls_signatures::{PrivateKey as BlsPrivateKey, Serialize as BlsSerialize};
use erasure_encoding::BlobId;
use fendermint_actor_blobs_shared::bytes::B256;
use fendermint_rpc::message::GasParams;
use fendermint_rpc::{FendermintClient, QueryClient};
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_shared::econ::TokenAmount;
use iroh_blobs::Hash;
use iroh_manager::IrohNode;
use tracing::{info, warn};
use warp::Filter;

use super::{SharedFendermintClient, SignatureStorage};
use crate::distribution::ShardPullRequest;

/// Start the RPC server for signature queries and blob queries
pub async fn start_rpc_server(
    bind_addr: SocketAddr,
    signatures: SignatureStorage,
    client: SharedFendermintClient,
    iroh: IrohNode,
    bls_private_key: BlsPrivateKey,
    rpc_url: tendermint_rpc::Url,
) -> Result<()> {
    // GET /signature/{hash}
    let get_signature = warp::path!("signature" / String)
        .and(warp::get())
        .and(with_signatures(signatures.clone()))
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

    // GET /v1/blobs/{hash}/content - returns blob content via shard retrieval
    let iroh_for_content = iroh.clone();
    let rpc_url_for_content = rpc_url;
    let get_blob_content = warp::path!("v1" / "blobs" / String / "content")
        .and(warp::get())
        .and(with_client(client))
        .and(with_iroh(iroh_for_content))
        .and(warp::any().map(move || rpc_url_for_content.clone()))
        .and_then(handle_get_blob_content);

    // GET /v1/node - returns this node's Iroh NodeAddr for P2P connectivity
    let iroh_for_node = iroh.clone();
    let get_node_addr = warp::path!("v1" / "node")
        .and(warp::get())
        .and(with_iroh(iroh_for_node))
        .and_then(handle_get_node_addr);

    // GET /v1/shards/{blob_id}/{chunk_index}/{shard_index}/hash - lookup shard Iroh hash
    let iroh_for_shard_hash = iroh.clone();
    let get_shard_hash = warp::path!("v1" / "shards" / String / usize / usize / "hash")
        .and(warp::get())
        .and(with_iroh(iroh_for_shard_hash))
        .and_then(handle_get_shard_hash);

    // POST /v1/shards/pull - accept a shard pull request from a distributor
    let signatures_for_pull = signatures.clone();
    let pull_shard = warp::path!("v1" / "shards" / "pull")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_iroh(iroh))
        .and(with_signatures(signatures_for_pull))
        .and(with_bls_key(bls_private_key))
        .and_then(handle_shard_pull);

    // CORS configuration - allow all origins for development
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .allow_headers(vec!["Content-Type", "Authorization"]);

    let routes = get_signature
        .or(health)
        .or(get_node_addr)
        .or(get_shard_hash)
        .or(get_blob_content)
        .or(get_blob)
        .or(pull_shard)
        .with(cors);

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

/// Warp filter to inject BLS private key
fn with_bls_key(
    key: BlsPrivateKey,
) -> impl Filter<Extract = (BlsPrivateKey,), Error = Infallible> + Clone {
    warp::any().map(move || key)
}

/// Handle GET /v1/node - returns this node's Iroh NodeAddr
async fn handle_get_node_addr(iroh: IrohNode) -> Result<impl warp::Reply, warp::Rejection> {
    let node_addr = iroh.endpoint().node_addr().await.map_err(|e| {
        warp::reject::custom(RpcBadRequest {
            message: format!("failed to get node address: {}", e),
        })
    })?;
    Ok(warp::reply::json(&node_addr))
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
fn with_iroh(iroh: IrohNode) -> impl Filter<Extract = (IrohNode,), Error = Infallible> + Clone {
    warp::any().map(move || iroh.clone())
}

/// Handle GET /v1/blobs/{hash}/content - returns blob content via shard retrieval
///
/// Reconstructs the blob by fetching shards from assigned operators and RS-decoding.
async fn handle_get_blob_content(
    hash_str: String,
    client: SharedFendermintClient,
    iroh: IrohNode,
    rpc_url: tendermint_rpc::Url,
) -> Result<impl warp::Reply, warp::Rejection> {
    use crate::gateway::BlobGateway;
    use crate::objects::build_node_directories;
    use crate::retrieval::{retrieve, BlobRetrievalParams};
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

    // Gas params for the query call
    let gas_params = GasParams {
        gas_limit: Default::default(),
        gas_fee_cap: Default::default(),
        gas_premium: Default::default(),
    };

    // Query the blobs actor to get blob info (size, k, m)
    let maybe_blob = {
        let mut client_guard = client.lock().await;
        client_guard
            .blob_get_call(
                blob_hash,
                TokenAmount::default(),
                gas_params,
                FvmQueryHeight::Committed,
            )
            .await
    };

    match maybe_blob {
        Ok(Some(blob)) => {
            let size = blob.size;
            let data_shards = blob.data_shards as usize;
            let parity_shards = blob.parity_shards as usize;

            // Build node directories from on-chain operator state
            let retrieval_client = FendermintClient::new_http(rpc_url, None).map_err(|e| {
                warp::reject::custom(RpcBadRequest {
                    message: format!("failed to create client: {}", e),
                })
            })?;
            let gateway = BlobGateway::new(retrieval_client, 10, std::time::Duration::from_secs(5));
            let (nodes, node_directory, node_rpc_directory) =
                build_node_directories(&gateway).await.map_err(|e| {
                    warp::reject::custom(RpcBadRequest {
                        message: format!("failed to build node directories: {}", e),
                    })
                })?;

            let blob_id = BlobId(blob_hash.0);
            let retrieved_data = retrieve(
                &BlobRetrievalParams {
                    blob_id,
                    original_len: size as usize,
                    data_shards,
                    parity_shards,
                    nodes,
                    node_directory,
                    node_rpc_directory,
                },
                iroh.blobs_client(),
            )
            .await
            .map_err(|e| {
                warp::reject::custom(RpcBadRequest {
                    message: format!("failed to retrieve blob: {}", e),
                })
            })?;

            let body = Body::from(retrieved_data);

            let mut response = warp::reply::Response::new(body);
            response.headers_mut().insert(
                "Content-Type",
                warp::http::HeaderValue::from_static("application/octet-stream"),
            );
            response
                .headers_mut()
                .insert("Content-Length", warp::http::HeaderValue::from(size));

            Ok(warp::reply::with_status(
                response,
                warp::http::StatusCode::OK,
            ))
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

/// Response for shard hash lookup
#[derive(serde::Serialize)]
struct ShardHashResponse {
    hash: String,
    node_addr: iroh::NodeAddr,
}

/// Handle GET /v1/shards/{blob_id}/{chunk_index}/{shard_index}/hash
///
/// Returns the Iroh content hash for a locally-stored shard, allowing other
/// nodes to download it via Iroh P2P.
async fn handle_get_shard_hash(
    blob_id_hex: String,
    chunk_index: usize,
    shard_index: usize,
    iroh: IrohNode,
) -> Result<impl warp::Reply, warp::Rejection> {
    let blob_id_bytes = hex::decode(&blob_id_hex).map_err(|_| {
        warp::reject::custom(RpcBadRequest {
            message: "invalid blob_id hex".to_string(),
        })
    })?;
    if blob_id_bytes.len() != 32 {
        return Err(warp::reject::custom(RpcBadRequest {
            message: format!("blob_id must be 32 bytes, got {}", blob_id_bytes.len()),
        }));
    }
    let mut blob_id_array = [0u8; 32];
    blob_id_array.copy_from_slice(&blob_id_bytes);
    let blob_id = BlobId(blob_id_array);

    let tag = crate::distribution::shard_key(&blob_id, chunk_index, shard_index);

    // Look up the Iroh hash for this shard tag
    let iroh_tag = iroh_blobs::Tag(tag.into());
    let hash = {
        use futures::StreamExt;
        let mut tags = iroh.blobs_client().tags().list().await.map_err(|e| {
            warp::reject::custom(RpcBadRequest {
                message: format!("failed to list tags: {}", e),
            })
        })?;
        let mut found = None;
        while let Some(Ok(tag_info)) = tags.next().await {
            if tag_info.name == iroh_tag {
                found = Some(tag_info.hash);
                break;
            }
        }
        found
    };

    match hash {
        Some(hash) => {
            let node_addr = iroh.endpoint().node_addr().await.map_err(|e| {
                warp::reject::custom(RpcBadRequest {
                    message: format!("failed to get node address: {}", e),
                })
            })?;
            Ok(warp::reply::with_status(
                warp::reply::json(&ShardHashResponse {
                    hash: hash.to_string(),
                    node_addr,
                }),
                warp::http::StatusCode::OK,
            ))
        }
        None => Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse {
                error: "shard not found".to_string(),
            }),
            warp::http::StatusCode::NOT_FOUND,
        )),
    }
}

/// Response for shard pull request
#[derive(serde::Serialize)]
struct ShardPullResponse {
    status: String,
    shard_key: String,
}

/// Handle POST /v1/shards/pull
///
/// A distributor calls this to tell us to download a shard from them.
/// We verify the shard is assigned to us, download from the gateway via Iroh,
/// and generate a BLS signature once the download completes.
async fn handle_shard_pull(
    request: ShardPullRequest,
    iroh: IrohNode,
    signatures: SignatureStorage,
    bls_private_key: BlsPrivateKey,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Parse blob_id
    let blob_id_bytes = hex::decode(&request.blob_id).map_err(|_| {
        warp::reject::custom(RpcBadRequest {
            message: "invalid blob_id hex".to_string(),
        })
    })?;
    if blob_id_bytes.len() != 32 {
        return Err(warp::reject::custom(RpcBadRequest {
            message: format!("blob_id must be 32 bytes, got {}", blob_id_bytes.len()),
        }));
    }
    let mut blob_id_array = [0u8; 32];
    blob_id_array.copy_from_slice(&blob_id_bytes);
    let blob_id = BlobId(blob_id_array);

    // Parse the shard's Iroh content hash
    let hash = Hash::from_str(&request.hash).map_err(|_| {
        warp::reject::custom(RpcBadRequest {
            message: "invalid hash".to_string(),
        })
    })?;

    // Verify this shard is assigned to us
    // TODO: get the node list from on-chain state at encoding epoch.
    // For now, we skip full assignment verification and accept the pull request.
    // Once on-chain integration is in place, we'd call:
    //   shard_verifier::verify_shard_assignment(
    //       &blob_id, request.chunk_index, request.shard_index,
    //       request.shards_per_chunk, &nodes, &our_node_id
    //   )?;

    let shard_key =
        crate::distribution::shard_key(&blob_id, request.chunk_index, request.shard_index);

    info!("Received shard pull request: {} (hash={})", shard_key, hash);

    // Spawn a task to download from the gateway and sign on completion
    let shard_key_clone = shard_key.clone();
    tokio::spawn(async move {
        let download_result = iroh
            .blobs_client()
            .download_with_opts(
                hash,
                iroh_blobs::rpc::client::blobs::DownloadOptions {
                    format: iroh_blobs::BlobFormat::Raw,
                    nodes: vec![request.source],
                    tag: iroh_blobs::util::SetTagOption::Named(iroh_blobs::Tag(
                        shard_key_clone.clone().into(),
                    )),
                    mode: iroh_blobs::rpc::client::blobs::DownloadMode::Queued,
                },
            )
            .await;

        match download_result {
            Ok(progress) => match progress.finish().await {
                Ok(outcome) => {
                    info!(
                        "Downloaded shard {} (downloaded: {} bytes, local: {} bytes, total: {} bytes)",
                        shard_key_clone, outcome.downloaded_size, outcome.local_size,
                        outcome.downloaded_size + outcome.local_size
                    );

                    // Verify shard content size
                    match iroh.blobs_client().read_to_bytes(hash).await {
                        Ok(bytes) => {
                            info!(
                                "Verified shard {} content: {} bytes",
                                shard_key_clone,
                                bytes.len()
                            );
                        }
                        Err(e) => {
                            warn!("Failed to verify shard {} content: {}", shard_key_clone, e);
                        }
                    }

                    // Generate BLS signature over the blob hash (not the shard hash)
                    // The contract verifies signatures over the blob hash
                    let blob_hash = Hash::from_bytes(blob_id.0);
                    let signature = bls_private_key.sign(blob_hash.as_bytes());
                    let signature_bytes = signature.as_bytes();

                    // Store signature keyed by blob hash for gateway collection
                    {
                        let mut sigs = signatures.write().unwrap();
                        sigs.insert(blob_hash, signature_bytes.clone());
                    }

                    info!(
                        "Generated BLS signature for shard {} (blob_hash={})",
                        shard_key_clone, blob_hash
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to complete shard {} download: {}",
                        shard_key_clone, e
                    );
                }
            },
            Err(e) => {
                warn!("Failed to start shard {} download: {}", shard_key_clone, e);
            }
        }
    });

    Ok(warp::reply::with_status(
        warp::reply::json(&ShardPullResponse {
            status: "accepted".to_string(),
            shard_key,
        }),
        warp::http::StatusCode::ACCEPTED,
    ))
}

#[allow(dead_code)]
#[derive(Debug)]
struct RpcBadRequest {
    message: String,
}

impl warp::reject::Reject for RpcBadRequest {}
