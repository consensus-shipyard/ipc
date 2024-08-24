// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::anyhow;
use cid::Cid;
use fendermint_actor_blobs_shared::state::{Blob, BlobStatus};
use fendermint_actor_blobs_shared::{add_blob, delete_blob, get_blob};
use fendermint_actor_machine::{ConstructorParams, MachineActor};
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorDowncast, ActorError, FIRST_EXPORTED_METHOD_NUMBER, INIT_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_ipld_hamt::BytesKey;
use fvm_shared::{error::ExitCode, MethodNum};

use crate::shared::{
    AddParams, DeleteParams, GetParams, ListObjectsReturn, ListParams, Method, Object,
    OBJECTSTORE_ACTOR_NAME,
};
use crate::state::{ObjectState, State};

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
        .map_err(to_state_error("failed to construct empty store"))?;
        rt.create(&state)
    }

    fn add_object(rt: &impl Runtime, params: AddParams) -> Result<Cid, ActorError> {
        Self::ensure_write_allowed(rt)?;
        let key = BytesKey(params.key);
        if let Some(object) = retrieve_object_state(rt, &key)? {
            if params.overwrite {
                delete_blob(rt, object.hash)?;
            } else {
                return Err(ActorError::illegal_state(
                    "key exists; use overwrite to force".into(),
                ));
            }
        }
        // Add object's blob
        add_blob(
            rt,
            params.to,
            params.source,
            params.hash,
            params.size,
            rt.curr_epoch() + 100,
        )?;
        // Update state
        let root = rt.transaction(|st: &mut State, rt| {
            st.add(
                rt.store(),
                key,
                params.hash,
                params.metadata,
                params.overwrite,
            )
            .map_err(to_state_error("failed to add object"))
        })?;
        Ok(root)
    }

    fn delete_object(rt: &impl Runtime, params: DeleteParams) -> Result<Cid, ActorError> {
        Self::ensure_write_allowed(rt)?;
        let key = BytesKey(params.key);
        let object = retrieve_object_state(rt, &key)?
            .ok_or(ActorError::illegal_state("object not found".into()))?;
        // Delete object's blob
        delete_blob(rt, object.hash)?;
        // Update state
        let res = rt.transaction(|st: &mut State, rt| {
            st.delete(rt.store(), &key)
                .map_err(to_state_error("failed to delete object"))
        })?;
        Ok(res.1)
    }

    fn get_object(rt: &impl Runtime, params: GetParams) -> Result<Option<Object>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        if let Some(object_state) = retrieve_object_state(rt, &BytesKey(params.key))? {
            let blob = get_blob(rt, object_state.hash)?;
            let object = build_object(&blob, &object_state)
                .map_err(to_state_error("failed to build object"))?;
            Ok(object)
        } else {
            Ok(None)
        }
    }

    fn list_objects(
        rt: &impl Runtime,
        params: ListParams,
    ) -> Result<ListObjectsReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let mut objects = Vec::new();
        let prefixes = rt
            .state::<State>()?
            .list(
                rt.store(),
                params.prefix,
                params.delimiter,
                params.offset,
                params.limit,
                |key: Vec<u8>, object_state: ObjectState| -> anyhow::Result<()> {
                    let blob = get_blob(rt, object_state.hash)?;
                    if let Some(object) = build_object(&blob, &object_state)? {
                        objects.push((key, object));
                    }
                    Ok(())
                },
            )
            .map_err(to_state_error("failed to list objects"))?;
        Ok(ListObjectsReturn {
            objects,
            common_prefixes: prefixes,
        })
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

/// Retrieve an object from the state.
fn retrieve_object_state(
    rt: &impl Runtime,
    key: &BytesKey,
) -> Result<Option<ObjectState>, ActorError> {
    rt.state::<State>()?
        .get(rt.store(), key)
        .map_err(to_state_error("failed to retrieve object"))
}

/// Build an object from its state and blob.
fn build_object(blob: &Blob, object_state: &ObjectState) -> anyhow::Result<Option<Object>> {
    match blob.status {
        BlobStatus::Resolved => {
            let max_sub = blob
                .subs
                .values()
                .max_by_key(|sub| sub.expiry)
                .ok_or_else(|| anyhow!("blob has no subscriptions; this should not happen"))?;
            Ok(Some(Object {
                hash: object_state.hash,
                size: blob.size,
                expiry: max_sub.expiry,
                metadata: object_state.metadata.clone(),
            }))
        }
        BlobStatus::Added(_) | BlobStatus::Failed => Ok(None),
    }
}

fn to_state_error(message: &'static str) -> impl FnOnce(anyhow::Error) -> ActorError {
    move |e| e.downcast_default(ExitCode::USR_ILLEGAL_STATE, message)
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

    use std::collections::HashMap;

    use cid::Cid;
    use fendermint_actor_blobs_shared::params::{AddBlobParams, DeleteBlobParams, GetBlobParams};
    use fendermint_actor_blobs_shared::state::{BlobStatus, Hash, PublicKey, Subscription};
    use fendermint_actor_blobs_shared::{Method as BlobMethod, BLOBS_ACTOR_ADDR};
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
            size: hash.1,
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
                size: add_params.size,
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
            size: hash.1,
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
                size: add_params.size,
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
            size: hash.1,
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
                size: add_params2.size,
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
            size: hash.1,
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
                size: add_params.size,
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
            size: hash.1,
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
            size: hash.1,
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
                size: add_params.size,
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

        // Now actually delete it.
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
            .deserialize::<Option<Object>>();
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
        let expiry = ChainEpoch::from(100);

        // Prerequisite for a delete operation: add to have a proper state of the actor.
        let add_params: AddParams = AddParams {
            to: f4_eth_addr,
            source: new_pk(),
            key: key.clone(),
            hash: hash.0,
            size: hash.1,
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
                size: add_params.size,
                expiry,
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
            size: add_params.size,
            subs: HashMap::from([(
                f4_eth_addr,
                Subscription {
                    expiry,
                    source: add_params.source,
                },
            )]),
            status: BlobStatus::Added(rt.curr_epoch()),
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
            .deserialize::<Option<Object>>();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(Object {
                hash: hash.0,
                size: blob.size,
                expiry,
                metadata: add_params.metadata,
            })
        );
        rt.verify();
    }
}
