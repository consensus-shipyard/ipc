// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;

/// Parameters for registering a node operator
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct RegisterNodeOperatorParams {
    /// BLS public key (must be 48 bytes)
    pub bls_pubkey: Vec<u8>,
    /// RPC URL where the operator's node can be queried for signatures
    pub rpc_url: String,
}

/// Parameters for getting operator information
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct GetOperatorInfoParams {
    /// Address of the operator
    pub address: Address,
}

/// Return type for getting operator information
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct OperatorInfo {
    /// BLS public key
    pub bls_pubkey: Vec<u8>,
    /// RPC URL
    pub rpc_url: String,
    /// Whether the operator is active
    pub active: bool,
}

/// Return type for getting all active operators
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct GetActiveOperatorsReturn {
    /// Ordered list of active operator addresses
    /// Index in this list corresponds to bit position in signature bitmap
    pub operators: Vec<Address>,
}
