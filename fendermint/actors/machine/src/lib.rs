// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::anyhow;
pub use fil_actor_adm::Kind;
use fil_actors_runtime::{runtime::Runtime, ActorError};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, MethodNum};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

/// Params for creating an object store machine.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    /// The machine creator robust address.
    pub creator: Address,
    /// Write access dictates who can write to the machine.
    pub write_access: WriteAccess,
}

/// The different types of machine write access.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum WriteAccess {
    /// Only the owner can write to the machine.
    OnlyOwner,
    /// Any valid account can write to the machine.
    Public,
}

impl FromStr for WriteAccess {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "onlyowner" => Self::OnlyOwner,
            "public" => Self::Public,
            _ => return Err(anyhow!("invalid write access")),
        })
    }
}

impl Display for WriteAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::OnlyOwner => "onlyowner",
            Self::Public => "public",
        };
        write!(f, "{}", str)
    }
}

/// Method number that machines must use for get metadata.
pub const GET_METADATA_METHOD: MethodNum = frc42_dispatch::method_hash!("GetMetadata");

// TODO: Add method for changing owner from ADM actor.
pub trait MachineActor {
    type State: MachineState + DeserializeOwned;

    /// Ensures that immediate caller is allowed to write to the machine.
    fn ensure_write_allowed(rt: &impl Runtime) -> Result<(), ActorError> {
        let state = rt.state::<Self::State>()?;
        match state.write_access() {
            WriteAccess::OnlyOwner => {
                // Leaving this note here as something to revist in the future before mainnet.
                //
                // We want owner to be stored as a robust address that users can understand,
                // but the caller is always an ID address. This means we have to resolve the
                // actor ID from the init actor, which adds some extra ops and charges gas.
                // We could instead store both actor ID and robust address in machine state,
                // but I _think_ that could result in incorrect robust address to actor ID
                // pairings in the case of a reorg.
                if let Some(owner_id) = rt.resolve_address(&state.owner()) {
                    rt.validate_immediate_caller_is(std::iter::once(&Address::new_id(owner_id)))?
                } else {
                    // This should not happen.
                    return Err(ActorError::forbidden(String::from(
                        "failed to resolve machine owner id",
                    )));
                }
            }
            WriteAccess::Public => rt.validate_immediate_caller_accept_any()?,
        }
        Ok(())
    }

    /// Get machine metadata.
    fn get_metadata(rt: &impl Runtime) -> Result<Metadata, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st = rt.state::<Self::State>()?;
        Ok(Metadata {
            owner: st.owner(),
            kind: st.kind(),
        })
    }
}

/// Machine metadata.
#[derive(Debug, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Metadata {
    /// Machine kind.
    pub kind: Kind,
    /// Machine owner robust address.
    pub owner: Address,
}

/// Trait that must be implemented by machine state.
pub trait MachineState {
    fn kind(&self) -> Kind;
    fn owner(&self) -> Address;
    fn write_access(&self) -> WriteAccess;
}
