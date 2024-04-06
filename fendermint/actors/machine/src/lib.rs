// Copyright 2024 Textile

use anyhow::anyhow;
use fil_actors_runtime::{runtime::Runtime, ActorError};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::ActorID;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Params for creating an object store machine.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    /// The machine creator.
    pub creator: ActorID,
    /// Write access dictates who can write to the machine.
    pub write_access: WriteAccess,
}

/// The different types of machine write access.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Trait that must be implemented by machine state.
pub trait MachineState {
    fn owner(&self) -> ActorID;
    fn write_access(&self) -> WriteAccess;
}

/// Ensures that immediate caller is allowed to write to the machine.
pub fn ensure_write_allowed<S>(rt: &impl Runtime) -> Result<(), ActorError>
where
    S: MachineState + DeserializeOwned,
{
    let state = rt.state::<S>()?;
    match state.write_access() {
        WriteAccess::OnlyOwner => {
            rt.validate_immediate_caller_is(std::iter::once(&Address::new_id(state.owner())))?
        }
        WriteAccess::Public => rt.validate_immediate_caller_accept_any()?,
    }
    Ok(())
}
