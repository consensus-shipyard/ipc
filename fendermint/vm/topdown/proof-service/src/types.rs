// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Types for the proof generator service

use filecoin_f3_certs::FinalityCertificate;
use fvm_shared::clock::ChainEpoch;
use proofs::proofs::common::bundle::UnifiedProofBundle;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Serializable F3 certificate for cache storage and transaction inclusion
///
/// Contains essential validated certificate data in a format that can be:
/// - Serialized for RocksDB persistence
/// - Included in consensus transactions
/// - Used for proof verification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializableF3Certificate {
    /// F3 instance ID
    pub instance_id: u64,
    /// All epochs finalized by this certificate
    pub finalized_epochs: Vec<ChainEpoch>,
    /// Power table CID (as string for serialization)
    pub power_table_cid: String,
    /// Validated BLS signature
    pub signature: Vec<u8>,
    /// Signer indices (bitfield as Vec for serialization)
    pub signers: Vec<u64>,
}

/// Entry in the proof cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// F3 instance ID this bundle proves
    pub instance_id: u64,

    /// All epochs finalized by this certificate
    pub finalized_epochs: Vec<ChainEpoch>,

    /// Typed proof bundle (storage + event proofs + witness blocks)
    pub proof_bundle: UnifiedProofBundle,

    /// Validated certificate (cryptographically verified)
    pub certificate: SerializableF3Certificate,

    /// Metadata
    pub generated_at: SystemTime,
    pub source_rpc: String,
}

/// Validated certificate from F3 light client
#[derive(Debug, Clone)]
pub struct ValidatedCertificate {
    pub instance_id: u64,
    pub f3_cert: FinalityCertificate,
}

impl SerializableF3Certificate {
    /// Create from a cryptographically validated F3 certificate
    pub fn from_validated(cert: &FinalityCertificate) -> Self {
        Self {
            instance_id: cert.gpbft_instance,
            finalized_epochs: cert.ec_chain.suffix().iter().map(|ts| ts.epoch).collect(),
            power_table_cid: cert.supplemental_data.power_table.to_string(),
            signature: cert.signature.clone(),
            signers: cert.signers.iter().collect(),
        }
    }
}

impl From<&FinalityCertificate> for SerializableF3Certificate {
    fn from(cert: &FinalityCertificate) -> Self {
        Self::from_validated(cert)
    }
}

impl CacheEntry {
    /// Create a new cache entry from a validated F3 certificate and proof bundle
    ///
    /// # Arguments
    /// * `f3_cert` - Cryptographically validated F3 certificate
    /// * `proof_bundle` - Generated proof bundle (typed)
    /// * `source_rpc` - RPC URL where certificate was fetched from
    pub fn new(
        f3_cert: &FinalityCertificate,
        proof_bundle: UnifiedProofBundle,
        source_rpc: String,
    ) -> Self {
        let certificate = SerializableF3Certificate::from(f3_cert);
        let instance_id = certificate.instance_id;
        let finalized_epochs = certificate.finalized_epochs.clone();

        Self {
            instance_id,
            finalized_epochs,
            proof_bundle,
            certificate,
            generated_at: SystemTime::now(),
            source_rpc,
        }
    }

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
    // Skipped since it requires complex setup with real ProofBundle from integration tests
    #[ignore]
    #[test]
    fn test_cache_entry_epoch_helpers() {
        // Note: Re-enable with real ProofBundle from integration test data
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
