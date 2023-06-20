// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::multihash::MultihashDigest;
use fvm_ipld_encoding::{
    strict_bytes,
    tuple::{Deserialize_tuple, Serialize_tuple},
};
use fvm_shared::{
    address::{Address, Error, SECP_PUB_LEN},
    ActorID, METHOD_CONSTRUCTOR,
};

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
    pub fn from_id(id: u64) -> Self {
        let mut bytes = [0u8; 20];
        bytes[0] = 0xff;
        bytes[12..].copy_from_slice(&id.to_be_bytes());
        Self(bytes)
    }

    /// Hash the public key according to the Ethereum convention.
    pub fn new_secp256k1(pubkey: &[u8]) -> Result<Self, Error> {
        if pubkey.len() != SECP_PUB_LEN {
            return Err(Error::InvalidSECPLength(pubkey.len()));
        }
        let mut hash20 = [0u8; 20];
        // Based on [ethers::core::types::Signature]
        let hash32 = cid::multihash::Code::Keccak256.digest(&pubkey[1..]);
        hash20.copy_from_slice(&hash32.digest()[12..]);
        Ok(Self(hash20))
    }
}

impl From<EthAddress> for Address {
    fn from(value: EthAddress) -> Address {
        if value.0[0] == 0xff {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&value.0[12..]);
            let id = u64::from_be_bytes(bytes);
            Address::new_id(id)
        } else {
            Address::new_delegated(EAM_ACTOR_ID, &value.0).expect("EthAddress is delegated")
        }
    }
}

/// Helper to read return value from contract creation.
#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct CreateReturn {
    pub actor_id: ActorID,
    pub robust_address: Option<Address>,
    pub eth_address: EthAddress,
}

impl CreateReturn {
    /// Delegated EAM address of the EVM actor, which can be used to invoke the contract.
    pub fn delegated_address(&self) -> Address {
        Address::new_delegated(EAM_ACTOR_ID, &self.eth_address.0).expect("ETH address should work")
    }
}
