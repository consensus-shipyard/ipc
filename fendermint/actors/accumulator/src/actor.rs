// Copyright 2024 Textile Inc
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::actor_dispatch;
use fil_actors_runtime::actor_error;
use fil_actors_runtime::builtin::singletons::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::ActorDowncast;
use fil_actors_runtime::ActorError;
use fvm_ipld_hamt::BytesKey;
use fvm_shared::error::ExitCode;

use crate::{Method, State, ACCUMULATOR_ACTOR_NAME};

fil_actors_runtime::wasm_trampoline!(Actor);

pub struct Actor;

impl Actor {
    fn constructor(rt: &impl Runtime) -> Result<(), ActorError> {
        // FIXME:(carsonfarmer) We're setting this up to be a subnet-wide actor for a single repo.
        // FIXME:(carsonfarmer) In the future, this could be deployed dynamically for multi repo subnets.
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let state = State::new(rt.store()).map_err(|e| {
            e.downcast_default(
                ExitCode::USR_ILLEGAL_STATE,
                "failed to construct empty store",
            )
        })?;
        rt.create(&state)
    }

    fn push<S: DeserializeOwned + Serialize>(rt: &impl Runtime, obj: S) -> Result<Cid, ActorError> {
        // FIXME:(carsonfarmer) We'll want to validate the caller is the owner of the repo.
        rt.validate_immediate_caller_accept_any()?;

        rt.transaction(|st: &mut State, rt| {
            st.push(rt.store(), obj).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to put object")
            })
        })
    }

    fn get_root(rt: &impl Runtime) -> Result<Cid, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state().map(|st| {
            st.get_root(rt.store())
                .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to bag peaks"))
        })
    }

    fn get_peaks(rt: &impl Runtime) -> Result<Vec<Cid>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state().map(|st| {
            st.get_peaks(rt.store())
                .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to get peaks"))
        })
    }

    fn get_count(rt: &impl Runtime) -> Result<u64, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state().map(|st| st.leaf_count)
    }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        ACCUMULATOR_ACTOR_NAME_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        Push => push,
        Root => get_root,
        Peaks => get_peaks,
        Count => get_count,
    }
}
