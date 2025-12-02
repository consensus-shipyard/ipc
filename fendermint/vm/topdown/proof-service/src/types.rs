// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Types for the proof generator service

use anyhow::{bail, Context, Result};
use filecoin_f3_certs::{FinalityCertificate, PowerTableDelta, PowerTableDiff};
use filecoin_f3_gpbft::{self, Cid, ECChain, PowerEntries, PowerEntry, SupplementalData, Tipset};
use fvm_ipld_bitfield::BitField;
use fvm_shared::clock::ChainEpoch;
use keccak_hash::H256;
use num_bigint::BigInt;
use proofs::proofs::common::bundle::UnifiedProofBundle;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Serializable EC Chain entry
///
/// Represents a single tipset in the finalized chain.
/// Matches the structure from filecoin_f3_gpbft::TipSet
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializableECChainEntry {
    /// Tipset epoch
    pub epoch: ChainEpoch,
    /// Tipset key (CIDs as strings for serialization)
    pub key: Vec<String>,
    /// Power table CID (as string for serialization)
    pub power_table: String,
    /// Commitments (32-byte hash as bytes)
    pub commitments: Vec<u8>,
}

impl SerializableECChainEntry {
    fn into_tipset(self) -> Result<Tipset> {
        let key = self
            .key
            .into_iter()
            .map(|byte| {
                byte.parse::<u8>()
                    .with_context(|| format!("Invalid tipset key byte: {}", byte))
            })
            .collect::<Result<Vec<_>>>()?;

        let power_table = self
            .power_table
            .parse::<Cid>()
            .context("Invalid power table CID in ECChain entry")?;

        if self.commitments.len() != 32 {
            bail!("Commitments must be 32 bytes");
        }
        let commitments = H256::from_slice(&self.commitments);

        Ok(Tipset {
            epoch: self.epoch,
            key,
            power_table,
            commitments,
        })
    }
}

/// Serializable supplemental data
///
/// Matches the structure from filecoin_f3_gpbft::SupplementalData
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializableSupplementalData {
    /// Power table CID (as string for serialization)
    pub power_table: String,
    /// Commitments (32-byte hash as bytes)
    pub commitments: Vec<u8>,
}

impl SerializableSupplementalData {
    fn into_supplemental_data(self) -> Result<SupplementalData> {
        if self.commitments.len() != 32 {
            bail!("Supplemental commitments must be 32 bytes");
        }
        let commitments = H256::from_slice(&self.commitments);
        let power_table = self
            .power_table
            .parse::<Cid>()
            .context("Invalid power table CID in supplemental data")?;

        Ok(SupplementalData {
            commitments,
            power_table,
        })
    }
}

/// Serializable power table delta entry
///
/// Matches the structure from filecoin_f3_gpbft::PowerTableDelta
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializablePowerTableDelta {
    /// Participant ID
    pub participant_id: u64,
    /// Power delta as string (signed - can be negative for decreases)
    pub power_delta: String,
    /// Signing key (public key bytes)
    pub signing_key: Vec<u8>,
}

impl SerializablePowerTableDelta {
    fn into_power_table_delta(self) -> Result<PowerTableDelta> {
        let power_delta = self.power_delta.parse::<BigInt>().with_context(|| {
            format!(
                "Invalid power delta for participant {}",
                self.participant_id
            )
        })?;

        Ok(PowerTableDelta {
            participant_id: self.participant_id,
            power_delta,
            signing_key: filecoin_f3_gpbft::PubKey(self.signing_key),
        })
    }
}

/// Serializable power table entry
///
/// Matches the structure from filecoin_f3_gpbft::PowerEntry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializablePowerEntry {
    /// Validator ID
    pub id: u64,
    /// Power/weight as string (BigInt)
    pub power: String,
    /// Public key bytes
    pub pub_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializablePowerEntries(pub Vec<SerializablePowerEntry>);

impl SerializablePowerEntry {
    fn into_power_entry(self) -> Result<PowerEntry> {
        let power = self
            .power
            .parse::<BigInt>()
            .with_context(|| format!("Invalid power value for participant {}", self.id))?;

        Ok(PowerEntry {
            id: self.id,
            power,
            pub_key: filecoin_f3_gpbft::PubKey(self.pub_key),
        })
    }
}

impl SerializablePowerEntries {
    pub fn into_power_entries(self) -> Result<PowerEntries> {
        let entries = self
            .0
            .into_iter()
            .map(|entry| entry.into_power_entry())
            .collect::<Result<Vec<_>>>()?;
        Ok(PowerEntries(entries))
    }
}

/// Serializable F3 certificate for cache storage and transaction inclusion
///
/// Contains essential validated certificate data in a format that can be:
/// - Serialized for RocksDB persistence
/// - Included in consensus transactions
/// - Used for proof verification
///
/// This structure matches filecoin_f3_certs::FinalityCertificate field names
/// exactly, but uses serializable types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializableF3Certificate {
    /// The GPBFT instance to which this finality certificate corresponds
    /// Matches: FinalityCertificate.gpbft_instance
    pub gpbft_instance: u64,

    /// The ECChain finalized during this instance
    /// Matches: FinalityCertificate.ec_chain
    /// Structure: [base, suffix...]
    /// - base: last tipset finalized in previous instance
    /// - suffix: new tipsets being finalized in this instance (may be empty)
    pub ec_chain: Vec<SerializableECChainEntry>,

    /// Additional data signed by the participants in this instance
    /// Matches: FinalityCertificate.supplemental_data
    pub supplemental_data: SerializableSupplementalData,

    /// Indexes in the base power table of the certifiers (bitfield)
    /// Matches: FinalityCertificate.signers
    pub signers: Vec<u64>,

    /// Aggregated signature of the certifiers
    /// Matches: FinalityCertificate.signature
    pub signature: Vec<u8>,

    /// Changes between the power table used to validate this finality certificate
    /// and the power table used to validate the next finality certificate
    /// Matches: FinalityCertificate.power_table_delta
    pub power_table_delta: Vec<SerializablePowerTableDelta>,
}

impl SerializableF3Certificate {
    /// Get all finalized epochs from the ec_chain
    ///
    /// Returns epochs from both base and suffix tipsets
    pub fn finalized_epochs(&self) -> Vec<ChainEpoch> {
        self.ec_chain.iter().map(|entry| entry.epoch).collect()
    }

    pub fn try_into_certificate(self) -> Result<FinalityCertificate> {
        let tipsets = self
            .ec_chain
            .into_iter()
            .map(|entry| entry.into_tipset())
            .collect::<Result<Vec<_>>>()?;
        let ec_chain = ECChain::new_unvalidated(tipsets);

        ec_chain.validate().context("Failed to validate EC chain")?;

        let supplemental_data = self.supplemental_data.into_supplemental_data()?;
        let signers = BitField::try_from_bits(self.signers.iter().copied())
            .context("Failed to rebuild signers bitfield")?;
        let power_table_delta = self
            .power_table_delta
            .into_iter()
            .map(|delta| delta.into_power_table_delta())
            .collect::<Result<PowerTableDiff>>()?;

        Ok(FinalityCertificate {
            gpbft_instance: self.gpbft_instance,
            ec_chain,
            supplemental_data,
            signers,
            signature: self.signature,
            power_table_delta,
        })
    }
}

impl From<&FinalityCertificate> for SerializableF3Certificate {
    fn from(cert: &FinalityCertificate) -> Self {
        // Convert EC chain to serializable format
        let ec_chain = cert
            .ec_chain
            .iter()
            .map(|ts| SerializableECChainEntry {
                epoch: ts.epoch,
                key: ts.key.iter().map(|cid| cid.to_string()).collect(),
                power_table: ts.power_table.to_string(),
                commitments: ts.commitments.as_bytes().to_vec(),
            })
            .collect();

        // Convert supplemental data
        let supplemental_data = SerializableSupplementalData {
            power_table: cert.supplemental_data.power_table.to_string(),
            commitments: cert.supplemental_data.commitments.as_bytes().to_vec(),
        };

        // Convert power table delta
        let power_table_delta = cert
            .power_table_delta
            .iter()
            .map(|delta| SerializablePowerTableDelta {
                participant_id: delta.participant_id,
                power_delta: delta.power_delta.to_string(),
                signing_key: delta.signing_key.0.clone(),
            })
            .collect();

        Self {
            gpbft_instance: cert.gpbft_instance,
            ec_chain,
            supplemental_data,
            signers: cert.signers.iter().collect(),
            signature: cert.signature.clone(),
            power_table_delta,
        }
    }
}

/// Entry in the proof cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableCacheEntry {
    pub proof_bundle: Option<UnifiedProofBundle>,
    pub certificate: SerializableF3Certificate,
    pub power_table: SerializablePowerEntries,
    pub generated_at: SystemTime,
    pub source_rpc: String,
}

impl From<&CacheEntry> for SerializableCacheEntry {
    fn from(entry: &CacheEntry) -> Self {
        Self {
            proof_bundle: entry.proof_bundle.clone(),
            certificate: SerializableF3Certificate::from(&entry.certificate),
            power_table: SerializablePowerEntries::from(&entry.power_table),
            generated_at: entry.generated_at,
            source_rpc: entry.source_rpc.clone(),
        }
    }
}

impl TryFrom<SerializableCacheEntry> for CacheEntry {
    type Error = anyhow::Error;

    fn try_from(value: SerializableCacheEntry) -> Result<Self> {
        Ok(Self {
            proof_bundle: value.proof_bundle,
            certificate: value.certificate.try_into_certificate()?,
            power_table: value.power_table.into_power_entries()?,
            generated_at: value.generated_at,
            source_rpc: value.source_rpc,
        })
    }
}

impl From<&PowerEntry> for SerializablePowerEntry {
    fn from(entry: &PowerEntry) -> Self {
        Self {
            id: entry.id,
            power: entry.power.to_string(),
            pub_key: entry.pub_key.0.clone(),
        }
    }
}

impl From<&PowerEntries> for SerializablePowerEntries {
    fn from(entries: &PowerEntries) -> Self {
        Self(entries.iter().map(SerializablePowerEntry::from).collect())
    }
}

/// Entry in the proof cache
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Typed proof bundle (storage + event proofs + witness blocks)
    /// None if the proof bundle was not generated (e.g. if the certificate has no suffix)
    pub proof_bundle: Option<UnifiedProofBundle>,

    /// Validated certificate (cryptographically verified)
    pub certificate: FinalityCertificate,

    /// Power table after applying this certificate's power_table_delta
    /// This is needed to resume F3 client state from cache
    pub power_table: PowerEntries,

    /// Metadata
    pub generated_at: SystemTime,
    pub source_rpc: String,
}

impl CacheEntry {
    /// Create a new cache entry from a validated F3 certificate and proof bundle
    pub fn new(
        certificate: FinalityCertificate,
        proof_bundle: Option<UnifiedProofBundle>,
        power_table: PowerEntries,
        source_rpc: String,
    ) -> Self {
        Self {
            proof_bundle,
            certificate,
            power_table,
            generated_at: SystemTime::now(),
            source_rpc,
        }
    }
}
