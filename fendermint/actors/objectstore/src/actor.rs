// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fendermint_actor_machine::{ensure_write_allowed, ConstructorParams};
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorDowncast, ActorError, FIRST_EXPORTED_METHOD_NUMBER, INIT_ACTOR_ADDR, SYSTEM_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_ipld_hamt::BytesKey;
use fvm_shared::{error::ExitCode, MethodNum};

use crate::{
    Method, Object, ObjectDeleteParams, ObjectGetParams, ObjectList, ObjectListParams,
    ObjectPutParams, ObjectResolveExternalParams, State, OBJECTSTORE_ACTOR_NAME,
};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

pub struct Actor;

impl Actor {
    fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&INIT_ACTOR_ADDR))?;

        let state = State::new(rt.store(), params.creator, params.write_access).map_err(|e| {
            e.downcast_default(
                ExitCode::USR_ILLEGAL_STATE,
                "failed to construct empty store",
            )
        })?;
        rt.create(&state)
    }

    fn put_object(rt: &impl Runtime, params: ObjectPutParams) -> Result<Cid, ActorError> {
        ensure_write_allowed::<State>(rt)?;

        let root = rt.transaction(|st: &mut State, rt| {
            st.put(rt.store(), BytesKey(params.key), params.kind, true)
                .map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to put object")
                })
        })?;
        Ok(root)
    }

    fn resolve_external_object(
        rt: &impl Runtime,
        params: ObjectResolveExternalParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut State, rt| {
            st.resolve_external(rt.store(), BytesKey(params.key), params.value)
                .map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to resolve object")
                })
        })?;
        Ok(())
    }

    fn delete_object(rt: &impl Runtime, params: ObjectDeleteParams) -> Result<Cid, ActorError> {
        ensure_write_allowed::<State>(rt)?;

        let res = rt.transaction(|st: &mut State, rt| {
            st.delete(rt.store(), &BytesKey(params.key)).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to delete object")
            })
        })?;

        // Clean up external object storage if it existed.
        if let Some(Object::External((v, _))) = res.0 {
            objectstore_actor_sdk::cid_rm(v.0).map_err(|en| {
                ActorError::checked(
                    ExitCode::USR_ILLEGAL_STATE,
                    format!("cid_rm syscall failed with {en}"),
                    None,
                )
            })?;
        }
        Ok(res.1)
    }

    fn get_object(
        rt: &impl Runtime,
        params: ObjectGetParams,
    ) -> Result<Option<Object>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let st: State = rt.state()?;
        st.get(rt.store(), &BytesKey(params.key))
            .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to get object"))
    }

    fn list_objects(
        rt: &impl Runtime,
        params: ObjectListParams,
    ) -> Result<Option<ObjectList>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let st: State = rt.state()?;
        let objects = st
            .list(
                rt.store(),
                params.prefix,
                params.delimiter,
                params.offset,
                params.limit,
            )
            .map_err(|e| {
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
        ResolveExternalObject => resolve_external_object,
        DeleteObject => delete_object,
        GetObject => get_object,
        ListObjects => list_objects,
        _ => fallback,
    }
}
