// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! F3 (Fast Finality) related message types for Lotus RPC

use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

use crate::lotus::message::CIDMap;

/// Response from F3.GetCertificate RPC call
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct F3CertificateResponse {
    /// F3 instance ID
    pub instance_id: u64,
    /// Epoch/height this certificate finalizes
    pub epoch: ChainEpoch,
    /// CID of the power table used for this certificate
    pub power_table_cid: CIDMap,
    /// Aggregated signature from F3 participants
    pub signature: Vec<u8>,
    /// Raw certificate data for verification
    pub certificate_data: Vec<u8>,
}

/// Response from F3.GetPowerTable RPC call
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct F3PowerTableResponse {
    /// List of power entries
    pub entries: Vec<F3PowerEntry>,
}

/// Power table entry for F3 consensus
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct F3PowerEntry {
    /// Public key of the validator (base64 encoded)
    pub public_key: String,
    /// Voting power of the validator
    pub power: u64,
}

/// Response from F3.GetInstanceID RPC call
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct F3InstanceIDResponse {
    pub instance_id: u64,
}
