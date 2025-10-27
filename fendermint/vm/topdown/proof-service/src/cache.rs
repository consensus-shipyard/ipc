// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! In-memory cache for proof bundles with optional disk persistence

use crate::config::CacheConfig;
use crate::observe::ProofCached;
use crate::persistence::ProofCachePersistence;
use crate::types::CacheEntry;
use anyhow::Result;
use ipc_observability::emit;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Thread-safe in-memory cache for proof bundles
#[derive(Clone)]
pub struct ProofCache {
    /// Map: instance_id -> CacheEntry
    /// Using BTreeMap for ordered iteration
    entries: Arc<RwLock<BTreeMap<u64, CacheEntry>>>,

    /// Last committed instance ID (updated after execution)
    last_committed_instance: Arc<AtomicU64>,

    /// Configuration
    config: CacheConfig,

    /// Optional disk persistence
    persistence: Option<Arc<ProofCachePersistence>>,
}

impl ProofCache {
    /// Create a new proof cache (in-memory only)
    pub fn new(last_committed_instance: u64, config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(BTreeMap::new())),
            last_committed_instance: Arc::new(AtomicU64::new(last_committed_instance)),
            config,
            persistence: None,
        }
    }

    /// Create a new proof cache with disk persistence
    ///
    /// Loads existing entries from disk on startup.
    /// If DB is fresh, uses `initial_instance` as the starting point.
    pub fn new_with_persistence(
        config: CacheConfig,
        db_path: &Path,
        initial_instance: u64,
    ) -> Result<Self> {
        let persistence = ProofCachePersistence::open(db_path)?;

        // Load last committed from disk, or use initial_instance if DB is fresh
        let last_committed = persistence
            .load_last_committed()?
            .unwrap_or(initial_instance);

        // Load all entries from disk into memory
        let entries_vec = persistence.load_all_entries()?;
        let entries: BTreeMap<u64, CacheEntry> = entries_vec
            .into_iter()
            .map(|e| (e.instance_id, e))
            .collect();

        tracing::info!(
            last_committed,
            entry_count = entries.len(),
            "Loaded cache from disk"
        );

        Ok(Self {
            entries: Arc::new(RwLock::new(entries)),
            last_committed_instance: Arc::new(AtomicU64::new(last_committed)),
            config,
            persistence: Some(Arc::new(persistence)),
        })
    }

    /// Get the next uncommitted proof (in sequential order)
    /// Returns the proof for (last_committed + 1)
    pub fn get_next_uncommitted(&self) -> Option<CacheEntry> {
        let last_committed = self.last_committed_instance.load(Ordering::Acquire);
        let next_instance = last_committed + 1;

        self.entries.read().get(&next_instance).cloned()
    }

    /// Get proof for a specific instance ID
    pub fn get(&self, instance_id: u64) -> Option<CacheEntry> {
        self.entries.read().get(&instance_id).cloned()
    }

    /// Check if an instance is already cached
    pub fn contains(&self, instance_id: u64) -> bool {
        let result = self.entries.read().contains_key(&instance_id);

        // Record cache hit/miss
        use crate::observe::CACHE_HIT_TOTAL;
        CACHE_HIT_TOTAL
            .with_label_values(&[if result { "hit" } else { "miss" }])
            .inc();

        result
    }

    /// Insert a proof into the cache
    pub fn insert(&self, entry: CacheEntry) -> anyhow::Result<()> {
        let instance_id = entry.instance_id;

        // Check if we're within the lookahead window
        let last_committed = self.last_committed_instance.load(Ordering::Acquire);
        let max_allowed = last_committed + self.config.lookahead_instances;

        if instance_id > max_allowed {
            anyhow::bail!(
                "Instance {} exceeds lookahead window (last_committed={}, max={})",
                instance_id,
                last_committed,
                max_allowed
            );
        }

        // Insert to memory
        self.entries.write().insert(instance_id, entry.clone());

        // Persist to disk if enabled
        if let Some(persistence) = &self.persistence {
            persistence.save_entry(&entry)?;
        }

        // Emit metrics
        let cache_size = self.entries.read().len();
        let highest = self.highest_cached_instance();

        if let Some(highest_cached) = highest {
            emit(ProofCached {
                instance: instance_id,
                cache_size,
                highest_cached,
            });
        }

        // Update cache size metric
        use crate::observe::CACHE_SIZE;
        CACHE_SIZE.set(cache_size as i64);

        tracing::debug!(instance_id, cache_size, "Inserted proof into cache");

        Ok(())
    }

    /// Mark an instance as committed and trigger cleanup
    pub fn mark_committed(&self, instance_id: u64) {
        let old_value = self
            .last_committed_instance
            .swap(instance_id, Ordering::Release);

        // Save to disk if enabled
        if let Some(persistence) = &self.persistence {
            let _ = persistence.save_last_committed(instance_id);
        }

        tracing::info!(
            old_instance = old_value,
            new_instance = instance_id,
            "Updated last committed instance"
        );

        // Cleanup old instances outside retention window
        self.cleanup_old_instances(instance_id);
    }

    /// Get the current last committed instance
    pub fn last_committed_instance(&self) -> u64 {
        self.last_committed_instance.load(Ordering::Acquire)
    }

    /// Get the highest cached instance
    pub fn highest_cached_instance(&self) -> Option<u64> {
        self.entries.read().keys().max().copied()
    }

    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }

    /// Remove instances older than the retention window
    fn cleanup_old_instances(&self, current_instance: u64) {
        let retention_cutoff = current_instance.saturating_sub(self.config.retention_instances);

        // Collect IDs to remove
        let to_remove: Vec<u64> = {
            let entries = self.entries.read();
            entries
                .keys()
                .filter(|&&id| id < retention_cutoff)
                .copied()
                .collect()
        };

        if !to_remove.is_empty() {
            // Remove from memory
            {
                let mut entries = self.entries.write();
                for id in &to_remove {
                    entries.remove(id);
                }
            }

            // Remove from disk if enabled
            if let Some(persistence) = &self.persistence {
                for id in &to_remove {
                    let _ = persistence.delete_entry(*id);
                }
            }

            tracing::debug!(
                removed = to_remove.len(),
                retention_cutoff,
                "Cleaned up old cache entries"
            );
        }
    }

    /// Get all cached instance IDs (for debugging)
    pub fn cached_instances(&self) -> Vec<u64> {
        self.entries.read().keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SerializableF3Certificate;
    use proofs::proofs::common::bundle::UnifiedProofBundle;
    use std::time::SystemTime;

    fn create_test_entry(instance_id: u64, epochs: Vec<i64>) -> CacheEntry {
        CacheEntry {
            instance_id,
            finalized_epochs: epochs.clone(),
            proof_bundle: UnifiedProofBundle {
                storage_proofs: vec![],
                event_proofs: vec![],
                blocks: vec![],
            },
            certificate: SerializableF3Certificate {
                instance_id,
                finalized_epochs: epochs,
                power_table_cid: {
                    use multihash_codetable::{Code, MultihashDigest};
                    cid::Cid::new_v1(0x55, Code::Blake2b256.digest(b"test")).to_string()
                },
                signature: vec![],
                signers: vec![],
            },
            generated_at: SystemTime::now(),
            source_rpc: "test".to_string(),
        }
    }

    #[test]
    fn test_cache_basic_operations() {
        let config = CacheConfig {
            lookahead_instances: 5,
            retention_instances: 2,
            max_size_bytes: 0,
        };

        let cache = ProofCache::new(100, config);

        assert_eq!(cache.last_committed_instance(), 100);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        // Insert next instance
        let entry = create_test_entry(101, vec![200, 201, 202]);
        cache.insert(entry).unwrap();

        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());
        assert!(cache.contains(101));

        // Get next uncommitted (should be 101)
        let next = cache.get_next_uncommitted();
        assert!(next.is_some());
        assert_eq!(next.unwrap().instance_id, 101);
    }

    #[test]
    fn test_cache_lookahead_enforcement() {
        let config = CacheConfig {
            lookahead_instances: 3,
            retention_instances: 1,
            max_size_bytes: 0,
        };

        let cache = ProofCache::new(100, config);

        // Can insert within lookahead (100 + 1..=100 + 3)
        cache.insert(create_test_entry(101, vec![201])).unwrap();
        cache.insert(create_test_entry(102, vec![202])).unwrap();
        cache.insert(create_test_entry(103, vec![203])).unwrap();

        // Should fail beyond lookahead
        let result = cache.insert(create_test_entry(105, vec![205]));
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_cleanup() {
        let config = CacheConfig {
            lookahead_instances: 10,
            retention_instances: 2,
            max_size_bytes: 0,
        };

        let cache = ProofCache::new(100, config);

        // Insert several entries
        for i in 101..=105 {
            cache.insert(create_test_entry(i, vec![i as i64])).unwrap();
        }

        assert_eq!(cache.len(), 5);

        // Mark 103 as committed (retention window is 2)
        // Should keep 101, 102, 103, 104, 105 (all within retention_cutoff = 103 - 2 = 101)
        cache.mark_committed(103);
        assert_eq!(cache.last_committed_instance(), 103);
        assert_eq!(cache.len(), 5); // All still within retention

        // Mark 105 as committed
        // Should remove 101, 102 (retention_cutoff = 105 - 2 = 103)
        cache.mark_committed(105);
        assert_eq!(cache.len(), 3); // 103, 104, 105 remain
        assert!(!cache.contains(101));
        assert!(!cache.contains(102));
        assert!(cache.contains(103));
    }

    #[test]
    fn test_cache_highest_instance() {
        let config = CacheConfig {
            lookahead_instances: 10,
            retention_instances: 2,
            max_size_bytes: 0,
        };

        let cache = ProofCache::new(100, config);

        assert_eq!(cache.highest_cached_instance(), None);

        cache.insert(create_test_entry(101, vec![201])).unwrap();
        assert_eq!(cache.highest_cached_instance(), Some(101));

        cache.insert(create_test_entry(105, vec![205])).unwrap();
        assert_eq!(cache.highest_cached_instance(), Some(105));

        cache.insert(create_test_entry(103, vec![203])).unwrap();
        assert_eq!(cache.highest_cached_instance(), Some(105));
    }
}
