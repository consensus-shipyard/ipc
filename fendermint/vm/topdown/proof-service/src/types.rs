// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Types for the proof generator service

use fendermint_actor_f3_cert_manager::types::F3Certificate;
use filecoin_f3_certs::FinalityCertificate;
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

    /// The serialized proof bundle (CBOR encoded)
    /// We store as bytes to avoid serialization issues
    pub proof_bundle_bytes: Vec<u8>,

    /// F3 certificate raw bytes (for validation)
    /// We store as bytes since FinalityCertificate doesn't implement Serialize
    pub f3_certificate_bytes: Vec<u8>,

    /// Certificate in actor format (for updating on-chain)
    pub actor_certificate: F3Certificate,

    /// Metadata
    pub generated_at: SystemTime,
    pub source_rpc: String,
}

/// Validated certificate from parent chain
#[derive(Debug, Clone)]
pub struct ValidatedCertificate {
    pub instance_id: u64,
    pub f3_cert: FinalityCertificate,
    pub lotus_response: ipc_provider::lotus::message::f3::F3CertificateResponse,
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

    // Helper function to create test entries
    // For now, we'll skip this test since it requires complex setup with ProofBundle
    #[ignore]
    #[test]
    fn test_cache_entry_epoch_helpers() {
        // TODO: Re-enable once we have proper test utilities for ProofBundle
        /*
        let entry = CacheEntry {
            instance_id: 1,
            finalized_epochs: vec![100, 101, 102, 103],
            // Would need real ProofBundle here
            ...
        };

        assert_eq!(entry.highest_epoch(), Some(103));
        assert_eq!(entry.lowest_epoch(), Some(100));
        assert!(entry.covers_epoch(101));
        assert!(!entry.covers_epoch(99));
        assert!(!entry.covers_epoch(104));
        */
    }
}
