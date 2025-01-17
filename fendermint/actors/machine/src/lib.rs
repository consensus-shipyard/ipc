// Copyright 2024 Hoku Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub use fil_actor_adm::Kind;
use fil_actors_runtime::{
    actor_error, runtime::builtins::Type, runtime::Runtime, ActorError, ADM_ACTOR_ADDR,
    FIRST_EXPORTED_METHOD_NUMBER, INIT_ACTOR_ADDR,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{ipld_block::IpldBlock, tuple::*};
pub use fvm_shared::METHOD_CONSTRUCTOR;
use fvm_shared::{address::Address, MethodNum};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

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

/// Returns an error if the address does not match the message origin or caller.
pub fn require_addr_is_origin_or_caller(
    rt: &impl Runtime,
    address: Address,
) -> Result<(), ActorError> {
    let address = to_id_address(rt, address, false)?;
    if address == rt.message().origin() || address == rt.message().caller() {
        return Ok(());
    }
    Err(ActorError::illegal_argument(format!(
        "address {} does not match origin or caller",
        address
    )))
}

/// Resolves an Account ID Address to its external delegated Address
pub fn resolve_delegated_address(
    rt: &impl Runtime,
    account_address: Address,
) -> Result<Address, ActorError> {
    let account_id = rt
        .resolve_address(&account_address)
        .ok_or(ActorError::not_found(format!(
            "actor {} not found",
            account_address
        )))?;

    rt.lookup_delegated_address(account_id)
        .ok_or(ActorError::not_found(format!(
            "invalid address: actor {} is not delegated",
            account_address
        )))
}

/// Resolves ID address of an actor.
/// If `require_delegated` is `true`, the address must be of type
/// EVM (a Solidity contract), EthAccount (an Ethereum-style EOA), or Placeholder (a yet to be
/// determined EOA or Solidity contract).
pub fn to_id_address(
    rt: &impl Runtime,
    address: Address,
    require_delegated: bool,
) -> Result<Address, ActorError> {
    let actor_id = rt
        .resolve_address(&address)
        .ok_or(ActorError::not_found(format!(
            "actor {} not found",
            address
        )))?;
    if require_delegated {
        let code_cid = rt
            .get_actor_code_cid(&actor_id)
            .expect("failed to lookup actor code cid");
        if !matches!(
            rt.resolve_builtin_actor_type(&code_cid),
            Some(Type::Placeholder | Type::EVM | Type::EthAccount)
        ) {
            return Err(ActorError::forbidden(format!(
                "address {} is not delegated",
                address,
            )));
        }
    }
    Ok(Address::new_id(actor_id))
}

// TODO: Add method for changing owner from ADM actor.
pub trait MachineActor {
    type State: MachineState + Serialize + DeserializeOwned;

    /// Machine actor constructor.
    fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&INIT_ACTOR_ADDR))?;
        params.owner.id().map_err(|_| {
            ActorError::illegal_argument("machine owner address must be an ID address".into())
        })?;
        let state = Self::State::new(rt.store(), params.owner, params.metadata)?;
        rt.create(&state)
    }

    /// Initializes the machine with its ID address.
    fn init(rt: &impl Runtime, params: InitParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&ADM_ACTOR_ADDR))?;
        params.address.id().map_err(|_| {
            ActorError::illegal_argument("machine address must be an ID address".into())
        })?;
        rt.transaction(|st: &mut Self::State, _| st.init(params.address))
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
        Ok(Metadata {
            owner: st.owner(),
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
