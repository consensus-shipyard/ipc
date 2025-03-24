// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

pub use fil_actor_adm::Kind;
use fil_actors_runtime::{
    actor_error, runtime::Runtime, ActorError, ADM_ACTOR_ADDR, FIRST_EXPORTED_METHOD_NUMBER,
    INIT_ACTOR_ADDR,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{ipld_block::IpldBlock, tuple::*};
pub use fvm_shared::METHOD_CONSTRUCTOR;
use fvm_shared::{address::Address, MethodNum};
use recall_actor_sdk::{
    emit_evm_event, to_delegated_address, to_id_address, to_id_and_delegated_address,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::sol_facade::{MachineCreated, MachineInitialized};

pub mod sol_facade;

/// Params for creating a machine.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    /// The machine owner ID address.
    pub owner: Address,
    /// User-defined metadata.
    pub metadata: HashMap<String, String>,
}

/// Params for initializing a machine.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct InitParams {
    /// The machine ID address.
    pub address: Address,
}

/// Machine initialization method number.
pub const INIT_METHOD: MethodNum = 2;
/// Get machine address method number.
pub const GET_ADDRESS_METHOD: MethodNum = frc42_dispatch::method_hash!("GetAddress");
/// Get machine metadata method number.
pub const GET_METADATA_METHOD: MethodNum = frc42_dispatch::method_hash!("GetMetadata");

// TODO: Add method for changing owner from ADM actor.
pub trait MachineActor {
    type State: MachineState + Serialize + DeserializeOwned;

    /// Machine actor constructor.
    fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&INIT_ACTOR_ADDR))?;

        let (id_addr, delegated_addr) = to_id_and_delegated_address(rt, params.owner)?;

        let state = Self::State::new(rt.store(), id_addr, params.metadata)?;
        rt.create(&state)?;

        emit_evm_event(
            rt,
            MachineCreated::new(state.kind(), delegated_addr, &state.metadata()),
        )
    }

    /// Initializes the machine with its ID address.
    fn init(rt: &impl Runtime, params: InitParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&ADM_ACTOR_ADDR))?;

        let id_addr = to_id_address(rt, params.address, false)?;

        let kind = rt.transaction(|st: &mut Self::State, _| {
            st.init(id_addr)?;
            Ok(st.kind())
        })?;

        emit_evm_event(rt, MachineInitialized::new(kind, id_addr))
    }

    /// Get machine robust address.
    fn get_address(rt: &impl Runtime) -> Result<Address, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st = rt.state::<Self::State>()?;
        st.address().get()
    }

    /// Get machine metadata.
    fn get_metadata(rt: &impl Runtime) -> Result<Metadata, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st = rt.state::<Self::State>()?;
        let owner = st.owner();
        let address = to_delegated_address(rt, owner).unwrap_or(owner);
        Ok(Metadata {
            owner: address,
            kind: st.kind(),
            metadata: st.metadata(),
        })
    }

    fn fallback(
        rt: &impl Runtime,
        method: MethodNum,
        _: Option<IpldBlock>,
    ) -> Result<Option<IpldBlock>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        if method >= FIRST_EXPORTED_METHOD_NUMBER {
            Ok(None)
        } else {
            Err(actor_error!(unhandled_message; "invalid method: {}", method))
        }
    }
}

/// Machine metadata.
#[derive(Debug, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Metadata {
    /// Machine kind.
    pub kind: Kind,
    /// Machine owner ID address.
    pub owner: Address,
    /// User-defined data.
    pub metadata: HashMap<String, String>,
}

/// Trait that must be implemented by machine state.
pub trait MachineState {
    fn new<BS: Blockstore>(
        store: &BS,
        owner: Address,
        metadata: HashMap<String, String>,
    ) -> Result<Self, ActorError>
    where
        Self: Sized;
    fn init(&mut self, address: Address) -> Result<(), ActorError>;
    fn address(&self) -> MachineAddress;
    fn kind(&self) -> Kind;
    fn owner(&self) -> Address;
    fn metadata(&self) -> HashMap<String, String>;
}

/// Machine address wrapper.
#[derive(Debug, Clone, Default, Serialize_tuple, Deserialize_tuple)]
pub struct MachineAddress {
    address: Option<Address>,
}

impl MachineAddress {
    /// Get machine address.
    pub fn get(&self) -> Result<Address, ActorError> {
        self.address.ok_or(ActorError::illegal_state(String::from(
            "machine address not set",
        )))
    }

    /// Set machine address. This can only be called once.
    pub fn set(&mut self, address: Address) -> Result<(), ActorError> {
        if self.address.is_some() {
            return Err(ActorError::forbidden(String::from(
                "machine address already set",
            )));
        }
        self.address = Some(address);
        Ok(())
    }
}
