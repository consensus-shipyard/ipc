// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! # ADM (Autonomous Data Management) Actor
//!
//! The ADM actor is a factory for creating and managing "machine" instances.
//! Machines are autonomous on-chain entities with specific purposes:
//! - Bucket: S3-like object storage with key-value semantics
//! - Timehub: MMR-based timestamping service
//!
//! The ADM actor:
//! - Creates machine instances via the Init actor
//! - Tracks ownership and metadata
//! - Manages deployer permissions
//! - Provides listing and discovery

use std::collections::HashMap;

use fil_actors_runtime::{
    actor_dispatch, actor_error, deserialize_block, extract_send_result,
    runtime::{ActorCode, Runtime},
    ActorError, INIT_ACTOR_ADDR,
};
use fvm_ipld_encoding::{ipld_block::IpldBlock, tuple::*, RawBytes};
use fvm_shared::{
    address::Address, econ::TokenAmount, ActorID, METHOD_CONSTRUCTOR,
};
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

pub mod sol_facade;
mod state;

pub use state::State;

// Init actor Exec4 method number (standard across Filecoin)
const EXEC4_METHOD: u64 = 3;

/// Parameters for Init.Exec4
#[derive(Serialize_tuple, Deserialize_tuple)]
struct Exec4Params {
    code_cid: cid::Cid,
    constructor_params: RawBytes,
    subaddress: RawBytes,
}

/// Return value from Init.Exec4
#[derive(Serialize_tuple, Deserialize_tuple)]
struct Exec4Return {
    id_address: ActorID,
    robust_address: Address,
}

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

/// ADM actor implementation
pub struct Actor;

impl Actor {
    /// Constructor - called when actor is created
    pub fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&INIT_ACTOR_ADDR))?;

        let state = State::new(rt.store(), params.deployers)?;
        rt.create(&state)?;

        Ok(())
    }

    /// Create a new machine (bucket, timehub, etc.) owned by an address
    pub fn create_external(
        rt: &impl Runtime,
        params: CreateExternalParams,
    ) -> Result<CreateExternalReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        // Get the code CID for the machine type from state
        let code_cid = rt.state::<State>()?.get_code_cid(&params.kind)?;

        // Prepare constructor params for the machine actor
        let constructor_params = RawBytes::serialize(fendermint_actor_machine::ConstructorParams {
            owner: params.owner,
            metadata: params.metadata.clone(),
        })?;

        // Create Exec4 params
        let exec_params = Exec4Params {
            code_cid,
            constructor_params,
            subaddress: RawBytes::default(), // Empty subaddress for machines
        };

        // Call Init.Exec4 to create the machine actor
        let exec_return: Exec4Return = deserialize_block(extract_send_result(rt.send(
            &INIT_ACTOR_ADDR,
            EXEC4_METHOD,
            IpldBlock::serialize_cbor(&exec_params)?,
            TokenAmount::from_atto(0),
            None, // gas limit
            fvm_shared::sys::SendFlags::empty(),
        ))?)?;

        // Call machine.Init to initialize it with its ID address
        let init_params = fendermint_actor_machine::InitParams {
            address: Address::new_id(exec_return.id_address),
        };

        rt.send(
            &Address::new_id(exec_return.id_address),
            fendermint_actor_machine::INIT_METHOD,
            IpldBlock::serialize_cbor(&init_params)?,
            TokenAmount::from_atto(0),
            None, // gas limit
            fvm_shared::sys::SendFlags::empty(),
        )?;

        // Track the machine in ADM state
        rt.transaction(|st: &mut State, _store| {
            st.add_machine(
                exec_return.id_address,
                params.owner,
                params.kind,
                &exec_return.robust_address,
            )
        })?;

        Ok(CreateExternalReturn {
            actor_id: exec_return.id_address,
            robust_address: Some(exec_return.robust_address),
        })
    }

    /// Update the list of addresses allowed to deploy machines
    pub fn update_deployers(
        rt: &impl Runtime,
        params: UpdateDeployersParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, _store| {
            st.update_deployers(params.deployers)
        })?;

        Ok(())
    }

    /// List all machines owned by an address
    pub fn list_metadata(
        rt: &impl Runtime,
        params: ListMetadataParams,
    ) -> Result<Vec<Metadata>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let state = rt.state::<State>()?;
        state.list_machines_by_owner(rt.store(), &params.owner)
    }

    /// Register code CID for a machine type (admin only)
    pub fn register_code_cid(
        rt: &impl Runtime,
        params: RegisterCodeCidParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, _store| {
            st.set_code_cid(&params.kind, params.code_cid);
            Ok(())
        })?;

        Ok(())
    }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        "ADM"
    }

    actor_dispatch! {
        Constructor => constructor,
        CreateExternal => create_external,
        UpdateDeployers => update_deployers,
        ListMetadata => list_metadata,
        RegisterCodeCid => register_code_cid,
    }
}

/// ADM actor methods
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    CreateExternal = frc42_dispatch::method_hash!("CreateExternal"),
    UpdateDeployers = frc42_dispatch::method_hash!("UpdateDeployers"),
    ListMetadata = frc42_dispatch::method_hash!("ListMetadata"),
    RegisterCodeCid = frc42_dispatch::method_hash!("RegisterCodeCid"),
}

/// Constructor parameters
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    /// Initial list of deployer addresses
    pub deployers: Vec<Address>,
}

/// Parameters for creating a machine
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct CreateExternalParams {
    /// Owner of the machine
    pub owner: Address,
    /// Type of machine to create
    pub kind: Kind,
    /// User-defined metadata
    pub metadata: HashMap<String, String>,
}

/// Return value from creating a machine
#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct CreateExternalReturn {
    /// Actor ID of the created machine
    pub actor_id: ActorID,
    /// Robust (delegated) address if available
    pub robust_address: Option<Address>,
}

/// Parameters for updating deployers
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct UpdateDeployersParams {
    /// New list of deployer addresses
    pub deployers: Vec<Address>,
}

/// Parameters for listing machines
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ListMetadataParams {
    /// Owner address to list machines for
    pub owner: Address,
}

/// Parameters for registering code CID
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct RegisterCodeCidParams {
    /// Machine type to register
    pub kind: Kind,
    /// Code CID for the machine actor
    pub code_cid: cid::Cid,
}

/// Machine metadata returned by list_metadata
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Metadata {
    /// Machine type
    pub kind: Kind,
    /// Machine address
    pub address: Address,
    /// User-defined metadata
    pub metadata: HashMap<String, String>,
}

/// Types of machines that can be created
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    /// S3-like object storage with key-value semantics
    Bucket,
    /// MMR accumulator for timestamping
    Timehub,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Bucket => write!(f, "bucket"),
            Kind::Timehub => write!(f, "timehub"),
        }
    }
}

