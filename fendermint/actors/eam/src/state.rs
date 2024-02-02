// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::{ActorError, Map2, MapKey, DEFAULT_HAMT_CONFIG};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub type DeployerMap<BS> = Map2<BS, Address, ()>;

/// The permission mode for controlling who can deploy contracts
#[derive(Debug, Clone, PartialEq)]
enum PermissionMode {
    /// No restriction, everyone can deploy
    NoRestriction,
    /// Only whitelisted addresses can deploy
    Whitelist(Cid), // HAMT[Address]()
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct State {
    permission_mode: PermissionMode,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS, deployers: Vec<Address>) -> Result<State, ActorError> {
        let mode = if deployers.is_empty() {
            PermissionMode::NoRestriction
        } else {
            let mut deployers_map = DeployerMap::empty(store, DEFAULT_HAMT_CONFIG, "empty");
            for d in deployers {
                deployers_map.set(&d, ())?;
            }
            PermissionMode::Whitelist(deployers_map.flush()?)
        };
        Ok(State {
            permission_mode: mode,
        })
    }

    pub fn can_deploy(
        &self,
        store: &impl Blockstore,
        deployer: &Address,
    ) -> Result<bool, ActorError> {
        Ok(match &self.permission_mode {
            PermissionMode::NoRestriction => true,
            PermissionMode::Whitelist(cid) => {
                let deployers = DeployerMap::load(store, cid, DEFAULT_HAMT_CONFIG, "verifiers")?;
                deployers.contains_key(deployer)?
            }
        })
    }
}

const NO_RESTRICTION_MODE: u8 = 0;
const WHITELIST_MODE: u8 = 1;

impl Serialize for PermissionMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PermissionMode::NoRestriction => {
                let inner: (_, Vec<u8>) = (NO_RESTRICTION_MODE, vec![]);
                Serialize::serialize(&inner, serde_tuple::Serializer(serializer))
            }
            PermissionMode::Whitelist(cid) => {
                let inner = (WHITELIST_MODE, cid.to_bytes());
                Serialize::serialize(&inner, serde_tuple::Serializer(serializer))
            }
        }
    }
}

impl<'de> Deserialize<'de> for PermissionMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (mode, bytes): (u8, Vec<u8>) =
            Deserialize::deserialize(serde_tuple::Deserializer(deserializer))?;
        Ok(match mode {
            NO_RESTRICTION_MODE => PermissionMode::NoRestriction,
            WHITELIST_MODE => PermissionMode::Whitelist(
                Cid::from_bytes(&bytes).map_err(|_| Error::custom("invalid cid"))?,
            ),
            _ => return Err(Error::custom("invalid permission mode")),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::state::PermissionMode;
    use cid::Cid;

    #[test]
    fn test_serialization() {
        let p = PermissionMode::NoRestriction;
        let v = fvm_ipld_encoding::to_vec(&p).unwrap();

        let dp: PermissionMode = fvm_ipld_encoding::from_slice(&v).unwrap();
        assert_eq!(dp, p);

        let p = PermissionMode::Whitelist(Cid::default());
        let v = fvm_ipld_encoding::to_vec(&p).unwrap();

        let dp: PermissionMode = fvm_ipld_encoding::from_slice(&v).unwrap();
        assert_eq!(dp, p)
    }
}
