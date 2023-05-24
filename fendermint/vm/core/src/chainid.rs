// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::collections::HashMap;

use cid::{multihash, multihash::MultihashDigest};
use fvm_shared::bigint::{BigInt, Integer, Sign};
use fvm_shared::chainid::ChainID;
use lazy_static::lazy_static;
use thiserror::Error;

lazy_static! {
    /// Well known Filecoin chain IDs.
    ///
    /// See all EVM chain IDs at this repo: https://github.com/ethereum-lists/chains/pull/1567
    /// For now I thought it would be enough to enumerate the Filecoin ones.
    static ref KNOWN_CHAIN_IDS: HashMap<u64, &'static str> = HashMap::from([
      (0,        ""), // Used as a default
      (314,      "filecoin"),
      (3141,     "hyperspace"),
      (31415,    "wallaby"),
      (3141592,  "butterflynet"),
      (314159,   "calibnet"),
      (31415926, "devnet"),
    ]);

    /// Reverse index over the chain IDs.
    static ref KNOWN_CHAIN_NAMES: HashMap<&'static str, u64> = KNOWN_CHAIN_IDS.iter().map(|(k, v)| (*v, *k)).collect();
}

/// Maximum value that MetaMask and other Ethereum JS tools can safely handle.
///
/// See https://github.com/ethereum/EIPs/issues/2294
pub const MAX_CHAIN_ID: u64 = 4503599627370476;

#[derive(Error, Debug)]
pub enum ChainIDError {
    /// The name was hashed to a numeric value of a well-known chain.
    /// The chances of this are low, but if it happens, try picking a different name, if possible.
    #[error("illegal name: {0} ({1})")]
    IllegalName(String, u64),
}

/// Hash the name of the chain and reduce it to a number within the acceptable range.
///
/// If the name is one of the well known ones, return that name as is.
pub fn from_str_hashed(name: &str) -> Result<ChainID, ChainIDError> {
    // TODO: If we want to use the subnet ID (e.g. "/root/foo/bar")
    // as the chain name, we should change it so the "/root" part is
    // not common to all chain, but rather be like "/filecoin/foo/bar".
    // And if someone is looking for the ID of "/filecoin" without
    // any further path (ie. the root) then we should strip the "/"
    // when looking up if it's a well known network ID.

    if let Some(chain_id) = KNOWN_CHAIN_NAMES.get(name) {
        return Ok(ChainID::from(*chain_id));
    }

    let bz = name.as_bytes();
    let digest = multihash::Code::Blake2b256.digest(bz);

    let num_digest = BigInt::from_bytes_be(Sign::Plus, digest.digest());
    let max_chain_id = BigInt::from(MAX_CHAIN_ID);

    let chain_id = num_digest.mod_floor(&max_chain_id);
    let chain_id: u64 = chain_id
        .try_into()
        .expect("modulo should be safe to convert to u64");

    if KNOWN_CHAIN_IDS.contains_key(&chain_id) {
        Err(ChainIDError::IllegalName(name.to_owned(), chain_id))
    } else {
        Ok(ChainID::from(chain_id))
    }
}

/// Anything that has a [`ChainID`].
pub trait HasChainID {
    fn chain_id(&self) -> &ChainID;
}

#[cfg(test)]
mod tests {

    use fvm_shared::chainid::ChainID;
    use quickcheck_macros::quickcheck;

    use crate::chainid::KNOWN_CHAIN_NAMES;

    use super::{from_str_hashed, MAX_CHAIN_ID};

    #[quickcheck]
    fn prop_chain_id_stable(name: String) -> bool {
        if let Ok(id1) = from_str_hashed(&name) {
            let id2 = from_str_hashed(&name).unwrap();
            return id1 == id2;
        }
        true
    }

    #[quickcheck]
    fn prop_chain_id_safe(name: String) -> bool {
        if let Ok(id) = from_str_hashed(&name) {
            let chain_id: u64 = id.into();
            return chain_id <= MAX_CHAIN_ID;
        }
        true
    }

    #[test]
    fn chain_id_ok() -> Result<(), String> {
        for name in ["test", "/root/foo/bar"] {
            if let Err(e) = from_str_hashed(name) {
                return Err(format!("failed: {name} - {e}"));
            }
        }
        Ok(())
    }

    #[test]
    fn chain_id_different() {
        let id1 = from_str_hashed("foo").unwrap();
        let id2 = from_str_hashed("bar").unwrap();
        assert_ne!(id1, id2)
    }

    #[test]
    fn chain_id_of_empty_is_zero() {
        assert_eq!(from_str_hashed("").unwrap(), ChainID::from(0))
    }

    #[test]
    fn chain_id_of_known() {
        for (name, id) in KNOWN_CHAIN_NAMES.iter() {
            assert_eq!(from_str_hashed(name).unwrap(), ChainID::from(*id))
        }
    }
}
