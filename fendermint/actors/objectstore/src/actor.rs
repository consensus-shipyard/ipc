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

        let add_params = IpldBlock::serialize_cbor(&ext::blobs::AddBlobParams {
            cid: params.cid,
            size: params.size as u64,
            expiry: rt.curr_epoch() + 100,
            source: Some(params.store),
        })?;
        extract_send_result(rt.send_simple(
            &ext::blobs::BLOBS_ACTOR_ADDR,
            ext::blobs::ADD_BLOB_METHOD,
            add_params,
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
            let cid = Cid::try_from(params.key.clone()).map_err(|e| {
                ActorError::unchecked(ExitCode::USR_ILLEGAL_ARGUMENT, e.to_string())
            })?;
            extract_send_result(rt.send_simple(
                &ext::blobs::BLOBS_ACTOR_ADDR,
                ext::blobs::DELETE_BLOB_METHOD,
                IpldBlock::serialize_cbor(&ext::blobs::DeleteBlobParams(cid))?,
                Default::default(),
            ))?;
            st.delete(rt.store(), &BytesKey(params.key)).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to delete object")
            })
        })?;
        Ok(res.1)
    }

    // TODO: fetch blob from blobs actor
    // TODO SU: How is Blob included in Object?? Shall we return Blob??
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

#[cfg(test)]
mod tests {
    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ETHACCOUNT_ACTOR_CODE_ID, SYSTEM_ACTOR_CODE_ID,
    };
    use fil_actors_runtime::INIT_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::econ::TokenAmount;

    use crate::{actor::Actor, ext, AddParams, Method, State};
    use cid::{multihash, Cid};
    use fendermint_actor_machine::WriteAccess;
    use fil_actors_runtime::runtime::Runtime;
    use fvm_shared::error::ExitCode;
    use rand::Rng;
    use std::collections::HashMap;

    fn construct_and_verify(creator: Address) -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, INIT_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![INIT_ACTOR_ADDR]);
        let write_access: WriteAccess = WriteAccess::Public;
        let metadata = HashMap::new();
        let actor_construction = rt
            .call::<Actor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&fendermint_actor_machine::ConstructorParams {
                    creator,
                    write_access,
                    metadata,
                })
                .unwrap(),
            )
            .unwrap();
        expect_empty(actor_construction);
        rt.verify();
        rt.reset();
        rt
    }

    fn new_cid() -> Cid {
        let mut rng = rand::thread_rng();
        let mut hash = [0u8; 32];
        rng.fill(&mut hash);
        Cid::new_v1(
            0x55,
            multihash::Multihash::wrap(multihash::Code::Blake2b256.into(), &hash).unwrap(),
        )
    }

    #[test]
    pub fn test_add_object() {
        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        let rt = construct_and_verify(f4_eth_addr);
        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);
        rt.expect_validate_caller_any();
        let add_params: AddParams = AddParams {
            store: f4_eth_addr,
            key: vec![0, 1, 2],
            cid: new_cid(),
            size: 20,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_send_simple(
            ext::blobs::BLOBS_ACTOR_ADDR,
            ext::blobs::ADD_BLOB_METHOD,
            IpldBlock::serialize_cbor(&ext::blobs::AddBlobParams {
                cid: add_params.cid,
                size: add_params.size as u64,
                expiry: 100,
                source: Some(add_params.store),
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            None,
            ExitCode::OK,
        );
        let result = rt
            .call::<Actor>(
                Method::AddObject as u64,
                IpldBlock::serialize_cbor(&add_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Cid>()
            .unwrap();
        let state = rt.state::<State>().unwrap();
        assert_eq!(state.root, result);
    }
}
