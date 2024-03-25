// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    builtin::singletons::SYSTEM_ACTOR_ADDR,
    runtime::{ActorCode, Runtime},
    ActorDowncast, ActorError, FIRST_EXPORTED_METHOD_NUMBER,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_ipld_hamt::BytesKey;
use fvm_shared::{error::ExitCode, MethodNum};

use crate::{ListOptions, Method, Object, ObjectList, ObjectParams, State, OBJECTSTORE_ACTOR_NAME};

#[cfg(feature = "fil-actor")]
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

        let root = rt.transaction(|st: &mut State, rt| {
            st.put(rt.store(), BytesKey(params.key), params.value, true)
                .map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to put object")
                })
        })?;

        Ok(root)
    }

    fn resolve_object(rt: &impl Runtime, params: ObjectParams) -> Result<Cid, ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let root = rt.transaction(|st: &mut State, rt| {
            st.resolve(rt.store(), BytesKey(params.key), params.value)
                .map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to resolve object")
                })
        })?;

        Ok(root)
    }

    fn delete_object(rt: &impl Runtime, key: Vec<u8>) -> Result<Cid, ActorError> {
        // FIXME:(carsonfarmer) We'll want to validate the caller is the owner of the repo.
        rt.validate_immediate_caller_accept_any()?;

        let res = rt.transaction(|st: &mut State, rt| {
            st.delete(rt.store(), &BytesKey(key)).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to delete object")
            })
        })?;

        // Clean up external storage of this key it existed.
        if let Some(o) = res.0 {
            objectstore_actor_sdk::cid_rm(o.value).map_err(|en| {
                ActorError::checked(
                    ExitCode::USR_ILLEGAL_STATE,
                    format!("cid_rm syscall failed with {en}"),
                    None,
                )
            })?;
        }

        Ok(res.1)
    }

    fn get_object(rt: &impl Runtime, key: Vec<u8>) -> Result<Option<Object>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let st: State = rt.state()?;
        st.get(rt.store(), &BytesKey(key))
            .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to get object"))
    }

    fn list_objects(
        rt: &impl Runtime,
        params: ListOptions,
    ) -> Result<Option<ObjectList>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let prefix = if params.prefix.is_empty() {
            None
        } else {
            Some(BytesKey(params.prefix))
        };
        let delimiter = if params.delimiter.is_empty() {
            None
        } else {
            Some(BytesKey(params.delimiter))
        };
        let limit = if params.limit == 0 {
            None
        } else {
            Some(params.limit)
        };
        let st: State = rt.state()?;
        let objects = st.list(rt.store(), prefix, delimiter, limit).map_err(|e| {
            e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to list objects")
        })?;
        Ok(Some(objects))
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
        _ => fallback,
    }
}
