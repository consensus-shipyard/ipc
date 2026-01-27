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
use std::ops::Deref;
use std::time::SystemTime;

/// Parse a 32-byte slice into an H256 hash
fn parse_commitments(bytes: &[u8]) -> Result<H256> {
    if bytes.len() != 32 {
        bail!("Commitments must be exactly 32 bytes, got {}", bytes.len());
    }
    Ok(H256::from_slice(bytes))
}

/// Parse a string as a BigInt with context
fn parse_bigint(s: &str, context: &str) -> Result<BigInt> {
    s.parse::<BigInt>()
        .with_context(|| format!("Invalid BigInt for {}: {}", context, s))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FinalizedTipsets(Vec<FinalizedTipset>);

impl Deref for FinalizedTipsets {
    type Target = Vec<FinalizedTipset>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FinalizedTipsets {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn last(&self) -> Option<&FinalizedTipset> {
        self.0.last()
    }

    /// Merge two ECChains into a single FinalizedTipsets
    pub fn merge(a: &ECChain, b: &ECChain) -> Self {
        Self(
            a.iter()
                .chain(b.iter())
                .map(FinalizedTipset::from)
                .collect(),
        )
    }
}

impl From<&[Tipset]> for FinalizedTipsets {
    /// Convert from slice of F3 Tipsets
    fn from(tipsets: &[Tipset]) -> Self {
        Self(tipsets.iter().map(FinalizedTipset::from).collect())
    }
}

impl From<&ECChain> for FinalizedTipsets {
    /// Convert from F3 ECChain
    fn from(ec_chain: &ECChain) -> Self {
        Self(ec_chain.iter().map(FinalizedTipset::from).collect())
    }
}

impl TryFrom<&[proofs::client::types::ApiTipset]> for FinalizedTipsets {
    type Error = anyhow::Error;

    /// Convert from slice of ApiTipsets
    fn try_from(tipsets: &[proofs::client::types::ApiTipset]) -> Result<Self> {
        tipsets
            .iter()
            .map(FinalizedTipset::try_from)
            .collect::<Result<Vec<_>>>()
            .map(Self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FinalizedTipset {
    /// The epoch of the tipset
    pub epoch: i64,
    /// Canonically ordered concatenated block-header CIDs
    pub block_cids: Vec<u8>,
}

impl FinalizedTipset {
    /// Verify this tipset matches another (e.g., fetched from RPC)
    ///
    /// Returns an error with details if they don't match.
    pub fn verify_matches(&self, other: &Self) -> Result<()> {
        if self.epoch != other.epoch || self.block_cids != other.block_cids {
            bail!(
                "Tipset mismatch: expected (epoch={}, cids={:x?}) got (epoch={}, cids={:x?})",
                self.epoch,
                self.block_cids,
                other.epoch,
                other.block_cids
            );
        }
        Ok(())
    }
}

impl From<&Tipset> for FinalizedTipset {
    /// Convert from F3 library's Tipset.
    /// The key field is already concatenated bytes.
    fn from(tipset: &Tipset) -> Self {
        Self {
            epoch: tipset.epoch,
            block_cids: tipset.key.clone(),
        }
    }
}

impl TryFrom<&proofs::client::types::ApiTipset> for FinalizedTipset {
    type Error = anyhow::Error;

    /// Convert from proofs library's ApiTipset.
    /// Follows F3's convert_tipset_key pattern.
    fn try_from(api_tipset: &proofs::client::types::ApiTipset) -> Result<Self> {
        let mut block_cids = Vec::new();
        for cid_map in &api_tipset.cids {
            let cid = Cid::try_from(cid_map.cid.as_str())?;
            block_cids.extend(cid.to_bytes());
        }
        Ok(Self {
            epoch: api_tipset.height,
            block_cids,
        })
    }
}

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

        let commitments = parse_commitments(&self.commitments)?;

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
        let commitments = parse_commitments(&self.commitments)?;
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
        let power_delta = parse_bigint(
            &self.power_delta,
            &format!("participant {}", self.participant_id),
        )?;

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
        let power = parse_bigint(&self.power, &format!("participant {}", self.id))?;

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

/// Entry in the epoch proof cache (keyed by epoch)
///
/// This is the primary cache entry that consumers will query.
/// It contains the proof for a single epoch and references to the
/// certificates needed for verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochProofEntry {
    /// The chain epoch at which the storage modifications has happened and events were emitted
    pub epoch: ChainEpoch,

    /// The proof bundle for this epoch
    pub proof_bundle: UnifiedProofBundle,

    /// Instance ID of the certificate that contains both this and the next tipset's epoch
    pub cert_instance: u64,

    /// Metadata
    pub generated_at: SystemTime,
}

impl EpochProofEntry {
    pub fn new(epoch: ChainEpoch, proof_bundle: UnifiedProofBundle, cert_instance: u64) -> Self {
        Self {
            epoch,
            proof_bundle,
            cert_instance,
            generated_at: SystemTime::now(),
        }
    }
}

/// Certificate entry for the certificate store (keyed by instance ID)
///
/// Certificates are stored separately to avoid duplication when multiple
/// epochs reference the same certificate.
#[derive(Debug, Clone)]
pub struct CertificateEntry {
    /// The validated F3 certificate
    pub certificate: FinalityCertificate,

    /// Power table after applying this certificate's power_table_delta
    pub power_table: PowerEntries,

    /// Source RPC endpoint
    pub source_rpc: String,

    /// When this certificate was fetched
    pub fetched_at: SystemTime,
}

impl CertificateEntry {
    pub fn new(
        certificate: FinalityCertificate,
        power_table: PowerEntries,
        source_rpc: String,
    ) -> Self {
        Self {
            certificate,
            power_table,
            source_rpc,
            fetched_at: SystemTime::now(),
        }
    }

    /// Get the instance ID of this certificate
    pub fn instance_id(&self) -> u64 {
        self.certificate.gpbft_instance
    }
}

/// Serializable version of CertificateEntry for disk persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableCertificateEntry {
    pub certificate: SerializableF3Certificate,
    pub power_table: SerializablePowerEntries,
    pub source_rpc: String,
    pub fetched_at: SystemTime,
}

impl From<&CertificateEntry> for SerializableCertificateEntry {
    fn from(entry: &CertificateEntry) -> Self {
        Self {
            certificate: SerializableF3Certificate::from(&entry.certificate),
            power_table: SerializablePowerEntries::from(&entry.power_table),
            source_rpc: entry.source_rpc.clone(),
            fetched_at: entry.fetched_at,
        }
    }
}

impl TryFrom<SerializableCertificateEntry> for CertificateEntry {
    type Error = anyhow::Error;

    fn try_from(entry: SerializableCertificateEntry) -> Result<Self> {
        Ok(Self {
            certificate: entry.certificate.try_into_certificate()?,
            power_table: entry.power_table.into_power_entries()?,
            source_rpc: entry.source_rpc,
            fetched_at: entry.fetched_at,
        })
    }
}

/// Result of looking up an epoch proof with its certificates
///
/// This is what consumers receive when they query for an epoch's proof.
/// It includes everything needed for verification.
#[derive(Debug, Clone)]
pub struct EpochProofWithCertificate {
    /// The chain epoch at which the storage modifications has happened and events were emitted
    pub epoch: ChainEpoch,

    /// The proof bundle
    pub proof_bundle: UnifiedProofBundle,

    /// The certificate that contains both this and the next tipset's epoch
    pub certificate: FinalityCertificate,

    pub finalized_tipsets: FinalizedTipsets,
}

impl EpochProofWithCertificate {
    /// Create from an epoch proof entry and its referenced certificate
    pub fn new(proof_entry: &EpochProofEntry, cert_entry: &CertificateEntry) -> Self {
        let finalized_tipsets = FinalizedTipsets::from(&cert_entry.certificate.ec_chain);
        Self {
            epoch: proof_entry.epoch,
            proof_bundle: proof_entry.proof_bundle.clone(),
            certificate: cert_entry.certificate.clone(),
            finalized_tipsets,
        }
    }
}

/// Combined entry for cache inspection
///
/// This combines a certificate with an optional proof bundle for display purposes.
/// Used by CLI tools to inspect the cache contents.
#[derive(Debug, Clone)]
pub struct CombinedCacheEntry {
    /// The F3 certificate
    pub certificate: FinalityCertificate,
    /// Optional proof bundle (if available for this certificate's instance)
    pub proof_bundle: Option<UnifiedProofBundle>,
    /// When the certificate was fetched
    pub generated_at: SystemTime,
    /// Source RPC endpoint
    pub source_rpc: String,
}

impl CombinedCacheEntry {
    /// Get the instance ID from the certificate
    pub fn instance_id(&self) -> u64 {
        self.certificate.gpbft_instance
    }

    /// Get the finalized epochs from the certificate's EC chain
    pub fn finalized_epochs(&self) -> Vec<ChainEpoch> {
        self.certificate.ec_chain.iter().map(|t| t.epoch).collect()
    }
}
