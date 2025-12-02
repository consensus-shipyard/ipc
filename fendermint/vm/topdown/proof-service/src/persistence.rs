// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Persistent storage for proof cache using RocksDB
//!
//! # Why a Separate Database?
//!
//! The proof cache uses its own RocksDB instance for:
//! 1. **Lifecycle Independence**: Can be cleared without affecting chain state
//! 2. **Performance Isolation**: Large proofs don't impact block storage I/O
//! 3. **Operational Flexibility**: Independent backup/restore
//!
//! If cache is wiped, proofs regenerate from parent chain.
//!
//! # Column Families
//!
//! - `metadata`: Schema version, last committed instance
//! - `bundles`: Proof bundles keyed by instance_id

use crate::types::{CacheEntry, SerializableCacheEntry};
use anyhow::{Context, Result};
use rocksdb::{Options, DB};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Database schema version
const SCHEMA_VERSION: u32 = 1;

/// Column family names
const CF_METADATA: &str = "metadata";
const CF_BUNDLES: &str = "bundles";

/// Metadata keys
const KEY_SCHEMA_VERSION: &[u8] = b"schema_version";

/// Persistent storage for proof cache
pub struct ProofCachePersistence {
    db: Arc<DB>,
}

impl ProofCachePersistence {
    /// Open or create a persistent cache at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        info!(?path, "Opening proof cache database");

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

        // Open database with column families
        let cfs = vec![CF_METADATA, CF_BUNDLES];
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
                self.db.put_cf(
                    &cf_meta,
                    KEY_SCHEMA_VERSION,
                    serde_json::to_vec(&SCHEMA_VERSION)?,
                )?;
                info!(version = SCHEMA_VERSION, "Initialized new schema");
            }
        }

        Ok(())
    }

    /// Save a cache entry to disk
    pub fn save_entry(&self, entry: &CacheEntry) -> Result<()> {
        let cf_bundles = self
            .db
            .cf_handle(CF_BUNDLES)
            .context("Failed to get bundles column family")?;

        let key = entry.certificate.gpbft_instance.to_be_bytes();
        let value = serde_json::to_vec(&SerializableCacheEntry::from(entry))
            .context("Failed to serialize cache entry")?;

        self.db.put_cf(&cf_bundles, key, value)?;

        debug!(
            instance_id = entry.certificate.gpbft_instance,
            "Saved cache entry to disk"
        );
        Ok(())
    }

    /// Load all entries from disk
    ///
    /// Used on startup to populate the in-memory cache.
    pub fn load_all_entries(&self) -> Result<Vec<CacheEntry>> {
        let cf_bundles = self
            .db
            .cf_handle(CF_BUNDLES)
            .context("Failed to get bundles column family")?;

        let mut entries = Vec::new();
        let iter = self
            .db
            .iterator_cf(&cf_bundles, rocksdb::IteratorMode::Start);

        for item in iter {
            let (_, value) = item?;
            let entry: SerializableCacheEntry =
                serde_json::from_slice(&value).context("Failed to deserialize cache entry")?;
            entries.push(CacheEntry::try_from(entry)?);
        }

        info!(
            loaded_count = entries.len(),
            "Loaded all cache entries from disk"
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

    /// Clear all entries from disk
    pub fn clear_all_entries(&self) -> Result<()> {
        let cf_bundles = self
            .db
            .cf_handle(CF_BUNDLES)
            .context("Failed to get bundles column family")?;

        // Collect all keys first to avoid iterator invalidation
        let keys: Vec<Box<[u8]>> = self
            .db
            .iterator_cf(&cf_bundles, rocksdb::IteratorMode::Start)
            .filter_map(|result| result.ok().map(|(k, _)| k))
            .collect();

        let count = keys.len();
        debug!(count, "Collected all keys to clear");
        for key in keys {
            self.db.delete_cf(&cf_bundles, &key)?;
        }

        debug!(count, "Cleared all cache entries from disk");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        CacheEntry, SerializableCacheEntry, SerializableECChainEntry, SerializableF3Certificate,
        SerializablePowerEntries, SerializablePowerEntry, SerializableSupplementalData,
    };
    use cid::Cid;
    use multihash_codetable::{Code, MultihashDigest};
    use proofs::proofs::common::bundle::UnifiedProofBundle;
    use std::time::SystemTime;
    use tempfile::tempdir;

    fn create_test_entry(instance_id: u64) -> CacheEntry {
        let power_table_cid = Cid::new_v1(0x55, Code::Blake2b256.digest(b"test"));

        let ec_chain = (100..=102)
            .map(|epoch| SerializableECChainEntry {
                epoch,
                key: vec![],
                power_table: power_table_cid.to_string(),
                commitments: vec![0u8; 32],
            })
            .collect();

        let serializable = SerializableCacheEntry {
            proof_bundle: Some(UnifiedProofBundle {
                storage_proofs: vec![],
                event_proofs: vec![],
                blocks: vec![],
            }),
            certificate: SerializableF3Certificate {
                gpbft_instance: instance_id,
                ec_chain,
                supplemental_data: SerializableSupplementalData {
                    power_table: power_table_cid.to_string(),
                    commitments: vec![0u8; 32],
                },
                signers: vec![0],
                signature: vec![],
                power_table_delta: vec![],
            },
            power_table: SerializablePowerEntries(vec![SerializablePowerEntry {
                id: 1,
                power: "1000".to_string(),
                pub_key: vec![0u8; 48],
            }]),
            generated_at: SystemTime::now(),
            source_rpc: "test".to_string(),
        };

        CacheEntry::try_from(serializable).expect("valid cache entry")
    }

    #[test]
    fn test_persistence_basic_operations() {
        let dir = tempdir().unwrap();
        let persistence = ProofCachePersistence::open(dir.path()).unwrap();

        // Test entry save/load
        let entry = create_test_entry(101);
        persistence.save_entry(&entry).unwrap();

        let loaded = persistence.load_all_entries().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].certificate.gpbft_instance, 101);
    }

    #[test]
    fn test_persistence_multiple_entries() {
        let dir = tempdir().unwrap();
        let persistence = ProofCachePersistence::open(dir.path()).unwrap();

        // Save multiple entries
        for i in 100..105 {
            persistence.save_entry(&create_test_entry(i)).unwrap();
        }

        // Load all
        let entries = persistence.load_all_entries().unwrap();
        assert_eq!(entries.len(), 5);
    }

    #[test]
    fn test_persistence_delete() {
        let dir = tempdir().unwrap();
        let persistence = ProofCachePersistence::open(dir.path()).unwrap();

        // Save and delete
        persistence.save_entry(&create_test_entry(100)).unwrap();
        persistence.delete_entry(100).unwrap();

        let entries = persistence.load_all_entries().unwrap();
        assert_eq!(entries.len(), 0);
    }
}
