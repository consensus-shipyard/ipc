// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! F3 (Fast Finality) related message types for Lotus RPC

use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

use crate::lotus::message::CIDMap;

/// Response from F3.GetLatestCertificate RPC call
/// This matches the actual Lotus API structure (gpbft.FinalityCertificate)
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct F3CertificateResponse {
    /// GPBFT instance number
    #[serde(rename = "GPBFTInstance")]
    pub gpbft_instance: u64,
    /// EC chain - array of tipsets
    #[serde(rename = "ECChain")]
    pub ec_chain: Vec<ECChainEntry>,
    /// Supplemental data
    pub supplemental_data: SupplementalData,
    /// Signers bitmap
    pub signers: Vec<u64>,
    /// Aggregated signature (base64 encoded string)
    pub signature: String,
}

/// EC Chain entry in the finality certificate
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ECChainEntry {
    /// Tipset keys
    pub key: Vec<CIDMap>,
    /// Epoch/height
    pub epoch: ChainEpoch,
    /// Power table CID
    pub power_table: CIDMap,
    /// Commitments
    pub commitments: String,
}

/// Supplemental data in the certificate
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SupplementalData {
    /// Commitments
    pub commitments: String,
    /// Power table CID
    pub power_table: CIDMap,
}

/// Response from F3.GetPowerTableByInstance RPC call
/// This is returned as a direct array of power entries (gpbft.PowerEntries)
pub type F3PowerTableResponse = Vec<F3PowerEntry>;

/// Power table entry for F3 consensus
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct F3PowerEntry {
    /// Validator ID
    #[serde(rename = "ID")]
    pub id: u64,
    /// Power/weight of this validator (string in API response)
    #[serde(rename = "Power")]
    pub power: String,
    /// Public key (base64 encoded)
    #[serde(rename = "PubKey")]
    pub pub_key: String,
}
