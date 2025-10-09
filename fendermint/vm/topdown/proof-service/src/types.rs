// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Types for the proof generator service

use fendermint_actor_f3_cert_manager::types::F3Certificate;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Entry in the proof cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// F3 instance ID this bundle proves
    pub instance_id: u64,

    /// All epochs finalized by this certificate
    pub finalized_epochs: Vec<ChainEpoch>,

    /// The complete proof bundle (will be from ipc-filecoin-proofs)
    /// For now, we'll use a placeholder until we integrate the library
    pub bundle: ProofBundlePlaceholder,

    /// Certificate in actor format (for updating on-chain)
    pub actor_certificate: F3Certificate,

    /// Metadata
    pub generated_at: SystemTime,
    pub source_rpc: String,
}

/// Placeholder for the proof bundle until we integrate ipc-filecoin-proofs
/// TODO: Replace with actual ProofBundle from ipc-filecoin-proofs library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofBundlePlaceholder {
    /// Parent height this bundle covers
    pub parent_height: u64,
    /// Placeholder data
    pub data: Vec<u8>,
}

impl CacheEntry {
    /// Get the highest epoch finalized by this certificate
    pub fn highest_epoch(&self) -> Option<ChainEpoch> {
        self.finalized_epochs.iter().max().copied()
    }

    /// Get the lowest epoch finalized by this certificate
    pub fn lowest_epoch(&self) -> Option<ChainEpoch> {
        self.finalized_epochs.iter().min().copied()
    }

    /// Check if this certificate finalizes a specific epoch
    pub fn covers_epoch(&self, epoch: ChainEpoch) -> bool {
        self.finalized_epochs.contains(&epoch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cid::Cid;
    use multihash::{Code, MultihashDigest};

    #[test]
    fn test_cache_entry_epoch_helpers() {
        let power_table_cid = Cid::new_v1(0x55, Code::Blake2b256.digest(b"test"));

        let entry = CacheEntry {
            instance_id: 1,
            finalized_epochs: vec![100, 101, 102, 103],
            bundle: ProofBundlePlaceholder {
                parent_height: 103,
                data: vec![],
            },
            actor_certificate: F3Certificate {
                instance_id: 1,
                finalized_epochs: vec![100, 101, 102, 103],
                power_table_cid,
                signature: vec![],
                certificate_data: vec![],
            },
            generated_at: SystemTime::now(),
            source_rpc: "http://test".to_string(),
        };

        assert_eq!(entry.highest_epoch(), Some(103));
        assert_eq!(entry.lowest_epoch(), Some(100));
        assert!(entry.covers_epoch(101));
        assert!(!entry.covers_epoch(99));
        assert!(!entry.covers_epoch(104));
    }
}
