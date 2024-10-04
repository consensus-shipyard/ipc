// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fendermint_actor_blobs_shared::state::{Blob, BlobStatus};
use fendermint_actor_blobs_shared::{add_blob, delete_blob, get_blob};
use fendermint_actor_machine::MachineActor;
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorError,
};
use fvm_ipld_hamt::BytesKey;

use crate::shared::{
    AddParams, DeleteParams, GetParams, ListObjectsReturn, ListParams, Method, Object,
    OBJECTSTORE_ACTOR_NAME,
};
use crate::state::{ObjectState, State};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

pub struct Actor;

impl Actor {
    fn add_object(rt: &impl Runtime, params: AddParams) -> Result<Cid, ActorError> {
        // Self::ensure_write_allowed(rt)?;
        rt.validate_immediate_caller_accept_any()?;
        let state = rt.state::<State>()?;
        let key = BytesKey(params.key);
        if let Some(object) = state.get(rt.store(), &key)? {
            if params.overwrite {
                delete_blob(rt, Some(state.owner), object.hash)?;
            } else {
                return Err(ActorError::illegal_state(
                    "key exists; use overwrite".into(),
                ));
            }
        }
        // Add blob for object
        add_blob(
            rt,
            Some(state.owner),
            params.source,
            params.hash,
            params.size,
            params.ttl,
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
        })?;
        Ok(root)
    }

    fn delete_object(rt: &impl Runtime, params: DeleteParams) -> Result<Cid, ActorError> {
        // Self::ensure_write_allowed(rt)?;
        rt.validate_immediate_caller_accept_any()?;
        let state = rt.state::<State>()?;
        let key = BytesKey(params.0);
        let object = state
            .get(rt.store(), &key)?
            .ok_or(ActorError::illegal_state("object not found".into()))?;
        // Delete blob for object
        delete_blob(rt, Some(state.owner), object.hash)?;
        // Update state
        let res = rt.transaction(|st: &mut State, rt| st.delete(rt.store(), &key))?;
        Ok(res.1)
    }

    fn get_object(rt: &impl Runtime, params: GetParams) -> Result<Option<Object>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let state = rt.state::<State>()?;
        let key = BytesKey(params.0);
        if let Some(object_state) = state.get(rt.store(), &key)? {
            if let Some(blob) = get_blob(rt, object_state.hash)? {
                let object = build_object(&blob, &object_state)?;
                Ok(object)
            } else {
                Ok(None)
            }
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
        let prefixes = rt.state::<State>()?.list(
            rt.store(),
            params.prefix,
            params.delimiter,
            params.offset,
            params.limit,
            |key: Vec<u8>, object_state: ObjectState| -> anyhow::Result<(), ActorError> {
                if let Some(blob) = get_blob(rt, object_state.hash)? {
                    if let Some(object) = build_object(&blob, &object_state)? {
                        objects.push((key, Some(object)));
                    }
                } else {
                    objects.push((key, None));
                }
                Ok(())
            },
        )?;
        Ok(ListObjectsReturn {
            objects,
            common_prefixes: prefixes,
        })
    }
}

/// Build an object from its state and blob.
fn build_object(
    blob: &Blob,
    object_state: &ObjectState,
) -> anyhow::Result<Option<Object>, ActorError> {
    match blob.status {
        BlobStatus::Resolved => {
            let max_sub = blob
                .subs
                .values()
                .max_by_key(|sub| sub.expiry)
                .ok_or_else(|| {
                    ActorError::illegal_state(
                        "blob has no subscriptions; this should not happen".into(),
                    )
                })?;
            Ok(Some(Object {
                hash: object_state.hash,
                size: blob.size,
                expiry: max_sub.expiry,
                metadata: object_state.metadata.clone(),
            }))
        }
        BlobStatus::Pending | BlobStatus::Failed => Ok(None),
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
        Init => init,
        GetAddress => get_address,
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
    use fendermint_actor_machine::{ConstructorParams, WriteAccess};
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
    use fvm_shared::sys::SendFlags;
    use fvm_shared::MethodNum;
    use rand::RngCore;

    fn construct_and_verify(owner: Address) -> MockRuntime {
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
                    owner,
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
            source: new_pk(),
            key: vec![0, 1, 2],
            hash: hash.0,
            size: hash.1,
            ttl: None,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                sponsor: Some(f4_eth_addr),
                source: add_params.source,
                hash: add_params.hash,
                size: add_params.size,
                ttl: add_params.ttl,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
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
            source: new_pk(),
            key: vec![0, 1, 2],
            hash: hash.0,
            size: hash.1,
            ttl: None,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                sponsor: Some(f4_eth_addr),
                source: add_params.source,
                hash: add_params.hash,
                size: add_params.size,
                ttl: add_params.ttl,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
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
            source: add_params.source,
            key: add_params.key,
            hash: hash.0,
            size: hash.1,
            ttl: None,
            metadata: HashMap::new(),
            overwrite: true,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::DeleteBlob as MethodNum,
            IpldBlock::serialize_cbor(&DeleteBlobParams {
                sponsor: Some(f4_eth_addr),
                hash: add_params.hash,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            None,
            ExitCode::OK,
        );
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                sponsor: Some(f4_eth_addr),
                source: add_params2.source,
                hash: add_params2.hash,
                size: add_params2.size,
                ttl: add_params2.ttl,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
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
            source: new_pk(),
            key: vec![0, 1, 2],
            hash: hash.0,
            size: hash.1,
            ttl: None,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                sponsor: Some(f4_eth_addr),
                source: add_params.source,
                hash: add_params.hash,
                size: add_params.size,
                ttl: add_params.ttl,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
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
            source: add_params.source,
            key: add_params.key,
            hash: hash.0,
            size: hash.1,
            ttl: None,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let result = rt.call::<Actor>(
            Method::AddObject as u64,
            IpldBlock::serialize_cbor(&add_params2).unwrap(),
        );
        assert!(result.is_err_and(|e| { e.msg().eq("key exists; use overwrite") }));
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
            source: new_pk(),
            key: key.clone(),
            hash: hash.0,
            size: hash.1,
            ttl: None,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                sponsor: Some(f4_eth_addr),
                source: add_params.source,
                hash: add_params.hash,
                size: add_params.size,
                ttl: add_params.ttl,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
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
        let delete_params = DeleteParams(key.clone());
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::DeleteBlob as MethodNum,
            IpldBlock::serialize_cbor(&DeleteBlobParams {
                sponsor: Some(f4_eth_addr),
                hash: add_params.hash,
            })
            .unwrap(),
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

        let get_params = GetParams(vec![0, 1, 2]);
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
        let ttl = ChainEpoch::from(3600);

        // Prerequisite for a delete operation: add to have a proper state of the actor.
        let add_params: AddParams = AddParams {
            source: new_pk(),
            key: key.clone(),
            hash: hash.0,
            size: hash.1,
            ttl: Some(ttl),
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                sponsor: Some(f4_eth_addr),
                source: add_params.source,
                hash: add_params.hash,
                size: add_params.size,
                ttl: add_params.ttl,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
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
                    added: 0,
                    expiry: ttl,
                    auto_renew: false,
                    source: add_params.source,
                    delegate: Some((f4_eth_addr, f4_eth_addr)),
                },
            )]),
            status: BlobStatus::Resolved,
        };
        rt.expect_validate_caller_any();
        rt.expect_send(
            BLOBS_ACTOR_ADDR,
            BlobMethod::GetBlob as MethodNum,
            IpldBlock::serialize_cbor(&GetBlobParams(add_params.hash)).unwrap(),
            TokenAmount::from_whole(0),
            None,
            SendFlags::READ_ONLY,
            IpldBlock::serialize_cbor(&Some(&blob)).unwrap(),
            ExitCode::OK,
            None,
        );
        let get_params = GetParams(key);
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
                expiry: ttl,
                metadata: add_params.metadata,
            })
        );
        rt.verify();
    }
}
