// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! HTTP client for storage gateway API
//!
//! This module provides functions to interact with the storage gateway's HTTP API.

use anyhow::{anyhow, Context, Result};
use fvm_shared::address::Address;
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Response from uploading a blob to the gateway
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    /// Original blob content hash
    pub hash: String,
    /// Number of chunks the data was split into
    pub num_chunks: usize,
    /// Number of data shards per chunk (k)
    pub data_shards: usize,
    /// Number of parity shards per chunk (m)
    pub parity_shards: usize,
    /// Original data length in bytes
    pub original_len: usize,
}

/// Node address information from the gateway
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub relay_url: Option<String>,
    pub direct_addresses: Vec<String>,
}

/// Storage gateway HTTP client
pub struct GatewayClient {
    base_url: String,
    client: reqwest::Client,
}

impl GatewayClient {
    /// Create a new gateway client
    pub fn new(base_url: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minute timeout for large uploads
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { base_url, client })
    }

    /// Upload a blob to the gateway
    ///
    /// This uploads the data to Iroh, erasure-encodes it, and distributes shards
    /// to the assigned storage nodes.
    pub async fn upload_blob(&self, data: Vec<u8>) -> Result<UploadResponse> {
        let size = data.len() as u64;

        // Build multipart form
        let form = Form::new()
            .text("size", size.to_string())
            .part(
                "data",
                Part::bytes(data)
                    .file_name("upload")
                    .mime_str("application/octet-stream")?,
            );

        let url = format!("{}/v1/objects", self.base_url);

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .context("Failed to send upload request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Upload failed with status {}: {}",
                status,
                body
            ));
        }

        let upload_response: UploadResponse = response
            .json()
            .await
            .context("Failed to parse upload response")?;

        Ok(upload_response)
    }

    /// Download a blob directly by its hash
    ///
    /// This retrieves the blob from storage nodes, RS-decodes it, and returns
    /// the plaintext data.
    pub async fn download_blob(&self, blob_hash: &str, height: Option<u64>) -> Result<Vec<u8>> {
        // Remove 0x prefix if present
        let hash_hex = blob_hash.strip_prefix("0x").unwrap_or(blob_hash);

        let mut url = format!("{}/v1/blobs/{}", self.base_url, hash_hex);
        if let Some(h) = height {
            url.push_str(&format!("?height={}", h));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send download request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Download failed with status {}: {}",
                status,
                body
            ));
        }

        let data = response
            .bytes()
            .await
            .context("Failed to read download response")?;

        Ok(data.to_vec())
    }

    /// Download an object from a bucket by its key/path
    ///
    /// This queries the bucket contract for the object metadata, then retrieves
    /// the blob data.
    pub async fn download_object(
        &self,
        bucket_address: &Address,
        key: &str,
        height: Option<u64>,
    ) -> Result<Vec<u8>> {
        let bucket_str = bucket_address.to_string();
        let encoded_key = urlencoding::encode(key);

        let mut url = format!("{}/v1/objects/{}/{}", self.base_url, bucket_str, encoded_key);
        if let Some(h) = height {
            url.push_str(&format!("?height={}", h));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send download request")?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(anyhow!("Object not found: {}", key));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Download failed with status {}: {}",
                status,
                body
            ));
        }

        let data = response
            .bytes()
            .await
            .context("Failed to read download response")?;

        Ok(data.to_vec())
    }

    /// Get the gateway node information
    pub async fn get_node_info(&self) -> Result<NodeInfo> {
        let url = format!("{}/v1/node", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send node info request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Node info request failed with status {}: {}",
                status,
                body
            ));
        }

        let node_info: NodeInfo = response
            .json()
            .await
            .context("Failed to parse node info response")?;

        Ok(node_info)
    }

    /// Check if the gateway is healthy and reachable
    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/health", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send health check request")?;

        if !response.status().is_success() {
            return Err(anyhow!("Gateway health check failed"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GatewayClient::new("http://localhost:8080".to_string());
        assert!(client.is_ok());
    }
}
