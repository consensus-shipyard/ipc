// Copyright 2024 Textile Inc
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::actor_dispatch;
use fil_actors_runtime::actor_error;
use fil_actors_runtime::builtin::singletons::SYSTEM_ACTOR_ADDR;
use fil_actors_runtime::runtime::{ActorCode, Runtime};
use fil_actors_runtime::ActorDowncast;
use fil_actors_runtime::ActorError;
use fvm_ipld_hamt::BytesKey;
use fvm_shared::error::ExitCode;

use crate::{Method, State, CONTENTRESOLVER_ACTOR_NAME};

fil_actors_runtime::wasm_trampoline!(Actor);

pub struct Actor;

impl Actor {
    fn constructor(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let state = State::new(rt.store()).map_err(|e| {
            e.downcast_default(
                ExitCode::USR_ILLEGAL_STATE,
                "failed to construct empty store",
            )
        })?;

        rt.create(&state)
    }

    fn push_cid(rt: &impl Runtime, cid: Vec<u8>) -> Result<Cid, ActorError> {
        // FIXME:(sander) We'll want to validate the caller is the system actor.
        rt.validate_immediate_caller_accept_any()?;

        let root = rt.transaction(|st: &mut State, rt| {
            st.push(rt.store(), BytesKey(cid))
                .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to push cid"))
        })?;

        Ok(root)
    }

    fn delete_cid(rt: &impl Runtime, cid: Vec<u8>) -> Result<Cid, ActorError> {
        // FIXME:(sander) We'll want to validate the caller is the system actor
        rt.validate_immediate_caller_accept_any()?;

        let root = rt.transaction(|st: &mut State, rt| {
            st.delete(rt.store(), &BytesKey(cid)).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to delete cid")
            })
        })?;

        Ok(root)
    }

    fn list_cids(rt: &impl Runtime) -> Result<Option<Vec<Vec<u8>>>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let st: State = rt.state()?;
        let cids = st
            .list(rt.store())
            .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to list cids"))?;
        Ok(Some(cids))
    }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        CONTENTRESOLVER_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        PushCid => push_cid,
        DeleteCid => delete_cid,
        ListCids => list_cids,
    }
}
