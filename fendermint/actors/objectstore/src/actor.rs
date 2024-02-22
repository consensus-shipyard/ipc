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

use crate::{Method, Object, ObjectParams, State, OBJECTSTORE_ACTOR_NAME};

// const SYSCALL_FAILED_EXIT_CODE: u32 = 0x31337;

fil_actors_runtime::wasm_trampoline!(Actor);

pub struct Actor;

impl Actor {
    fn constructor(rt: &impl Runtime) -> Result<(), ActorError> {
        // FIXME:(sander) We're setting this up to be a subnet-wide actor for a single repo.
        // FIXME:(sander) In the future, this could be deployed dynamically for multi repo subnets.
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let state = State::new(rt.store()).map_err(|e| {
            e.downcast_default(
                ExitCode::USR_ILLEGAL_STATE,
                "failed to construct empty store",
            )
        })?;

        rt.create(&state)
    }

    fn put_object(rt: &impl Runtime, params: ObjectParams) -> Result<Cid, ActorError> {
        // FIXME:(carsonfarmer) We'll want to validate the caller is the owner of the repo.
        rt.validate_immediate_caller_accept_any()?;

        // objectstore_actor_sdk::load_car(params.file).map_err(|en| {
        //     let msg = format!("load_car syscall failed with {en}");
        //     ActorError::checked(ExitCode::new(SYSCALL_FAILED_EXIT_CODE), msg, None)
        // })?;

        let root = rt.transaction(|st: &mut State, rt| {
            st.put(rt.store(), BytesKey(params.key), params.value, true)
                .map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to put object")
                })
        })?;

        Ok(root)
    }

    fn resolve_object(rt: &impl Runtime, key: Vec<u8>) -> Result<Cid, ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let root = rt.transaction(|st: &mut State, rt| {
            st.resolve(rt.store(), &BytesKey(key)).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to resolve object")
            })
        })?;

        Ok(root)
    }

    fn delete_object(rt: &impl Runtime, key: Vec<u8>) -> Result<Cid, ActorError> {
        // FIXME:(carsonfarmer) We'll want to validate the caller is the owner of the repo.
        rt.validate_immediate_caller_accept_any()?;

        let root = rt.transaction(|st: &mut State, rt| {
            st.delete(rt.store(), &BytesKey(key)).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to delete object")
            })
        })?;

        Ok(root)
    }

    fn get_object(rt: &impl Runtime, key: Vec<u8>) -> Result<Option<Object>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let st: State = rt.state()?;
        st.get(rt.store(), &BytesKey(key))
            .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to get object"))
    }

    fn list_objects(rt: &impl Runtime) -> Result<Option<Vec<(Vec<u8>, Object)>>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let st: State = rt.state()?;
        let objects = st.list(rt.store()).map_err(|e| {
            e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to list objects")
        })?;
        Ok(Some(objects))
    }
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        OBJECTSTORE_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        PutObject => put_object,
        ResolveObject => resolve_object,
        DeleteObject => delete_object,
        GetObject => get_object,
        ListObjects => list_objects,
    }
}
