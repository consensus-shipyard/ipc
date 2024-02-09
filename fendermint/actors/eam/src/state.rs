// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::{ActorError, Map2, DEFAULT_HAMT_CONFIG};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use serde::{Deserialize, Serialize};

pub type DeployerMap<BS> = Map2<BS, Address, ()>;

/// The args used to create the permission mode in storage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PermissionModeParams {
    /// No restriction, everyone can deploy
    Unrestricted,
    /// Only whitelisted addresses can deploy
    AllowList(Vec<Address>),
}

/// The permission mode for controlling who can deploy contracts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum PermissionMode {
    /// No restriction, everyone can deploy
    Unrestricted,
    /// Only whitelisted addresses can deploy
    AllowList(Cid), // HAMT[Address]()
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct State {
    permission_mode: PermissionMode,
}

impl State {
    pub fn new<BS: Blockstore>(
        store: &BS,
        args: PermissionModeParams,
    ) -> Result<State, ActorError> {
        let permission_mode = match args {
            PermissionModeParams::Unrestricted => PermissionMode::Unrestricted,
            PermissionModeParams::AllowList(deployers) => {
                let mut deployers_map = DeployerMap::empty(store, DEFAULT_HAMT_CONFIG, "empty");
                for d in deployers {
                    deployers_map.set(&d, ())?;
                }
                PermissionMode::AllowList(deployers_map.flush()?)
            }
        };
        Ok(State { permission_mode })
    }

    pub fn can_deploy(
        &self,
        store: &impl Blockstore,
        deployer: &Address,
    ) -> Result<bool, ActorError> {
        Ok(match &self.permission_mode {
            PermissionMode::Unrestricted => true,
            PermissionMode::AllowList(cid) => {
                let deployers = DeployerMap::load(store, cid, DEFAULT_HAMT_CONFIG, "verifiers")?;
                deployers.contains_key(deployer)?
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::state::PermissionMode;
    use cid::Cid;

    #[test]
    fn test_serialization() {
        let p = PermissionMode::Unrestricted;
        let v = fvm_ipld_encoding::to_vec(&p).unwrap();

        let dp: PermissionMode = fvm_ipld_encoding::from_slice(&v).unwrap();
        assert_eq!(dp, p);

        let p = PermissionMode::AllowList(Cid::default());
        let v = fvm_ipld_encoding::to_vec(&p).unwrap();

        let dp: PermissionMode = fvm_ipld_encoding::from_slice(&v).unwrap();
        assert_eq!(dp, p)
    }
}
