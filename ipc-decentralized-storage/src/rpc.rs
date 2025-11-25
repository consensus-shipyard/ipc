// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! JSON-RPC server for signature collection
//!
//! This module provides a JSON-RPC 2.0 server that validators use to submit
//! their signatures for blob finalization.

use anyhow::{Context, Result};
use iroh_blobs::Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

/// Parse a hex-encoded hash string into an iroh Hash
fn parse_hash(hex_str: &str) -> Result<Hash> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(hex_str).context("invalid hex string")?;
    if bytes.len() != 32 {
        anyhow::bail!("hash must be 32 bytes, got {}", bytes.len());
    }
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);
    Ok(Hash::from_bytes(array))
}

/// A signature submission from a validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobSignature {
    /// The blob hash being signed
    pub blob_hash: String,
    /// The validator's address
    pub validator_address: String,
    /// The signature bytes (hex encoded)
    pub signature: String,
    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    pub fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    pub fn invalid_request() -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        }
    }

    pub fn method_not_found() -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }
    }

    pub fn invalid_params(msg: String) -> Self {
        Self {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(serde_json::json!({ "detail": msg })),
        }
    }

    pub fn internal_error(msg: String) -> Self {
        Self {
            code: -32603,
            message: "Internal error".to_string(),
            data: Some(serde_json::json!({ "detail": msg })),
        }
    }
}

/// In-memory signature store
/// TODO: Replace with persistent storage and proper validation
#[derive(Clone)]
pub struct SignatureStore {
    signatures: Arc<RwLock<HashMap<Hash, Vec<BlobSignature>>>>,
}

impl SignatureStore {
    pub fn new() -> Self {
        Self {
            signatures: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a signature to the store
    pub async fn add_signature(&self, sig: BlobSignature) -> Result<()> {
        let hash = parse_hash(&sig.blob_hash)?;
        let mut store = self.signatures.write().await;
        store.entry(hash).or_insert_with(Vec::new).push(sig);
        Ok(())
    }

    /// Get all signatures for a blob
    pub async fn get_signatures(&self, blob_hash: &Hash) -> Vec<BlobSignature> {
        let store = self.signatures.read().await;
        store.get(blob_hash).cloned().unwrap_or_default()
    }

    /// Get signature count for a blob
    pub async fn signature_count(&self, blob_hash: &Hash) -> usize {
        let store = self.signatures.read().await;
        store.get(blob_hash).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for SignatureStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Response for submit_signature method
#[derive(Debug, Serialize)]
pub struct SubmitSignatureResponse {
    /// Whether the signature was accepted
    pub accepted: bool,
    /// Total number of signatures collected for this blob
    pub signature_count: usize,
    /// Message (e.g., reason for rejection)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Response for get_signatures method
#[derive(Debug, Serialize)]
pub struct GetSignaturesResponse {
    /// The blob hash
    pub blob_hash: String,
    /// List of signatures
    pub signatures: Vec<BlobSignature>,
    /// Total count
    pub count: usize,
}

/// Handle a JSON-RPC request
async fn handle_rpc_request(
    req: JsonRpcRequest,
    store: SignatureStore,
) -> JsonRpcResponse {
    let id = req.id.clone();

    // Validate JSON-RPC version
    if req.jsonrpc != "2.0" {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError::invalid_request()),
            id,
        };
    }

    // Route to the appropriate method handler
    match req.method.as_str() {
        "submit_signature" => handle_submit_signature(req.params, store, id).await,
        "get_signatures" => handle_get_signatures(req.params, store, id).await,
        "signature_count" => handle_signature_count(req.params, store, id).await,
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError::method_not_found()),
            id,
        },
    }
}

/// Handle submit_signature method
async fn handle_submit_signature(
    params: serde_json::Value,
    store: SignatureStore,
    id: serde_json::Value,
) -> JsonRpcResponse {
    // Parse parameters
    let signature: BlobSignature = match serde_json::from_value(params) {
        Ok(sig) => sig,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::invalid_params(e.to_string())),
                id,
            }
        }
    };

    // Validate blob hash format
    let hash = match parse_hash(&signature.blob_hash) {
        Ok(h) => h,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::invalid_params(format!(
                    "Invalid blob hash: {}",
                    e
                ))),
                id,
            }
        }
    };

    // TODO: Validate signature cryptographically
    // TODO: Check if validator is authorized
    // TODO: Check if blob exists and is in the correct state

    // Store the signature
    match store.add_signature(signature.clone()).await {
        Ok(()) => {
            let count = store.signature_count(&hash).await;

            let response = SubmitSignatureResponse {
                accepted: true,
                signature_count: count,
                message: Some("Signature accepted".to_string()),
            };

            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(serde_json::to_value(response).unwrap()),
                error: None,
                id,
            }
        }
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError::internal_error(e.to_string())),
            id,
        },
    }
}

/// Handle get_signatures method
async fn handle_get_signatures(
    params: serde_json::Value,
    store: SignatureStore,
    id: serde_json::Value,
) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct GetSignaturesParams {
        blob_hash: String,
    }

    let params: GetSignaturesParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::invalid_params(e.to_string())),
                id,
            }
        }
    };

    let hash = match parse_hash(&params.blob_hash) {
        Ok(h) => h,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::invalid_params(format!(
                    "Invalid blob hash: {}",
                    e
                ))),
                id,
            }
        }
    };

    let signatures = store.get_signatures(&hash).await;
    let count = signatures.len();

    let response = GetSignaturesResponse {
        blob_hash: params.blob_hash,
        signatures,
        count,
    };

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(serde_json::to_value(response).unwrap()),
        error: None,
        id,
    }
}

/// Handle signature_count method
async fn handle_signature_count(
    params: serde_json::Value,
    store: SignatureStore,
    id: serde_json::Value,
) -> JsonRpcResponse {
    #[derive(Deserialize)]
    struct SignatureCountParams {
        blob_hash: String,
    }

    let params: SignatureCountParams = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::invalid_params(e.to_string())),
                id,
            }
        }
    };

    let hash = match parse_hash(&params.blob_hash) {
        Ok(h) => h,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::invalid_params(format!(
                    "Invalid blob hash: {}",
                    e
                ))),
                id,
            }
        }
    };

    let count = store.signature_count(&hash).await;

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(serde_json::json!({ "count": count })),
        error: None,
        id,
    }
}

/// Start the JSON-RPC server
pub async fn start_rpc_server(addr: SocketAddr, store: SignatureStore) -> Result<()> {
    let store_filter = warp::any().map(move || store.clone());

    let rpc = warp::post()
        .and(warp::path("rpc"))
        .and(warp::body::json())
        .and(store_filter)
        .and_then(
            |req: JsonRpcRequest, store: SignatureStore| async move {
                Ok::<_, warp::Rejection>(warp::reply::json(&handle_rpc_request(req, store).await))
            },
        );

    let health = warp::get()
        .and(warp::path("health"))
        .map(|| warp::reply::json(&serde_json::json!({ "status": "ok" })));

    let routes = rpc.or(health).with(
        warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["POST", "GET"])
            .allow_headers(vec!["Content-Type"]),
    );

    tracing::info!("Starting JSON-RPC server on {}", addr);
    warp::serve(routes).run(addr).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_store() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let store = SignatureStore::new();
            let sig = BlobSignature {
                blob_hash: "0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
                validator_address: "t01234".to_string(),
                signature: "deadbeef".to_string(),
                metadata: HashMap::new(),
            };

            store.add_signature(sig.clone()).await.unwrap();
            let hash = parse_hash(&sig.blob_hash).unwrap();
            assert_eq!(store.signature_count(&hash).await, 1);

            let sigs = store.get_signatures(&hash).await;
            assert_eq!(sigs.len(), 1);
            assert_eq!(sigs[0].validator_address, "t01234");
        });
    }
}
