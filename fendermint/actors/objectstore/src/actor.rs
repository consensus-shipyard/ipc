// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{
    AddParams, DeleteParams, GetParams, GotObject, ListParams, Method, Object, ObjectList, State,
    OBJECTSTORE_ACTOR_NAME,
};
use cid::Cid;
use fendermint_actor_blobs_shared::params::{AddBlobParams, DeleteBlobParams, GetBlobParams};
use fendermint_actor_blobs_shared::state::Blob;
use fendermint_actor_blobs_shared::{Method as BlobMethod, BLOBS_ACTOR_ADDR};
use fendermint_actor_machine::{ConstructorParams, MachineActor};
use fil_actors_runtime::{
    actor_dispatch, actor_error, deserialize_block, extract_send_result,
    runtime::{ActorCode, Runtime},
    ActorDowncast, ActorError, FIRST_EXPORTED_METHOD_NUMBER, INIT_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_ipld_hamt::BytesKey;
use fvm_shared::{error::ExitCode, MethodNum};

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

    fn add_object(rt: &impl Runtime, params: AddParams) -> Result<Cid, ActorError> {
        Self::ensure_write_allowed(rt)?;
        let key = BytesKey(params.key);
        if let Some(object) = Self::retrieve_object(rt, &key)? {
            if params.overwrite {
                extract_send_result(rt.send_simple(
                    &BLOBS_ACTOR_ADDR,
                    BlobMethod::DeleteBlob as MethodNum,
                    IpldBlock::serialize_cbor(&DeleteBlobParams(object.hash))?,
                    rt.message().value_received(),
                ))?;
            } else {
                return Err(ActorError::illegal_state(
                    "asked not to overwrite".to_string(),
                ));
            }
        }

        let add_params = IpldBlock::serialize_cbor(&AddBlobParams {
            from: Some(params.to),
            source: params.source,
            hash: params.hash,
            size: params.size as u64,
            expiry: rt.curr_epoch() + 100,
        })?;
        extract_send_result(rt.send_simple(
            &BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            add_params,
            rt.message().value_received(),
        ))?;

        let root = rt.transaction(|st: &mut State, rt| {
            st.add(
                rt.store(),
                key,
                params.hash,
                params.size,
                params.metadata,
                params.overwrite,
            )
            .map_err(|e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to add object"))
        })?;
        Ok(root)
    }

    // Deleting an object removes the key from the store, but not from the underlying storage.
    // So, we can't just delete it here via syscall.
    // Once implemented, the DA mechanism may cause the data to be entangled with other data.
    // The retention policies will handle deleting / GC.
    fn delete_object(rt: &impl Runtime, params: DeleteParams) -> Result<Cid, ActorError> {
        Self::ensure_write_allowed(rt)?;
        let key = BytesKey(params.key);
        // 1. Retrieve object CID
        let object = Self::retrieve_object(rt, &key)?.ok_or(ActorError::unchecked(
            ExitCode::USR_ILLEGAL_STATE,
            "no object stored".to_string(),
        ))?;
        // 2. Delete from the blobs actor
        extract_send_result(rt.send_simple(
            &BLOBS_ACTOR_ADDR,
            BlobMethod::DeleteBlob as MethodNum,
            IpldBlock::serialize_cbor(&DeleteBlobParams(object.hash))?,
            rt.message().value_received(),
        ))?;
        // 3. Delete from the state
        let res = rt.transaction(|st: &mut State, rt| {
            st.delete(rt.store(), &key).map_err(|e| {
                e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to delete object")
            })
        })?;
        Ok(res.1)
    }

    fn get_object(rt: &impl Runtime, params: GetParams) -> Result<Option<GotObject>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        if let Some(object) = Self::retrieve_object(rt, &BytesKey(params.key))? {
            let blob = deserialize_block::<Blob>(extract_send_result(rt.send_simple(
                &BLOBS_ACTOR_ADDR,
                BlobMethod::GetBlob as MethodNum,
                IpldBlock::serialize_cbor(&GetBlobParams(object.hash))?,
                rt.message().value_received(),
            ))?)?;
            Ok(Some(GotObject {
                hash: object.hash,
                size: blob.size as usize,
                expiry: blob.expiry,
                metadata: object.metadata,
            }))
        } else {
            Ok(None)
        }
    }

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

    /// Retrieve object from the state.
    fn retrieve_object(rt: &impl Runtime, key: &BytesKey) -> Result<Option<Object>, ActorError> {
        let state = rt.state::<State>()?;
        let store = rt.store();
        state.get(store, key).map_err(|e| {
            e.downcast_default(ExitCode::USR_ILLEGAL_STATE, "failed to retrieve object")
        })
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
        DeleteObject => delete_object,
        GetObject => get_object,
        ListObjects => list_objects,
        _ => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cid::Cid;
    use fendermint_actor_blobs_shared::state::{Hash, PublicKey};
    use fendermint_actor_machine::WriteAccess;
    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::runtime::Runtime;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ETHACCOUNT_ACTOR_CODE_ID, SYSTEM_ACTOR_CODE_ID,
    };
    use fil_actors_runtime::INIT_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::clock::ChainEpoch;
    use fvm_shared::econ::TokenAmount;
    use fvm_shared::error::ExitCode;
    use rand::RngCore;
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
                IpldBlock::serialize_cbor(&ConstructorParams {
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

    pub fn new_hash(size: usize) -> (Hash, u64) {
        let mut rng = rand::thread_rng();
        let mut data = vec![0u8; size];
        rng.fill_bytes(&mut data);
        (
            Hash(*iroh_base::hash::Hash::new(&data).as_bytes()),
            size as u64,
        )
    }

    pub fn new_pk() -> PublicKey {
        let mut rng = rand::thread_rng();
        let mut data = [0u8; 32];
        rng.fill_bytes(&mut data);
        PublicKey(data)
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

        let hash = new_hash(256);
        let add_params: AddParams = AddParams {
            to: f4_eth_addr,
            source: new_pk(),
            key: vec![0, 1, 2],
            hash: hash.0,
            size: hash.1 as usize,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                from: Some(add_params.to),
                source: add_params.source,
                hash: add_params.hash,
                size: add_params.size as u64,
                expiry: 100,
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
        rt.verify();
    }

    #[test]
    pub fn test_add_overwrite() {
        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        let rt = construct_and_verify(f4_eth_addr);
        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);

        let hash = new_hash(256);
        let add_params: AddParams = AddParams {
            to: f4_eth_addr,
            source: new_pk(),
            key: vec![0, 1, 2],
            hash: hash.0,
            size: hash.1 as usize,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                from: Some(add_params.to),
                source: add_params.source,
                hash: add_params.hash,
                size: add_params.size as u64,
                expiry: 100,
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
        rt.verify();

        let hash = new_hash(256);
        let add_params2 = AddParams {
            to: add_params.to,
            source: add_params.source,
            key: add_params.key,
            hash: hash.0,
            size: hash.1 as usize,
            metadata: HashMap::new(),
            overwrite: true,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::DeleteBlob as MethodNum,
            IpldBlock::serialize_cbor(&DeleteBlobParams(add_params.hash)).unwrap(),
            TokenAmount::from_whole(0),
            None,
            ExitCode::OK,
        );
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                from: Some(add_params2.to),
                source: add_params2.source,
                hash: add_params2.hash,
                size: add_params2.size as u64,
                expiry: 100,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            None,
            ExitCode::OK,
        );
        let result = rt
            .call::<Actor>(
                Method::AddObject as u64,
                IpldBlock::serialize_cbor(&add_params2).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Cid>()
            .unwrap();
        let state = rt.state::<State>().unwrap();
        assert_eq!(state.root, result);
        rt.verify();
    }

    #[test]
    pub fn test_add_overwrite_fail() {
        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        let rt = construct_and_verify(f4_eth_addr);
        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);

        let hash = new_hash(256);
        let add_params: AddParams = AddParams {
            to: f4_eth_addr,
            source: new_pk(),
            key: vec![0, 1, 2],
            hash: hash.0,
            size: hash.1 as usize,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                from: Some(add_params.to),
                source: add_params.source,
                hash: add_params.hash,
                size: add_params.size as u64,
                expiry: 100,
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
        rt.verify();

        let hash = new_hash(256);
        let add_params2 = AddParams {
            to: add_params.to,
            source: add_params.source,
            key: add_params.key,
            hash: hash.0,
            size: hash.1 as usize,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let result = rt.call::<Actor>(
            Method::AddObject as u64,
            IpldBlock::serialize_cbor(&add_params2).unwrap(),
        );
        assert!(result.is_err_and(|e| { e.msg().eq("asked not to overwrite") }));
        let state2 = rt.state::<State>().unwrap();
        assert_eq!(state2.root, state.root);
        rt.verify();
    }

    #[test]
    pub fn test_delete_object() {
        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        let rt = construct_and_verify(f4_eth_addr);
        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);

        let key = vec![0, 1, 2];
        let hash = new_hash(256);

        // Prerequisite for a delete operation: add to have a proper state of the actor.
        let add_params: AddParams = AddParams {
            to: f4_eth_addr,
            source: new_pk(),
            key: key.clone(),
            hash: hash.0,
            size: hash.1 as usize,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                from: Some(add_params.to),
                source: add_params.source,
                hash: add_params.hash,
                size: add_params.size as u64,
                expiry: 100,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            None,
            ExitCode::OK,
        );
        let result_add = rt
            .call::<Actor>(
                Method::AddObject as u64,
                IpldBlock::serialize_cbor(&add_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Cid>()
            .unwrap();
        let state = rt.state::<State>().unwrap();
        assert_eq!(state.root, result_add);
        rt.verify();

        // Now actually delete.
        let delete_params = DeleteParams { key: key.clone() };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::DeleteBlob as MethodNum,
            IpldBlock::serialize_cbor(&DeleteBlobParams(hash.0)).unwrap(),
            TokenAmount::from_whole(0),
            None,
            ExitCode::OK,
        );
        let result_delete = rt.call::<Actor>(
            Method::DeleteObject as u64,
            IpldBlock::serialize_cbor(&delete_params).unwrap(),
        );
        assert!(result_delete.is_ok());
        rt.verify();
    }

    #[test]
    pub fn test_get_object_none() {
        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        let rt = construct_and_verify(f4_eth_addr);
        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);

        let get_params = GetParams { key: vec![0, 1, 2] };
        rt.expect_validate_caller_any();
        let result = rt
            .call::<Actor>(
                Method::GetObject as u64,
                IpldBlock::serialize_cbor(&get_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Option<GotObject>>();
        assert!(result.is_ok());
        assert_eq!(result, Ok(None));
        rt.verify();
    }

    #[test]
    pub fn test_get_object() {
        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        let rt = construct_and_verify(f4_eth_addr);
        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);

        let key = vec![0, 1, 2];
        let hash = new_hash(256);

        // Prerequisite for a delete operation: add to have a proper state of the actor.
        let add_params: AddParams = AddParams {
            to: f4_eth_addr,
            source: new_pk(),
            key: key.clone(),
            hash: hash.0,
            size: hash.1 as usize,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                from: Some(add_params.to),
                source: add_params.source,
                hash: add_params.hash,
                size: add_params.size as u64,
                expiry: 100,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            None,
            ExitCode::OK,
        );
        rt.call::<Actor>(
            Method::AddObject as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        )
        .unwrap()
        .unwrap()
        .deserialize::<Cid>()
        .unwrap();
        rt.verify();

        let blob = Blob {
            size: add_params.size as u64,
            expiry: 100 as ChainEpoch,
            source: add_params.source,
            resolved: true,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::GetBlob as MethodNum,
            IpldBlock::serialize_cbor(&GetBlobParams(add_params.hash)).unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Some(&blob)).unwrap(),
            ExitCode::OK,
        );
        let get_params = GetParams { key };
        let result = rt
            .call::<Actor>(
                Method::GetObject as u64,
                IpldBlock::serialize_cbor(&get_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Option<GotObject>>();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(GotObject {
                hash: hash.0,
                size: blob.size as usize,
                expiry: blob.expiry,
                metadata: add_params.metadata
            })
        );
        rt.verify();
    }
}
