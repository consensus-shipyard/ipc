// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fendermint_actor_machine::{ConstructorParams, MachineActor};
use fil_actors_runtime::{
    actor_dispatch, actor_error, extract_send_result,
    runtime::{ActorCode, Runtime},
    ActorDowncast, ActorError, FIRST_EXPORTED_METHOD_NUMBER, INIT_ACTOR_ADDR, SYSTEM_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_ipld_hamt::BytesKey;
use fvm_shared::{error::ExitCode, MethodNum};

use crate::{
    ext, AddParams, DeleteParams, GetParams, ListParams, Method, Object, ObjectList, ResolveParams,
    State, OBJECTSTORE_ACTOR_NAME,
};

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
        )
        .map_err(|e| {
            e.downcast_default(
                ExitCode::USR_ILLEGAL_STATE,
                "failed to construct empty store",
            )
        })?;
        rt.create(&state)
    }

    // TODO: if overwriting, delete the old blob from blobs actor
    fn add_object(rt: &impl Runtime, params: AddParams) -> Result<Cid, ActorError> {
        Self::ensure_write_allowed(rt)?;

        let blob_params = ext::blobs::AddBlobParams {
            cid: params.cid,
            size: params.size as u64,
            expiry: rt.curr_epoch() + 100,
            source: Some(params.store),
        };

        // TODO: use read-only flag
        extract_send_result(rt.send_simple(
            &ext::blobs::BLOBS_ACTOR_ADDR,
            ext::blobs::ADD_BLOB_METHOD,
            IpldBlock::serialize_cbor(&blob_params)?,
            rt.message().value_received(),
        ))?;

        let root = rt.transaction(|st: &mut State, rt| {
            st.add(
                rt.store(),
                BytesKey(params.key),
                params.cid,
                params.size,
                params.metadata,
                params.overwrite,
            )
            .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to add object"))
        })?;
        Ok(root)
    }

    // TODO: remove this
    fn resolve_object(rt: &impl Runtime, params: ResolveParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        rt.transaction(|st: &mut State, rt| {
            st.resolve(rt.store(), BytesKey(params.key), params.value)
                .map_err(|e| {
                    e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to resolve object")
                })
        })?;
        Ok(())
    }

    // Deleting an object removes the key from the store, but not from the underlying storage.
    // So, we can't just delete it here via syscall.
    // Once implemented, the DA mechanism may cause the data to be entangled with other data.
    // The retention policies will handle deleting / GC.
    // TODO: call blobs actor to delete
    fn delete_object(rt: &impl Runtime, params: DeleteParams) -> Result<Cid, ActorError> {
        Self::ensure_write_allowed(rt)?;

        let res = rt.transaction(|st: &mut State, rt| {
            st.delete(rt.store(), &BytesKey(params.key)).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to delete object")
            })
        })?;
        Ok(res.1)
    }

    // TODO: fetch blob from blobs actor
    fn get_object(rt: &impl Runtime, params: GetParams) -> Result<Option<Object>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let st: State = rt.state()?;
        st.get(rt.store(), &BytesKey(params.key))
            .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to get object"))
    }

    // TODO: fetch size from blobs actor?
    fn list_objects(rt: &impl Runtime, params: ListParams) -> Result<ObjectList, ActorError> {
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
        Ok(objects)
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
        OBJECTSTORE_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,
        GetMetadata => get_metadata,
        AddObject => add_object,
        ResolveObject => resolve_object,
        DeleteObject => delete_object,
        GetObject => get_object,
        ListObjects => list_objects,
        _ => fallback,
    }
}
