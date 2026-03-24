// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Path parsing for storage URIs
//!
//! This module handles parsing of ipc:// URIs for storage operations.
//! Format: ipc://bucket_address/path/to/object
//!
//! Examples:
//! - ipc://0x1234.../documents/file.txt
//! - ipc://t1.../data/backup.tar.gz

use anyhow::{anyhow, Context, Result};
use fvm_shared::address::{Address, Error as NetworkError, Network};
use std::str::FromStr;

/// Represents a parsed storage path
#[derive(Debug, Clone)]
pub struct StoragePath {
    /// The bucket contract address
    pub bucket_address: Address,
    /// The object key/path within the bucket
    pub key: String,
}

impl StoragePath {
    /// Parse a storage URI in the format ipc://bucket_address/path/to/object
    pub fn parse(uri: &str) -> Result<Self> {
        if !uri.starts_with("ipc://") {
            return Err(anyhow!("Storage path must start with ipc://"));
        }

        let path_part = &uri[6..]; // Remove "ipc://"

        // Find the first '/' to separate bucket address from key
        let (bucket_str, key) = match path_part.find('/') {
            Some(idx) => {
                let bucket = &path_part[..idx];
                let key = &path_part[idx + 1..]; // Skip the '/'
                (bucket, key.to_string())
            }
            None => {
                // No key provided, just bucket address
                (path_part, String::new())
            }
        };

        if bucket_str.is_empty() {
            return Err(anyhow!("Bucket address cannot be empty"));
        }

        let bucket_address = parse_address(bucket_str)
            .with_context(|| format!("Invalid bucket address: {}", bucket_str))?;

        Ok(StoragePath {
            bucket_address,
            key,
        })
    }

    /// Check if this path represents a bucket root (no key specified)
    pub fn is_bucket_root(&self) -> bool {
        self.key.is_empty()
    }

    /// Get the parent directory path (for recursive operations)
    pub fn parent(&self) -> Option<String> {
        if self.key.is_empty() {
            return None;
        }

        match self.key.rfind('/') {
            Some(idx) => Some(self.key[..=idx].to_string()),
            None => Some(String::new()),
        }
    }

    /// Check if this path is a prefix of another path (directory-like)
    pub fn is_prefix_of(&self, other: &str) -> bool {
        if self.key.is_empty() {
            return true;
        }

        other.starts_with(&self.key) && (
            other.len() == self.key.len() ||
            other.as_bytes().get(self.key.len()) == Some(&b'/')
        )
    }

    /// Convert to a string representation
    pub fn to_uri(&self) -> String {
        if self.key.is_empty() {
            format!("ipc://{}", self.bucket_address)
        } else {
            format!("ipc://{}/{}", self.bucket_address, self.key)
        }
    }
}

/// Parse an address from a string (supports f/t addresses and 0x addresses)
pub fn parse_address(s: &str) -> Result<Address> {
    // Try parsing as Filecoin address (f/t prefix)
    let addr = Network::Mainnet
        .parse_address(s)
        .or_else(|e| match e {
            NetworkError::UnknownNetwork => Network::Testnet.parse_address(s),
            _ => Err(e),
        })
        .or_else(|_| {
            // Try parsing as Ethereum address (0x prefix)
            let eth_addr = ethers::types::Address::from_str(s)?;
            ipc_api::ethers_address_to_fil_address(&eth_addr)
        })?;

    Ok(addr)
}

/// Check if a path string is a storage path (starts with ipc://)
pub fn is_storage_path(path: &str) -> bool {
    path.starts_with("ipc://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_storage_path() {
        let path = StoragePath::parse("ipc://0x1234567890123456789012345678901234567890/documents/file.txt")
            .expect("should parse");

        assert_eq!(path.key, "documents/file.txt");
    }

    #[test]
    fn test_parse_bucket_root() {
        let path = StoragePath::parse("ipc://0x1234567890123456789012345678901234567890")
            .expect("should parse");

        assert!(path.is_bucket_root());
        assert_eq!(path.key, "");
    }

    #[test]
    fn test_parse_bucket_with_trailing_slash() {
        let path = StoragePath::parse("ipc://0x1234567890123456789012345678901234567890/")
            .expect("should parse");

        assert_eq!(path.key, "");
    }

    #[test]
    fn test_invalid_uri() {
        assert!(StoragePath::parse("http://bucket/file").is_err());
        assert!(StoragePath::parse("bucket/file").is_err());
        assert!(StoragePath::parse("ipc://").is_err());
    }

    #[test]
    fn test_parent_path() {
        let path = StoragePath::parse("ipc://0x1234567890123456789012345678901234567890/dir1/dir2/file.txt")
            .expect("should parse");

        assert_eq!(path.parent(), Some("dir1/dir2/".to_string()));
    }

    #[test]
    fn test_is_prefix_of() {
        let path = StoragePath::parse("ipc://0x1234567890123456789012345678901234567890/documents/")
            .expect("should parse");

        assert!(path.is_prefix_of("documents/file.txt"));
        assert!(path.is_prefix_of("documents/subfolder/file.txt"));
        assert!(!path.is_prefix_of("other/file.txt"));
    }

    #[test]
    fn test_to_uri() {
        let uri = "ipc://0x1234567890123456789012345678901234567890/path/to/file.txt";
        let path = StoragePath::parse(uri).expect("should parse");
        assert_eq!(path.to_uri(), uri);
    }
}
