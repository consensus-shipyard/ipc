// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fendermint_actor_machine::{ConstructorParams, MachineActor};
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorError, FIRST_EXPORTED_METHOD_NUMBER, INIT_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::MethodNum;

use crate::{Method, PushParams, PushReturn, State, ACCUMULATOR_ACTOR_NAME};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

pub struct Actor;

impl Actor {
    fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&INIT_ACTOR_ADDR))?;
        let state = State::new(
            rt.store(),
            params.creator,
            params.write_access,
            params.metadata,
        )?;
        rt.create(&state)
    }

    fn push(rt: &impl Runtime, params: PushParams) -> Result<PushReturn, ActorError> {
        Self::ensure_write_allowed(rt)?;
        rt.transaction(|st: &mut State, rt| st.push(rt.store(), params.0))
    }

    fn get_leaf_at(rt: &impl Runtime, index: u64) -> Result<Option<Vec<u8>>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        st.get_leaf_at(rt.store(), index)
    }

    fn get_root(rt: &impl Runtime) -> Result<Cid, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        st.get_root(rt.store())
    }

    fn get_peaks(rt: &impl Runtime) -> Result<Vec<Cid>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        st.get_peaks(rt.store())
    }

    fn get_count(rt: &impl Runtime) -> Result<u64, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        Ok(st.leaf_count)
    }

    /// Fallback method for unimplemented method numbers.
    pub fn fallback(
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

impl MachineActor for Actor {
    type State = State;
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        ACCUMULATOR_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        GetMetadata => get_metadata,
        Push => push,
        Get => get_leaf_at,
        Root => get_root,
        Peaks => get_peaks,
        Count => get_count,
        _ => fallback,
    }
}
