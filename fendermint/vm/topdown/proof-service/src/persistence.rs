// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Persistent storage for proof cache using RocksDB

use crate::types::CacheEntry;
use anyhow::{Context, Result};
use rocksdb::{Options, DB};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Database schema version
const SCHEMA_VERSION: u32 = 1;

/// Column family names
const CF_METADATA: &str = "metadata";
const CF_BUNDLES: &str = "bundles";
const CF_CERTIFICATES: &str = "certificates";

/// Metadata keys
const KEY_SCHEMA_VERSION: &[u8] = b"schema_version";
const KEY_LAST_COMMITTED: &[u8] = b"last_committed_instance";
const KEY_HIGHEST_CACHED: &[u8] = b"highest_cached_instance";

/// Persistent cache metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub schema_version: u32,
    pub last_committed_instance: u64,
    pub highest_cached_instance: Option<u64>,
    pub provider_provenance: String,
}

/// Persistent storage for proof cache
pub struct ProofCachePersistence {
    db: Arc<DB>,
}

impl ProofCachePersistence {
    /// Open or create a persistent cache at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        info!(?path, "Opening proof cache database");

        // Configure RocksDB
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

        // Open database with column families
        let cfs = vec![CF_METADATA, CF_BUNDLES, CF_CERTIFICATES];
        let db = DB::open_cf(&opts, path, cfs)
            .context("Failed to open RocksDB database for proof cache")?;

        let persistence = Self { db: Arc::new(db) };

        // Initialize or verify schema
        persistence.init_schema()?;

        Ok(persistence)
    }

    /// Initialize schema or verify existing one
    fn init_schema(&self) -> Result<()> {
        let cf_meta = self
            .db
            .cf_handle(CF_METADATA)
            .context("Failed to get metadata column family")?;

        // Check existing schema version
        match self.db.get_cf(&cf_meta, KEY_SCHEMA_VERSION)? {
            Some(data) => {
            let version = serde_json::from_slice::<u32>(&data)
                .context("Failed to deserialize schema version")?;

                if version != SCHEMA_VERSION {
                    anyhow::bail!(
                        "Schema version mismatch: found {}, expected {}",
                        version,
                        SCHEMA_VERSION
                    );
                }
                debug!(version, "Verified schema version");
            }
            None => {
                // Initialize new schema
                self.db
                    .put_cf(&cf_meta, KEY_SCHEMA_VERSION, serde_json::to_vec(&SCHEMA_VERSION)?)
                    .context("Failed to write schema version")?;
                info!(version = SCHEMA_VERSION, "Initialized new schema");
            }
        }

        Ok(())
    }

    /// Load last committed instance from disk
    pub fn load_last_committed(&self) -> Result<Option<u64>> {
        let cf_meta = self
            .db
            .cf_handle(CF_METADATA)
            .context("Failed to get metadata column family")?;

        match self.db.get_cf(&cf_meta, KEY_LAST_COMMITTED)? {
            Some(data) => {
            let instance = serde_json::from_slice(&data)
                .context("Failed to deserialize last committed instance")?;
                Ok(Some(instance))
            }
            None => Ok(None),
        }
    }

    /// Save last committed instance to disk
    pub fn save_last_committed(&self, instance: u64) -> Result<()> {
        let cf_meta = self
            .db
            .cf_handle(CF_METADATA)
            .context("Failed to get metadata column family")?;

        self.db
            .put_cf(&cf_meta, KEY_LAST_COMMITTED, serde_json::to_vec(&instance)?)
            .context("Failed to save last committed instance")?;

        debug!(instance, "Saved last committed instance");
        Ok(())
    }

    /// Save a cache entry to disk
    pub fn save_entry(&self, entry: &CacheEntry) -> Result<()> {
        let cf_bundles = self
            .db
            .cf_handle(CF_BUNDLES)
            .context("Failed to get bundles column family")?;

        let key = entry.instance_id.to_be_bytes();
        let value = serde_json::to_vec(entry).context("Failed to serialize cache entry")?;

        self.db
            .put_cf(&cf_bundles, key, value)
            .context("Failed to save cache entry")?;

        debug!(instance_id = entry.instance_id, "Saved cache entry to disk");
        Ok(())
    }

    /// Load a cache entry from disk
    pub fn load_entry(&self, instance_id: u64) -> Result<Option<CacheEntry>> {
        let cf_bundles = self
            .db
            .cf_handle(CF_BUNDLES)
            .context("Failed to get bundles column family")?;

        let key = instance_id.to_be_bytes();

        match self.db.get_cf(&cf_bundles, key)? {
            Some(data) => {
                let entry = serde_json::from_slice(&data)
                    .context("Failed to deserialize cache entry")?;
                Ok(Some(entry))
            }
            None => Ok(None),
        }
    }

    /// Load all entries within a range
    pub fn load_range(&self, start: u64, end: u64) -> Result<Vec<CacheEntry>> {
        let cf_bundles = self
            .db
            .cf_handle(CF_BUNDLES)
            .context("Failed to get bundles column family")?;

        let mut entries = Vec::new();
        
        // Create iterator with range bounds
        let start_key = start.to_be_bytes();
        let end_key = end.to_be_bytes();
        
        let iter = self.db.iterator_cf(
            &cf_bundles,
            rocksdb::IteratorMode::From(&start_key, rocksdb::Direction::Forward),
        );

        for item in iter {
            let (key, value) = item?;
            
            // Check if we've gone past the end
            if key.as_ref() > &end_key[..] {
                break;
            }

            let entry: CacheEntry = serde_json::from_slice(&value)
                .context("Failed to deserialize cache entry during range load")?;
            entries.push(entry);
        }

        debug!(
            start,
            end,
            loaded_count = entries.len(),
            "Loaded cache entries from disk"
        );

        Ok(entries)
    }

    /// Delete an entry from disk
    pub fn delete_entry(&self, instance_id: u64) -> Result<()> {
        let cf_bundles = self
            .db
            .cf_handle(CF_BUNDLES)
            .context("Failed to get bundles column family")?;

        let key = instance_id.to_be_bytes();
        self.db.delete_cf(&cf_bundles, key)?;

        debug!(instance_id, "Deleted cache entry from disk");
        Ok(())
    }

    /// Delete entries older than the given instance
    pub fn cleanup_old_entries(&self, cutoff_instance: u64) -> Result<usize> {
        let cf_bundles = self
            .db
            .cf_handle(CF_BUNDLES)
            .context("Failed to get bundles column family")?;

        let mut count = 0;
        let cutoff_key = cutoff_instance.to_be_bytes();

        // Collect keys to delete (can't delete while iterating)
        let mut keys_to_delete = Vec::new();
        
        let iter = self.db.iterator_cf(
            &cf_bundles,
            rocksdb::IteratorMode::Start,
        );

        for item in iter {
            let (key, _) = item?;
            
            // If key is less than cutoff, mark for deletion
            if key.as_ref() < &cutoff_key[..] {
                keys_to_delete.push(key.to_vec());
            } else {
                // Keys are ordered, so we can stop here
                break;
            }
        }

        // Delete collected keys
        for key in keys_to_delete {
            self.db.delete_cf(&cf_bundles, &key)?;
            count += 1;
        }

        if count > 0 {
            info!(
                count,
                cutoff_instance,
                "Cleaned up old entries from disk"
            );
        }

        Ok(count)
    }

    /// Save certificate verification cache
    pub fn save_verified_certificate(&self, cert_hash: &[u8], instance_id: u64) -> Result<()> {
        let cf_certs = self
            .db
            .cf_handle(CF_CERTIFICATES)
            .context("Failed to get certificates column family")?;

        self.db
            .put_cf(&cf_certs, cert_hash, instance_id.to_be_bytes())
            .context("Failed to save verified certificate")?;

        Ok(())
    }

    /// Check if a certificate has been verified before
    pub fn is_certificate_verified(&self, cert_hash: &[u8]) -> Result<bool> {
        let cf_certs = self
            .db
            .cf_handle(CF_CERTIFICATES)
            .context("Failed to get certificates column family")?;

        Ok(self.db.get_cf(&cf_certs, cert_hash)?.is_some())
    }

    /// Validate cache integrity on startup
    pub fn validate_integrity(&self) -> Result<()> {
        info!("Validating cache integrity");

        let cf_bundles = self
            .db
            .cf_handle(CF_BUNDLES)
            .context("Failed to get bundles column family")?;

        let mut valid_count = 0;
        let mut invalid_count = 0;

        let iter = self.db.iterator_cf(&cf_bundles, rocksdb::IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            
            // Try to deserialize
            match serde_json::from_slice::<CacheEntry>(&value) {
                Ok(entry) => {
                    // Verify key matches instance ID
                    let expected_key = entry.instance_id.to_be_bytes();
                    if key.as_ref() == &expected_key[..] {
                        valid_count += 1;
                    } else {
                        warn!(
                            instance_id = entry.instance_id,
                            "Key mismatch in cache entry"
                        );
                        invalid_count += 1;
                    }
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        "Failed to deserialize cache entry"
                    );
                    invalid_count += 1;
                }
            }
        }

        info!(
            valid_count,
            invalid_count,
            "Cache integrity validation complete"
        );

        if invalid_count > 0 {
            warn!("Found {} invalid entries during integrity check", invalid_count);
        }

        Ok(())
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<PersistenceStats> {
        let cf_bundles = self
            .db
            .cf_handle(CF_BUNDLES)
            .context("Failed to get bundles column family")?;

        let mut entry_count = 0;
        let mut total_size = 0;

        let iter = self.db.iterator_cf(&cf_bundles, rocksdb::IteratorMode::Start);

        for item in iter {
            let (_, value) = item?;
            entry_count += 1;
            total_size += value.len();
        }

        Ok(PersistenceStats {
            entry_count,
            total_size_bytes: total_size,
            last_committed: self.load_last_committed()?,
        })
    }
}

/// Statistics about the persistent cache
#[derive(Debug, Clone)]
pub struct PersistenceStats {
    pub entry_count: usize,
    pub total_size_bytes: usize,
    pub last_committed: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cid::Cid;
    use fendermint_actor_f3_cert_manager::types::F3Certificate;
    use multihash_codetable::{Code, MultihashDigest};
    use std::time::SystemTime;
    use tempfile::tempdir;

    fn create_test_entry(instance_id: u64) -> CacheEntry {
        let power_table_cid = Cid::new_v1(0x55, Code::Blake2b256.digest(b"test"));

        CacheEntry {
            instance_id,
            finalized_epochs: vec![100, 101, 102],
            proof_bundle_bytes: vec![1, 2, 3], // Mock proof bundle bytes
            f3_certificate_bytes: vec![4, 5, 6], // Mock F3 certificate bytes
            actor_certificate: F3Certificate {
                instance_id,
                finalized_epochs: vec![100, 101, 102],
                power_table_cid,
                signature: vec![],
                certificate_data: vec![],
            },
            generated_at: SystemTime::now(),
            source_rpc: "test".to_string(),
        }
    }

    #[test]
    fn test_persistence_basic_operations() {
        let dir = tempdir().unwrap();
        let persistence = ProofCachePersistence::open(dir.path()).unwrap();

        // Test last committed
        assert_eq!(persistence.load_last_committed().unwrap(), None);
        persistence.save_last_committed(100).unwrap();
        assert_eq!(persistence.load_last_committed().unwrap(), Some(100));

        // Test entry save/load
        let entry = create_test_entry(101);
        persistence.save_entry(&entry).unwrap();

        let loaded = persistence.load_entry(101).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().instance_id, 101);

        // Test non-existent entry
        assert!(persistence.load_entry(999).unwrap().is_none());
    }

    #[test]
    fn test_persistence_range_operations() {
        let dir = tempdir().unwrap();
        let persistence = ProofCachePersistence::open(dir.path()).unwrap();

        // Save multiple entries
        for i in 100..110 {
            persistence.save_entry(&create_test_entry(i)).unwrap();
        }

        // Load range
        let entries = persistence.load_range(103, 107).unwrap();
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].instance_id, 103);
        assert_eq!(entries[4].instance_id, 107);
    }

    #[test]
    fn test_persistence_cleanup() {
        let dir = tempdir().unwrap();
        let persistence = ProofCachePersistence::open(dir.path()).unwrap();

        // Save multiple entries
        for i in 100..110 {
            persistence.save_entry(&create_test_entry(i)).unwrap();
        }

        // Cleanup old entries
        let deleted = persistence.cleanup_old_entries(105).unwrap();
        assert_eq!(deleted, 5);

        // Verify cleanup
        assert!(persistence.load_entry(104).unwrap().is_none());
        assert!(persistence.load_entry(105).unwrap().is_some());
    }

    #[test]
    fn test_persistence_integrity() {
        let dir = tempdir().unwrap();
        let persistence = ProofCachePersistence::open(dir.path()).unwrap();

        // Save some entries
        for i in 100..103 {
            persistence.save_entry(&create_test_entry(i)).unwrap();
        }

        // Validate should succeed
        persistence.validate_integrity().unwrap();

        // Get stats
        let stats = persistence.get_stats().unwrap();
        assert_eq!(stats.entry_count, 3);
    }
}
