// Copyright 2021-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::clock::ChainEpoch;

/// F3 certificate data structure
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct F3Certificate {
    /// F3 instance ID
    pub instance_id: u64,
    /// All epochs finalized by this certificate (from ECChain)
    /// Must contain at least one epoch
    pub finalized_epochs: Vec<ChainEpoch>,
    /// CID of the power table used for this certificate
    pub power_table_cid: Cid,
    /// Aggregated signature from F3 participants
    pub signature: Vec<u8>,
    /// Raw certificate data for verification (full Lotus cert with ECChain)
    pub certificate_data: Vec<u8>,
}

/// Power table entry for F3 consensus
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct PowerEntry {
    /// Public key of the validator
    pub public_key: Vec<u8>,
    /// Voting power of the validator
    pub power: u64,
}

/// Constructor parameters for the F3 certificate manager
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct ConstructorParams {
    /// Genesis F3 instance ID
    pub genesis_instance_id: u64,
    /// Genesis power table
    pub genesis_power_table: Vec<PowerEntry>,
}

/// Parameters for updating the F3 certificate
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct UpdateCertificateParams {
    /// New F3 certificate
    pub certificate: F3Certificate,
}

/// Response containing the latest F3 state
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct GetCertificateResponse {
    /// Current F3 instance ID
    pub current_instance_id: u64,
    /// Latest finalized height
    pub latest_finalized_height: ChainEpoch,
}

/// Response containing the F3 instance information
#[derive(Deserialize_tuple, Serialize_tuple, Debug, Clone, PartialEq, Eq)]
pub struct GetInstanceInfoResponse {
    /// Genesis F3 instance ID
    pub genesis_instance_id: u64,
    /// Genesis power table
    pub genesis_power_table: Vec<PowerEntry>,
    /// Latest finalized height
    pub latest_finalized_height: ChainEpoch,
}
