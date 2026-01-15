// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Two-level cache for proof bundles with optional disk persistence
//!
//! # Architecture
//!
//! The cache is organized in two levels:
//! - **Certificate Store**: Stores F3 certificates keyed by instance ID
//! - **Epoch Proof Store**: Stores proof bundles keyed by epoch
//!
//! This design avoids duplicating certificates when multiple epochs
//! reference the same certificate

use crate::config::CacheConfig;
use crate::observe::{ProofCached, CACHE_HIT_TOTAL, CACHE_SIZE};
use crate::persistence::ProofCachePersistence;
use crate::types::{CertificateEntry, EpochProofEntry, EpochProofWithCertificate};
use anyhow::{Context, Result};
use fvm_shared::clock::ChainEpoch;
use ipc_observability::emit;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

/// Thread-safe two-level cache for proof bundles
#[derive(Clone)]
pub struct ProofCache {
    /// Certificate store: instance_id -> CertificateEntry
    certificates: Arc<RwLock<BTreeMap<u64, CertificateEntry>>>,

    /// Epoch proof store: epoch -> EpochProofEntry
    epoch_proofs: Arc<RwLock<BTreeMap<ChainEpoch, EpochProofEntry>>>,

    /// Configuration
    config: CacheConfig,

    /// Last committed epoch (updated when proofs are used on-chain)
    last_committed_epoch: Arc<AtomicI64>,

    /// Last committed F3 instance (updated when proofs are used on-chain)
    last_committed_instance: Arc<AtomicU64>,

    /// Optional disk persistence
    persistence: Option<Arc<ProofCachePersistence>>,
}

impl ProofCache {
    /// Create a new proof cache (in-memory only)
    pub fn new(
        last_committed_epoch: ChainEpoch,
        last_committed_instance: u64,
        config: CacheConfig,
    ) -> Self {
        Self {
            certificates: Arc::new(RwLock::new(BTreeMap::new())),
            epoch_proofs: Arc::new(RwLock::new(BTreeMap::new())),
            last_committed_epoch: Arc::new(AtomicI64::new(last_committed_epoch)),
            last_committed_instance: Arc::new(AtomicU64::new(last_committed_instance)),
            config,
            persistence: None,
        }
    }

    /// Create a new proof cache with disk persistence
    ///
    /// Loads existing entries from disk on startup. If committed state exists in
    /// persistence, uses the higher of persisted vs provided values.
    pub fn new_with_persistence(
        initial_committed_epoch: ChainEpoch,
        initial_committed_instance: u64,
        config: CacheConfig,
        db_path: &Path,
    ) -> Result<Self> {
        let persistence = ProofCachePersistence::open(db_path)?;

        // Load certificates
        let cert_entries = persistence
            .load_all_certificates()
            .context("Failed to load certificates from disk")?;
        let certificates: BTreeMap<u64, CertificateEntry> = cert_entries
            .into_iter()
            .map(|e| (e.instance_id(), e))
            .collect();

        // Load epoch proofs
        let proof_entries = persistence
            .load_all_epoch_proofs()
            .context("Failed to load epoch proofs from disk")?;
        let epoch_proofs: BTreeMap<ChainEpoch, EpochProofEntry> =
            proof_entries.into_iter().map(|e| (e.epoch, e)).collect();

        tracing::info!(
            certificates = certificates.len(),
            epoch_proofs = epoch_proofs.len(),
            "Loaded cache from disk"
        );

        let cache = Self {
            certificates: Arc::new(RwLock::new(certificates)),
            epoch_proofs: Arc::new(RwLock::new(epoch_proofs)),
            last_committed_epoch: Arc::new(AtomicI64::new(initial_committed_epoch)),
            last_committed_instance: Arc::new(AtomicU64::new(initial_committed_instance)),
            config,
            persistence: Some(Arc::new(persistence)),
        };

        // Cleanup old entries
        cache.cleanup_old_epochs(initial_committed_epoch)?;

        Ok(cache)
    }

    /// Insert a certificate into the store
    pub fn insert_certificate(&self, entry: CertificateEntry) -> Result<()> {
        let instance_id = entry.instance_id();
        self.certificates.write().insert(instance_id, entry.clone());
        self.with_persistence(|p| p.save_certificate(&entry))?;
        tracing::debug!(instance_id, "Inserted certificate into cache");
        Ok(())
    }

    /// Get a certificate by instance ID
    pub fn get_certificate(&self, instance_id: u64) -> Option<CertificateEntry> {
        self.certificates.read().get(&instance_id).cloned()
    }

    /// Check if a certificate exists
    pub fn contains_certificate(&self, instance_id: u64) -> bool {
        self.certificates.read().contains_key(&instance_id)
    }

    /// Get the highest cached certificate instance ID
    pub fn highest_cached_instance(&self) -> Option<u64> {
        self.certificates.read().keys().max().copied()
    }

    /// Insert epoch proofs into the cache
    ///
    /// This is typically called after processing a certificate, inserting
    /// proofs for all epochs in the certificate's suffix.
    pub fn insert_epoch_proofs(&self, entries: Vec<EpochProofEntry>) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        let epochs: Vec<ChainEpoch> = entries.iter().map(|e| e.epoch).collect();

        {
            let mut proofs = self.epoch_proofs.write();
            for entry in entries.iter() {
                proofs.insert(entry.epoch, entry.clone());
            }
        }

        self.with_persistence(|p| {
            for entry in &entries {
                p.save_epoch_proof(entry)?;
            }
            Ok(())
        })?;

        self.emit_cache_metrics(&epochs);
        tracing::debug!(?epochs, "Inserted epoch proofs into cache");
        Ok(())
    }

    fn emit_cache_metrics(&self, epochs: &[ChainEpoch]) {
        let cache_size = self.epoch_proofs.read().len();
        if let Some(highest) = self.highest_cached_epoch() {
            for epoch in epochs {
                emit(ProofCached {
                    instance: *epoch as u64,
                    cache_size,
                    highest_cached: highest as u64,
                });
            }
        }
        CACHE_SIZE.set(cache_size as i64);
    }

    /// Get proof for a specific epoch
    ///
    /// Returns the proof entry without the certificate.
    /// Use `get_epoch_proof_with_certificate` for full data.
    pub fn get_epoch_proof(&self, epoch: ChainEpoch) -> Option<EpochProofEntry> {
        let result = self.epoch_proofs.read().get(&epoch).cloned();

        // Record cache hit/miss
        CACHE_HIT_TOTAL
            .with_label_values(&[if result.is_some() { "hit" } else { "miss" }])
            .inc();

        result
    }

    /// Get proof for a specific epoch with its certificate
    ///
    /// This is the main query method for consumers. Returns everything
    /// needed for verification, including the finalized tipsets.
    pub fn get_epoch_proof_with_certificate(
        &self,
        epoch: ChainEpoch,
    ) -> Option<EpochProofWithCertificate> {
        let proof_entry = self.get_epoch_proof(epoch)?;
        let cert = self.get_certificate(proof_entry.cert_instance)?;

        Some(EpochProofWithCertificate::new(&proof_entry, &cert))
    }

    /// Check if an epoch proof exists
    pub fn contains_epoch_proof(&self, epoch: ChainEpoch) -> bool {
        self.epoch_proofs.read().contains_key(&epoch)
    }

    /// Get the highest cached epoch
    pub fn highest_cached_epoch(&self) -> Option<ChainEpoch> {
        self.epoch_proofs.read().keys().max().copied()
    }

    /// Get the lowest cached epoch
    pub fn lowest_cached_epoch(&self) -> Option<ChainEpoch> {
        self.epoch_proofs.read().keys().min().copied()
    }

    /// Mark an epoch and instance as committed and trigger cleanup
    pub fn mark_committed(&self, epoch: ChainEpoch, instance: u64) -> Result<()> {
        let old_epoch = self.last_committed_epoch.swap(epoch, Ordering::Release);
        let old_instance = self
            .last_committed_instance
            .swap(instance, Ordering::Release);

        tracing::info!(
            old_epoch,
            new_epoch = epoch,
            old_instance,
            new_instance = instance,
            "Updated last committed epoch and instance"
        );

        self.cleanup_old_epochs(epoch)
    }

    /// Get the last committed epoch and instance
    pub fn last_committed(&self) -> (ChainEpoch, u64) {
        (
            self.last_committed_epoch.load(Ordering::Acquire),
            self.last_committed_instance.load(Ordering::Acquire),
        )
    }

    /// Get the current last committed epoch
    pub fn last_committed_epoch(&self) -> ChainEpoch {
        self.last_committed_epoch.load(Ordering::Acquire)
    }

    /// Get the current last committed F3 instance
    pub fn last_committed_instance(&self) -> u64 {
        self.last_committed_instance.load(Ordering::Acquire)
    }

    /// Get the next uncommitted epoch (last_committed_epoch + 1)
    /// Returns None if no proof is available for that epoch
    pub fn get_next_uncommitted_epoch(&self) -> Option<ChainEpoch> {
        let next_epoch = self.last_committed_epoch() + 1;
        if self.contains_epoch_proof(next_epoch) {
            Some(next_epoch)
        } else {
            None
        }
    }

    /// Get the next uncommitted proof entry (epoch + certificate)
    /// Returns None if no proof is available for next epoch
    pub fn get_next_uncommitted_epoch_with_cert(&self) -> Option<EpochProofWithCertificate> {
        let next_epoch = self.get_next_uncommitted_epoch()?;
        self.get_epoch_proof_with_certificate(next_epoch)
    }

    /// Get the number of cached epoch proofs
    pub fn epoch_proof_count(&self) -> usize {
        self.epoch_proofs.read().len()
    }

    /// Get the number of cached certificates
    pub fn certificate_count(&self) -> usize {
        self.certificates.read().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.epoch_proofs.read().is_empty()
    }

    /// Remove epochs older than the retention window
    fn cleanup_old_epochs(&self, current_epoch: ChainEpoch) -> Result<()> {
        let cutoff = current_epoch.saturating_sub(self.config.retention_epochs as i64);

        let epochs_to_remove = self.collect_epochs_before(cutoff);
        if epochs_to_remove.is_empty() {
            tracing::debug!(cutoff, "No old epochs to cleanup");
            return Ok(());
        }

        // Remove proofs first, then cleanup orphaned certificates
        self.remove_epoch_proofs(&epochs_to_remove);
        let certs_to_remove = self.collect_unreferenced_certs();
        self.remove_certificates(&certs_to_remove);

        self.persist_deletions(&epochs_to_remove, &certs_to_remove)?;

        CACHE_SIZE.set(self.epoch_proofs.read().len() as i64);

        tracing::debug!(
            epochs_removed = epochs_to_remove.len(),
            certs_removed = certs_to_remove.len(),
            cutoff,
            "Cleaned up old cache entries"
        );

        Ok(())
    }

    fn collect_epochs_before(&self, cutoff: ChainEpoch) -> Vec<ChainEpoch> {
        self.epoch_proofs
            .read()
            .keys()
            .filter(|&&epoch| epoch < cutoff)
            .copied()
            .collect()
    }

    /// Find certificates not referenced by any remaining proofs
    fn collect_unreferenced_certs(&self) -> Vec<u64> {
        let referenced: std::collections::HashSet<u64> = self
            .epoch_proofs
            .read()
            .values()
            .map(|p| p.cert_instance)
            .collect();

        self.certificates
            .read()
            .keys()
            .filter(|id| !referenced.contains(id))
            .copied()
            .collect()
    }

    fn remove_epoch_proofs(&self, epochs: &[ChainEpoch]) {
        let mut proofs = self.epoch_proofs.write();
        for epoch in epochs {
            proofs.remove(epoch);
        }
    }

    fn remove_certificates(&self, cert_ids: &[u64]) {
        let mut certs = self.certificates.write();
        for id in cert_ids {
            certs.remove(id);
        }
    }

    fn persist_deletions(&self, epochs: &[ChainEpoch], cert_ids: &[u64]) -> Result<()> {
        self.with_persistence(|p| {
            for epoch in epochs {
                p.delete_epoch_proof(*epoch)?;
            }
            for id in cert_ids {
                p.delete_certificate(*id)?;
            }
            Ok(())
        })
    }

    /// Execute a function with persistence if enabled, otherwise no-op.
    fn with_persistence<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&ProofCachePersistence) -> Result<()>,
    {
        if let Some(persistence) = &self.persistence {
            f(persistence)?;
        }
        Ok(())
    }

    /// Get all cached epochs (for debugging)
    pub fn cached_epochs(&self) -> Vec<ChainEpoch> {
        self.epoch_proofs.read().keys().copied().collect()
    }

    /// Get all cached certificate instance IDs (for debugging)
    pub fn cached_certificate_instances(&self) -> Vec<u64> {
        self.certificates.read().keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        SerializableCertificateEntry, SerializableECChainEntry, SerializableF3Certificate,
        SerializablePowerEntries, SerializablePowerEntry, SerializableSupplementalData,
    };
    use proofs::proofs::common::bundle::UnifiedProofBundle;
    use std::time::SystemTime;

    fn create_test_certificate(instance_id: u64, epochs: Vec<i64>) -> CertificateEntry {
        use multihash_codetable::{Code, MultihashDigest};

        let power_table_cid = cid::Cid::new_v1(0x55, Code::Blake2b256.digest(b"test")).to_string();

        let ec_chain = epochs
            .into_iter()
            .map(|epoch| SerializableECChainEntry {
                epoch,
                key: vec!["0".to_string()],
                power_table: power_table_cid.clone(),
                commitments: vec![0u8; 32],
            })
            .collect();

        let serializable = SerializableCertificateEntry {
            certificate: SerializableF3Certificate {
                gpbft_instance: instance_id,
                ec_chain,
                supplemental_data: SerializableSupplementalData {
                    power_table: power_table_cid.clone(),
                    commitments: vec![0u8; 32],
                },
                signers: vec![0],
                signature: vec![],
                power_table_delta: vec![],
            },
            power_table: SerializablePowerEntries(vec![SerializablePowerEntry {
                id: 1,
                power: "1000".to_string(),
                pub_key: vec![1; 48],
            }]),
            source_rpc: "test".to_string(),
            fetched_at: SystemTime::now(),
        };

        CertificateEntry::try_from(serializable).expect("valid certificate entry")
    }

    fn create_test_epoch_proof(epoch: ChainEpoch, cert_instance: u64) -> EpochProofEntry {
        EpochProofEntry::new(
            epoch,
            UnifiedProofBundle {
                storage_proofs: vec![],
                event_proofs: vec![],
                blocks: vec![],
            },
            cert_instance,
        )
    }

    #[test]
    fn test_cache_basic_operations() {
        let config = CacheConfig {
            lookahead_instances: 10,
            retention_epochs: 5,
        };

        let cache = ProofCache::new(100, 0, config);

        assert!(cache.is_empty());
        assert_eq!(cache.epoch_proof_count(), 0);
        assert_eq!(cache.certificate_count(), 0);

        // Insert certificates
        let cert1 = create_test_certificate(5, vec![100, 101, 102]);
        let cert2 = create_test_certificate(6, vec![102, 103]);
        cache.insert_certificate(cert1).unwrap();
        cache.insert_certificate(cert2).unwrap();

        assert_eq!(cache.certificate_count(), 2);
        assert!(cache.contains_certificate(5));
        assert!(cache.contains_certificate(6));

        // Insert epoch proofs
        let proofs = vec![
            create_test_epoch_proof(100, 5),
            create_test_epoch_proof(101, 5),
            create_test_epoch_proof(102, 5),
        ];
        cache.insert_epoch_proofs(proofs).unwrap();

        assert_eq!(cache.epoch_proof_count(), 3);
        assert!(cache.contains_epoch_proof(100));
        assert!(cache.contains_epoch_proof(101));
        assert!(cache.contains_epoch_proof(102));
    }

    #[test]
    fn test_get_epoch_proof_with_certificate() {
        let config = CacheConfig {
            lookahead_instances: 10,
            retention_epochs: 5,
        };

        let cache = ProofCache::new(100, 0, config);

        // Insert certificates
        let cert1 = create_test_certificate(5, vec![100, 101, 102]);
        let cert2 = create_test_certificate(6, vec![102, 103]);
        cache.insert_certificate(cert1).unwrap();
        cache.insert_certificate(cert2).unwrap();

        // Insert epoch proof
        let proof = create_test_epoch_proof(101, 5);
        cache.insert_epoch_proofs(vec![proof]).unwrap();

        // Get with certificate
        let result = cache.get_epoch_proof_with_certificate(101);
        assert!(result.is_some());

        let entry = result.unwrap();
        assert_eq!(entry.epoch, 101);
        assert_eq!(entry.certificate.gpbft_instance, 5);
        assert!(!entry.finalized_tipsets.is_empty());
    }

    #[test]
    fn test_cache_cleanup() {
        let config = CacheConfig {
            lookahead_instances: 10,
            retention_epochs: 2,
        };

        let cache = ProofCache::new(100, 0, config);

        // Insert certificates
        let cert1 = create_test_certificate(5, vec![100, 101, 102]);
        let cert2 = create_test_certificate(6, vec![102, 103, 104]);
        let cert3 = create_test_certificate(7, vec![104, 105]);
        cache.insert_certificate(cert1).unwrap();
        cache.insert_certificate(cert2).unwrap();
        cache.insert_certificate(cert3).unwrap();

        // Insert epoch proofs
        let proofs = vec![
            create_test_epoch_proof(100, 5),
            create_test_epoch_proof(101, 5),
            create_test_epoch_proof(102, 5),
            create_test_epoch_proof(103, 6),
            create_test_epoch_proof(104, 6),
        ];
        cache.insert_epoch_proofs(proofs).unwrap();

        assert_eq!(cache.epoch_proof_count(), 5);

        // Mark epoch 104, instance 7 as committed (retention is 2)
        // Should remove epochs < 102 (i.e., 100, 101)
        cache.mark_committed(104, 7).unwrap();

        assert_eq!(cache.epoch_proof_count(), 3); // 102, 103, 104 remain
        assert!(!cache.contains_epoch_proof(100));
        assert!(!cache.contains_epoch_proof(101));
        assert!(cache.contains_epoch_proof(102));
        assert!(cache.contains_epoch_proof(103));
        assert!(cache.contains_epoch_proof(104));

        // Certificate 5 might be removed if no longer referenced
        // (depends on which proofs still reference it)
    }

    #[test]
    fn test_highest_cached_epoch() {
        let config = CacheConfig {
            lookahead_instances: 10,
            retention_epochs: 5,
        };

        let cache = ProofCache::new(100, 0, config);

        assert_eq!(cache.highest_cached_epoch(), None);

        // Insert certificates first
        cache
            .insert_certificate(create_test_certificate(5, vec![100]))
            .unwrap();
        cache
            .insert_certificate(create_test_certificate(6, vec![101]))
            .unwrap();

        cache
            .insert_epoch_proofs(vec![create_test_epoch_proof(100, 5)])
            .unwrap();
        assert_eq!(cache.highest_cached_epoch(), Some(100));

        cache
            .insert_epoch_proofs(vec![create_test_epoch_proof(105, 5)])
            .unwrap();
        assert_eq!(cache.highest_cached_epoch(), Some(105));

        cache
            .insert_epoch_proofs(vec![create_test_epoch_proof(103, 5)])
            .unwrap();
        assert_eq!(cache.highest_cached_epoch(), Some(105));
    }
}
