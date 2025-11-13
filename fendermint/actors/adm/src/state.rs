// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::anyhow;
use cid::Cid;
use fil_actors_runtime::{runtime::Runtime, ActorError, Map2, MapKey, DEFAULT_HAMT_CONFIG};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, ActorID};
use integer_encoding::VarInt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

type MachineCodeMap<BS> = Map2<BS, Kind, Cid>;
type DeployerMap<BS> = Map2<BS, Address, ()>;
type OwnerMap<BS> = Map2<BS, Address, Vec<Metadata>>;

/// The args used to create the permission mode in storage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PermissionModeParams {
    /// No restriction, everyone can deploy.
    Unrestricted,
    /// Only whitelisted addresses can deploy.
    AllowList(Vec<Address>),
}

/// The permission mode for controlling who can deploy contracts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PermissionMode {
    /// No restriction, everyone can deploy.
    Unrestricted,
    /// Only whitelisted addresses can deploy.
    AllowList(Cid), // HAMT[Address]()
}

/// The kinds of machines available. Their code Cids are given at genesis.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Kind {
    /// An object storage bucket with S3-like key semantics.
    Bucket,
    /// An MMR timehub.
    Timehub,
}

impl MapKey for Kind {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        if let Some((result, size)) = u64::decode_var(b) {
            if size != b.len() {
                return Err(format!("trailing bytes after varint in {:?}", b));
            }
            match result {
                0 => Ok(Kind::Bucket),
                1 => Ok(Kind::Timehub),
                _ => Err(format!("failed to decode kind from {}", result)),
            }
        } else {
            Err(format!("failed to decode varint in {:?}", b))
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let int = match self {
            Self::Bucket => 0,
            Self::Timehub => 1,
        };
        Ok(int.encode_var_vec())
    }
}

impl FromStr for Kind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "bucket" => Self::Bucket,
            "timehub" => Self::Timehub,
            _ => return Err(anyhow!("invalid machine kind")),
        })
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Bucket => "bucket",
            Self::Timehub => "timehub",
        };
        write!(f, "{}", str)
    }
}

/// Machine metadata.
#[derive(Debug, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Metadata {
    /// Machine kind.
    pub kind: Kind,
    /// Machine ID address.
    pub address: Address,
    /// User-defined data.
    pub metadata: HashMap<String, String>,
}

/// ADM actor state representation.
#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The root of a HAMT[u64]Cid containing available machine codes.
    /// This is fixed at genesis.
    pub machine_codes: Cid,
    /// The permission mode controlling who can create machines.
    /// This is fixed at genesis, but in allowlist mode, the set of deployers can be changed
    /// by any member.
    /// Modeled after the IPC EAM actor.
    pub permission_mode: PermissionMode,
    /// The root of a HAMT[Address]Vec<AddressMetadata> containing address and kind metadata
    /// keyed by owner robust address.
    pub owners: Cid,
}

impl State {
    pub fn new<BS: Blockstore>(
        store: &BS,
        machine_codes: HashMap<Kind, Cid>,
        permission_mode: PermissionModeParams,
    ) -> Result<State, ActorError> {
        let mut machine_code_map = MachineCodeMap::empty(store, DEFAULT_HAMT_CONFIG, "machines");
        for (kind, code) in machine_codes {
            machine_code_map.set(&kind, code)?;
        }
        let machine_codes = machine_code_map.flush()?;

        let permission_mode = match permission_mode {
            PermissionModeParams::Unrestricted => PermissionMode::Unrestricted,
            PermissionModeParams::AllowList(deployers) => {
                let mut deployers_map = DeployerMap::empty(store, DEFAULT_HAMT_CONFIG, "deployers");
                for d in deployers {
                    deployers_map.set(&d, ())?;
                }
                PermissionMode::AllowList(deployers_map.flush()?)
            }
        };

        let owners = OwnerMap::empty(store, DEFAULT_HAMT_CONFIG, "owners").flush()?;

        Ok(State { machine_codes, permission_mode, owners })
    }

    pub fn get_machine_code<BS: Blockstore>(
        &self,
        store: &BS,
        kind: &Kind,
    ) -> Result<Option<Cid>, ActorError> {
        let machine_code_map =
            MachineCodeMap::load(store, &self.machine_codes, DEFAULT_HAMT_CONFIG, "machines")?;
        let code = machine_code_map.get(kind).map(|c| c.cloned())?;
        Ok(code)
    }

    pub fn set_deployers<BS: Blockstore>(
        &mut self,
        store: &BS,
        deployers: Vec<Address>,
    ) -> anyhow::Result<()> {
        match self.permission_mode {
            PermissionMode::Unrestricted => {
                return Err(anyhow::anyhow!(
                    "cannot set deployers in unrestricted permission mode"
                ));
            }
            PermissionMode::AllowList(_) => {
                let mut deployers_map = DeployerMap::empty(store, DEFAULT_HAMT_CONFIG, "deployers");
                for d in deployers {
                    deployers_map.set(&d, ())?;
                }
                self.permission_mode = PermissionMode::AllowList(deployers_map.flush()?);
            }
        }
        Ok(())
    }

    pub fn can_deploy(&self, rt: &impl Runtime, deployer: ActorID) -> Result<bool, ActorError> {
        Ok(match &self.permission_mode {
            PermissionMode::Unrestricted => true,
            PermissionMode::AllowList(cid) => {
                let deployer_map =
                    DeployerMap::load(rt.store(), cid, DEFAULT_HAMT_CONFIG, "deployers")?;
                let mut allowed = false;
                deployer_map.for_each(|k, _| {
                    // Normalize allowed addresses to ID addresses, so we can compare any kind of allowlisted address.
                    // This includes f1, f2, f3, etc.
                    // We cannot normalize the allowlist at construction time because the addresses may not be bound to IDs yet (counterfactual usage).
                    // Unfortunately, API of Hamt::for_each won't let us stop iterating on match, so this is more wasteful than we'd like. We can optimize later.
                    // Hamt has implemented Iterator recently, but it's not exposed through Map2 (see ENG-800).
                    allowed = allowed || rt.resolve_address(&k) == Some(deployer);
                    Ok(())
                })?;
                allowed
            }
        })
    }

    pub fn set_metadata<BS: Blockstore>(
        &mut self,
        store: &BS,
        owner: Address,
        address: Address,
        kind: Kind,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<()> {
        let mut owner_map = OwnerMap::load(store, &self.owners, DEFAULT_HAMT_CONFIG, "owners")?;
        let mut machine_metadata =
            owner_map.get(&owner)?.map(|machines| machines.to_owned()).unwrap_or_default();
        machine_metadata.push(Metadata { kind, address, metadata });
        owner_map.set(&owner, machine_metadata)?;
        self.owners = owner_map.flush()?;
        Ok(())
    }

    pub fn get_metadata<BS: Blockstore>(
        &self,
        store: &BS,
        owner: Address,
    ) -> anyhow::Result<Vec<Metadata>> {
        let owner_map = OwnerMap::load(store, &self.owners, DEFAULT_HAMT_CONFIG, "owners")?;
        let metadata = owner_map.get(&owner)?.map(|m| m.to_owned()).unwrap_or_default();
        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use cid::Cid;

    use crate::state::PermissionMode;

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
