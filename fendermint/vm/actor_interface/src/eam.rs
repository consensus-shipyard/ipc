// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::{
    strict_bytes,
    tuple::{Deserialize_tuple, Serialize_tuple},
};
use fvm_shared::{address::Address, ActorID, METHOD_CONSTRUCTOR};

define_singleton!(EAM {
    id: 10,
    code_id: 15
});

/// Ethereum Address Manager actor methods available.
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Create = 2,
    Create2 = 3,
    CreateExternal = 4,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EthAddress(#[serde(with = "strict_bytes")] pub [u8; 20]);

impl EthAddress {
    /// Returns an EVM-form ID address from actor ID.
    ///
    /// This is copied from the `evm` actor library.
    pub fn from_id(id: u64) -> EthAddress {
        let mut bytes = [0u8; 20];
        bytes[0] = 0xff;
        bytes[12..].copy_from_slice(&id.to_be_bytes());
        EthAddress(bytes)
    }
}

/// Helper to read return value from contract creation.
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct CreateReturn {
    pub actor_id: ActorID,
    pub robust_address: Option<Address>,
    pub eth_address: EthAddress,
}
