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
//! - `metadata`: Schema version
//! - `certificates`: F3 certificates keyed by instance_id
//! - `epoch_proofs`: Proof bundles keyed by epoch

use crate::types::{CertificateEntry, EpochProofEntry, SerializableCertificateEntry};
use anyhow::{Context, Result};
use fvm_shared::clock::ChainEpoch;
use rocksdb::{BoundColumnFamily, Options, DB};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Database schema version
const SCHEMA_VERSION: u32 = 1;

/// Column family names
const CF_METADATA: &str = "metadata";
const CF_CERTIFICATES: &str = "certificates";
const CF_EPOCH_PROOFS: &str = "epoch_proofs";

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

        let cfs = vec![CF_METADATA, CF_CERTIFICATES, CF_EPOCH_PROOFS];
        let db = DB::open_cf(&opts, path, cfs)
            .context("Failed to open RocksDB database for proof cache")?;

        let persistence = Self { db: Arc::new(db) };
        persistence.init_schema()?;

        Ok(persistence)
    }

    fn get_cf(&self, name: &str) -> Result<Arc<BoundColumnFamily>> {
        self.db
            .cf_handle(name)
            .with_context(|| format!("Failed to get {} column family", name))
    }

    fn init_schema(&self) -> Result<()> {
        let cf = self.get_cf(CF_METADATA)?;

        match self.db.get_cf(&cf, KEY_SCHEMA_VERSION)? {
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
                info!(version = SCHEMA_VERSION, "Verified schema version");
            }
            None => {
                self.db.put_cf(
                    &cf,
                    KEY_SCHEMA_VERSION,
                    serde_json::to_vec(&SCHEMA_VERSION)?,
                )?;
                info!(version = SCHEMA_VERSION, "Initialized new schema");
            }
        }

        Ok(())
    }

    pub fn save_certificate(&self, entry: &CertificateEntry) -> Result<()> {
        let cf = self.get_cf(CF_CERTIFICATES)?;
        let key = entry.instance_id().to_be_bytes();
        let value = serde_json::to_vec(&SerializableCertificateEntry::from(entry))
            .context("Failed to serialize certificate entry")?;

        self.db.put_cf(&cf, key, value)?;
        debug!(
            instance_id = entry.instance_id(),
            "Saved certificate to disk"
        );
        Ok(())
    }

    pub fn load_all_certificates(&self) -> Result<Vec<CertificateEntry>> {
        let cf = self.get_cf(CF_CERTIFICATES)?;
        let mut entries = Vec::new();

        for item in self.db.iterator_cf(&cf, rocksdb::IteratorMode::Start) {
            let (_, value) = item?;
            let entry: SerializableCertificateEntry = serde_json::from_slice(&value)
                .context("Failed to deserialize certificate entry")?;
            entries.push(CertificateEntry::try_from(entry)?);
        }

        info!(count = entries.len(), "Loaded certificates from disk");
        Ok(entries)
    }

    pub fn delete_certificate(&self, instance_id: u64) -> Result<()> {
        let cf = self.get_cf(CF_CERTIFICATES)?;
        self.db.delete_cf(&cf, instance_id.to_be_bytes())?;
        debug!(instance_id, "Deleted certificate from disk");
        Ok(())
    }

    pub fn save_epoch_proof(&self, entry: &EpochProofEntry) -> Result<()> {
        let cf = self.get_cf(CF_EPOCH_PROOFS)?;
        let key = entry.epoch.to_be_bytes();
        let value = serde_json::to_vec(entry).context("Failed to serialize epoch proof entry")?;

        self.db.put_cf(&cf, key, value)?;
        debug!(epoch = entry.epoch, "Saved epoch proof to disk");
        Ok(())
    }

    pub fn load_all_epoch_proofs(&self) -> Result<Vec<EpochProofEntry>> {
        let cf = self.get_cf(CF_EPOCH_PROOFS)?;
        let mut entries = Vec::new();

        for item in self.db.iterator_cf(&cf, rocksdb::IteratorMode::Start) {
            let (_, value) = item?;
            let entry: EpochProofEntry = serde_json::from_slice(&value)
                .context("Failed to deserialize epoch proof entry")?;
            entries.push(entry);
        }

        info!(count = entries.len(), "Loaded epoch proofs from disk");
        Ok(entries)
    }

    pub fn delete_epoch_proof(&self, epoch: ChainEpoch) -> Result<()> {
        let cf = self.get_cf(CF_EPOCH_PROOFS)?;
        self.db.delete_cf(&cf, epoch.to_be_bytes())?;
        debug!(epoch, "Deleted epoch proof from disk");
        Ok(())
    }

    pub fn clear_all(&self) -> Result<()> {
        self.clear_cf(CF_CERTIFICATES)?;
        self.clear_cf(CF_EPOCH_PROOFS)?;
        debug!("Cleared all cache entries from disk");
        Ok(())
    }

    fn clear_cf(&self, cf_name: &str) -> Result<()> {
        if let Some(cf) = self.db.cf_handle(cf_name) {
            let keys: Vec<Box<[u8]>> = self
                .db
                .iterator_cf(&cf, rocksdb::IteratorMode::Start)
                .filter_map(|r| r.ok().map(|(k, _)| k))
                .collect();
            for key in keys {
                self.db.delete_cf(&cf, &key)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        SerializableCertificateEntry, SerializableECChainEntry, SerializableF3Certificate,
        SerializablePowerEntries, SerializablePowerEntry, SerializableSupplementalData,
    };
    use cid::Cid;
    use multihash_codetable::{Code, MultihashDigest};
    use proofs::proofs::common::bundle::UnifiedProofBundle;
    use std::time::SystemTime;
    use tempfile::tempdir;

    fn create_test_certificate(instance_id: u64) -> CertificateEntry {
        let power_table_cid = Cid::new_v1(0x55, Code::Blake2b256.digest(b"test"));

        let ec_chain = (100..=102)
            .map(|epoch| SerializableECChainEntry {
                epoch,
                key: vec!["0".to_string()],
                power_table: power_table_cid.to_string(),
                commitments: vec![0u8; 32],
            })
            .collect();

        let serializable = SerializableCertificateEntry {
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
                pub_key: vec![1u8; 48],
            }]),
            source_rpc: "test".to_string(),
            fetched_at: SystemTime::now(),
        };

        CertificateEntry::try_from(serializable).expect("valid certificate entry")
    }

    fn create_test_epoch_proof(epoch: ChainEpoch) -> EpochProofEntry {
        EpochProofEntry::new(
            epoch,
            UnifiedProofBundle {
                storage_proofs: vec![],
                event_proofs: vec![],
                blocks: vec![],
            },
            5, // cert_instance
        )
    }

    #[test]
    fn test_persistence_certificates() {
        let dir = tempdir().unwrap();
        let persistence = ProofCachePersistence::open(dir.path()).unwrap();

        // Save certificates
        let cert1 = create_test_certificate(100);
        let cert2 = create_test_certificate(101);
        persistence.save_certificate(&cert1).unwrap();
        persistence.save_certificate(&cert2).unwrap();

        // Load all
        let loaded = persistence.load_all_certificates().unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_persistence_epoch_proofs() {
        let dir = tempdir().unwrap();
        let persistence = ProofCachePersistence::open(dir.path()).unwrap();

        // Save epoch proofs
        for epoch in 100..105 {
            persistence
                .save_epoch_proof(&create_test_epoch_proof(epoch))
                .unwrap();
        }

        // Load all
        let loaded = persistence.load_all_epoch_proofs().unwrap();
        assert_eq!(loaded.len(), 5);
    }

    #[test]
    fn test_persistence_delete() {
        let dir = tempdir().unwrap();
        let persistence = ProofCachePersistence::open(dir.path()).unwrap();

        // Save and delete certificate
        persistence
            .save_certificate(&create_test_certificate(100))
            .unwrap();
        persistence.delete_certificate(100).unwrap();
        let certs = persistence.load_all_certificates().unwrap();
        assert_eq!(certs.len(), 0);

        // Save and delete epoch proof
        persistence
            .save_epoch_proof(&create_test_epoch_proof(200))
            .unwrap();
        persistence.delete_epoch_proof(200).unwrap();
        let proofs = persistence.load_all_epoch_proofs().unwrap();
        assert_eq!(proofs.len(), 0);
    }

    #[test]
    fn test_persistence_clear_all() {
        let dir = tempdir().unwrap();
        let persistence = ProofCachePersistence::open(dir.path()).unwrap();

        // Save some data
        persistence
            .save_certificate(&create_test_certificate(100))
            .unwrap();
        persistence
            .save_epoch_proof(&create_test_epoch_proof(200))
            .unwrap();

        // Clear all
        persistence.clear_all().unwrap();

        assert_eq!(persistence.load_all_certificates().unwrap().len(), 0);
        assert_eq!(persistence.load_all_epoch_proofs().unwrap().len(), 0);
    }
}
