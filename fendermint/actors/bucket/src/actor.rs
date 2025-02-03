// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fendermint_actor_blobs_shared::state::{Blob, BlobStatus, SubscriptionId};
use fendermint_actor_blobs_shared::{add_blob, delete_blob, get_blob, overwrite_blob};
use fendermint_actor_machine::events::EventBuilder;
use fendermint_actor_machine::MachineActor;
use fil_actors_evm_shared::uints::U256;
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorError,
};
use fvm_ipld_hamt::BytesKey;
use fvm_shared::address::Address;

use crate::shared::{
    AddParams, DeleteParams, GetParams, ListObjectsReturn, ListParams, Method, Object,
    BUCKET_ACTOR_NAME,
};
use crate::state::{ObjectState, State};
use crate::{
    UpdateObjectMetadataParams, MAX_METADATA_ENTRIES, MAX_METADATA_KEY_SIZE,
    MAX_METADATA_VALUE_SIZE,
};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(Actor);

const OBJECT_ADDED_EVENT: &str = "ObjectAdded(bytes32,uint256,uint256)";

pub struct Actor;

impl Actor {
    fn add_object(rt: &impl Runtime, params: AddParams) -> Result<Object, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let state = rt.state::<State>()?;
        let key = BytesKey(params.key.clone());
        let metadata = params.metadata.clone();
        validate_metadata(&metadata)?;

        let sub_id = get_blob_id(&state, params.key)?;
        let sub = if let Some(object) = state.get(rt.store(), &key)? {
            // If we have existing blob
            if params.overwrite {
                // Overwrite if the flag is passed
                overwrite_blob(
                    rt,
                    object.hash,
                    sub_id,
                    params.hash,
                    Some(state.owner),
                    params.source,
                    params.recovery_hash,
                    params.size,
                    params.ttl,
                )?
            } else {
                // Return an error if no overwrite flag gets passed
                return Err(ActorError::illegal_state(
                    "key exists; use overwrite".into(),
                ));
            }
        } else {
            // No object found, just a new blob
            add_blob(
                rt,
                sub_id,
                params.hash,
                Some(state.owner),
                params.source,
                params.recovery_hash,
                params.size,
                params.ttl,
            )?
        };
        // Update state
        rt.transaction(|st: &mut State, rt| {
            st.add(
                rt.store(),
                key,
                params.hash,
                params.size,
                params.metadata,
                params.overwrite,
            )
        })?;

        EventBuilder::new(OBJECT_ADDED_EVENT)
            .param_indexed(params.hash.0)
            .param_indexed(U256::from(params.size))
            .param_indexed(U256::from(sub.expiry))
            .emit(rt)?;

        Ok(Object {
            hash: params.hash,
            recovery_hash: params.recovery_hash,
            size: params.size,
            expiry: sub.expiry,
            metadata,
        })
    }

    fn delete_object(rt: &impl Runtime, params: DeleteParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let state = rt.state::<State>()?;
        let key = BytesKey(params.0.clone());
        let object = state
            .get(rt.store(), &key)?
            .ok_or(ActorError::illegal_state("object not found".into()))?;
        // Delete blob for object
        let sub_id = get_blob_id(&state, params.0)?;
        delete_blob(rt, sub_id, object.hash, Some(state.owner))?;
        // Update state
        rt.transaction(|st: &mut State, rt| st.delete(rt.store(), &key))?;
        Ok(())
    }

    fn get_object(rt: &impl Runtime, params: GetParams) -> Result<Option<Object>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let state = rt.state::<State>()?;
        let owner = state.owner;
        let key = BytesKey(params.0.clone());
        let sub_id = get_blob_id(&state, params.0)?;
        if let Some(object_state) = state.get(rt.store(), &key)? {
            if let Some(blob) = get_blob(rt, object_state.hash)? {
                let object = build_object(&blob, &object_state, sub_id, owner)?;
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
        let state = rt.state::<State>()?;
        let start_key = params.start_key.map(BytesKey::from);
        let (prefixes, next_key) = state.list(
            rt.store(),
            params.prefix,
            params.delimiter,
            start_key.as_ref(),
            params.limit,
            |key: Vec<u8>, object_state: ObjectState| -> anyhow::Result<(), ActorError> {
                objects.push((key, object_state));
                Ok(())
            },
        )?;
        let next_key = next_key.map(|key| key.0);
        Ok(ListObjectsReturn {
            objects,
            next_key,
            common_prefixes: prefixes,
        })
    }

    fn update_object_metadata(
        rt: &impl Runtime,
        params: UpdateObjectMetadataParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let state = rt.state::<State>()?;
        let key = BytesKey(params.key.clone());
        let metadata = params.metadata.clone();

        let mut object = state
            .get(rt.store(), &key)?
            .ok_or(ActorError::illegal_state("object not found".into()))?;

        validate_metadata_optional(&metadata)?;

        rt.transaction(|st: &mut State, rt| {
            for (key, val) in metadata {
                match val {
                    Some(v) => {
                        object
                            .metadata
                            .entry(key)
                            .and_modify(|s| *s = v.clone())
                            .or_insert(v);
                    }
                    None => {
                        object.metadata.remove(&key);
                    }
                }
            }

            if object.metadata.len() as u32 > MAX_METADATA_ENTRIES {
                return Err(ActorError::illegal_state(format!(
                    "the maximum metadata entries allowed is {}",
                    MAX_METADATA_ENTRIES
                )));
            }

            st.add(
                rt.store(),
                key,
                object.hash,
                object.size,
                object.metadata,
                true,
            )
        })?;
        Ok(())
    }
}

/// Returns a blob subscription ID specific to this machine and object key.
fn get_blob_id(state: &State, key: Vec<u8>) -> anyhow::Result<SubscriptionId, ActorError> {
    let mut data = state.address.get()?.payload_bytes();
    data.extend(key);
    let id = blake3::hash(&data).to_hex().to_string();
    SubscriptionId::new(&id)
}

/// Build an object from its state and blob.
fn build_object(
    blob: &Blob,
    object_state: &ObjectState,
    sub_id: SubscriptionId,
    subscriber: Address,
) -> anyhow::Result<Option<Object>, ActorError> {
    match blob.status {
        BlobStatus::Resolved => {
            let group = blob
                .subscribers
                .get(&subscriber.to_string())
                .ok_or_else(|| {
                    ActorError::illegal_state(format!(
                        "owner {} is not subscribed to blob {}; this should not happen",
                        subscriber, object_state.hash
                    ))
                })?;
            let (expiry, _) = group.max_expiries(&sub_id, None);
            if let Some(expiry) = expiry {
                Ok(Some(Object {
                    hash: object_state.hash,
                    recovery_hash: blob.metadata_hash,
                    size: blob.size,
                    expiry,
                    metadata: object_state.metadata.clone(),
                }))
            } else {
                Err(ActorError::illegal_state(format!(
                    "subscription group is empty for blob {}; this should not happen",
                    object_state.hash,
                )))
            }
        }
        BlobStatus::Added | BlobStatus::Pending | BlobStatus::Failed => Ok(None),
    }
}

fn validate_metadata(metadata: &HashMap<String, String>) -> anyhow::Result<(), ActorError> {
    if metadata.len() as u32 > MAX_METADATA_ENTRIES {
        return Err(ActorError::illegal_state(format!(
            "the maximum metadata entries allowed is {}",
            MAX_METADATA_ENTRIES
        )));
    }

    for (key, value) in metadata {
        if key.len() as u32 > MAX_METADATA_KEY_SIZE {
            return Err(ActorError::illegal_state(format!(
                "key must be less than or equal to {}",
                MAX_METADATA_KEY_SIZE
            )));
        }

        if value.is_empty() || value.len() as u32 > MAX_METADATA_VALUE_SIZE {
            return Err(ActorError::illegal_state(format!(
                "value must non-empty and less than or equal to {}",
                MAX_METADATA_VALUE_SIZE
            )));
        }
    }

    Ok(())
}

fn validate_metadata_optional(
    metadata: &HashMap<String, Option<String>>,
) -> anyhow::Result<(), ActorError> {
    for (key, value) in metadata {
        if key.len() as u32 > MAX_METADATA_KEY_SIZE {
            return Err(ActorError::illegal_state(format!(
                "key must be less than or equal to {}",
                MAX_METADATA_KEY_SIZE
            )));
        }

        if let Some(value) = value {
            if value.is_empty() || value.len() as u32 > MAX_METADATA_VALUE_SIZE {
                return Err(ActorError::illegal_state(format!(
                    "value must non-empty and less than or equal to {}",
                    MAX_METADATA_VALUE_SIZE
                )));
            }
        }
    }

    Ok(())
}

impl MachineActor for Actor {
    type State = State;
}

impl ActorCode for Actor {
    type Methods = Method;

    fn name() -> &'static str {
        BUCKET_ACTOR_NAME
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
        UpdateObjectMetadata => update_object_metadata,
        _ => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use fendermint_actor_blobs_shared::params::{
        AddBlobParams, DeleteBlobParams, GetBlobParams, OverwriteBlobParams,
    };
    use fendermint_actor_blobs_shared::state::{Subscription, SubscriptionGroup};
    use fendermint_actor_blobs_shared::{Method as BlobMethod, BLOBS_ACTOR_ADDR};
    use fendermint_actor_blobs_testing::{new_hash, new_pk};
    use fendermint_actor_machine::{ConstructorParams, InitParams};
    use fil_actors_runtime::runtime::Runtime;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ADM_ACTOR_CODE_ID, ETHACCOUNT_ACTOR_CODE_ID, INIT_ACTOR_CODE_ID,
    };
    use fil_actors_runtime::{ADM_ACTOR_ADDR, INIT_ACTOR_ADDR};
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::clock::ChainEpoch;
    use fvm_shared::econ::TokenAmount;
    use fvm_shared::error::ExitCode;
    use fvm_shared::sys::SendFlags;
    use fvm_shared::MethodNum;

    fn get_runtime() -> (MockRuntime, Address) {
        let origin = Address::new_id(110);
        let rt = construct_and_verify(origin);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, origin);
        rt.set_origin(origin);
        (rt, origin)
    }

    fn construct_and_verify(owner: Address) -> MockRuntime {
        let buck_addr = Address::new_id(111);
        let rt = MockRuntime {
            receiver: buck_addr,
            ..Default::default()
        };
        rt.set_caller(*INIT_ACTOR_CODE_ID, INIT_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![INIT_ACTOR_ADDR]);
        let metadata = HashMap::new();
        let actor_construction = rt
            .call::<Actor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&ConstructorParams { owner, metadata }).unwrap(),
            )
            .unwrap();
        expect_empty(actor_construction);
        rt.verify();
        rt.set_caller(*ADM_ACTOR_CODE_ID, ADM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![ADM_ACTOR_ADDR]);
        let actor_init = rt
            .call::<Actor>(
                Method::Init as u64,
                IpldBlock::serialize_cbor(&InitParams { address: buck_addr }).unwrap(),
            )
            .unwrap();
        expect_empty(actor_init);
        rt.verify();
        rt.reset();
        rt
    }

    fn expect_emitted_add_event(rt: &MockRuntime, add_params: &AddParams) {
        let event = EventBuilder::new(OBJECT_ADDED_EVENT)
            .param_indexed(add_params.hash.0)
            .param_indexed(U256::from(add_params.size))
            .param_indexed(U256::from(0))
            .build()
            .unwrap();
        rt.expect_emitted_event(event);
    }

    #[test]
    pub fn test_add_object() {
        let (rt, origin) = get_runtime();

        // Add an object
        let hash = new_hash(256);
        let key = vec![0, 1, 2];
        let add_params: AddParams = AddParams {
            source: new_pk(),
            key: key.clone(),
            hash: hash.0,
            recovery_hash: new_hash(256).0,
            size: hash.1,
            ttl: None,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let state = rt.state::<State>().unwrap();
        let sub_id = get_blob_id(&state, key).unwrap();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                sponsor: Some(origin),
                source: add_params.source,
                hash: add_params.hash,
                metadata_hash: add_params.recovery_hash,
                id: sub_id,
                size: add_params.size,
                ttl: add_params.ttl,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
            ExitCode::OK,
        );
        expect_emitted_add_event(&rt, &add_params);
        let result = rt
            .call::<Actor>(
                Method::AddObject as u64,
                IpldBlock::serialize_cbor(&add_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Object>()
            .unwrap();
        assert_eq!(add_params.hash, result.hash);
        assert_eq!(add_params.recovery_hash, result.recovery_hash);
        assert_eq!(add_params.size, result.size);
        assert_eq!(add_params.metadata, result.metadata);
        rt.verify();
    }

    #[test]
    pub fn test_add_overwrite() {
        let (rt, origin) = get_runtime();

        // Add an object
        let hash = new_hash(256);
        let key = vec![0, 1, 2];
        let add_params: AddParams = AddParams {
            source: new_pk(),
            key: key.clone(),
            hash: hash.0,
            recovery_hash: new_hash(256).0,
            size: hash.1,
            ttl: None,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let state = rt.state::<State>().unwrap();
        let sub_id = get_blob_id(&state, key).unwrap();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                sponsor: Some(origin),
                source: add_params.source,
                hash: add_params.hash,
                metadata_hash: add_params.recovery_hash,
                id: sub_id.clone(),
                size: add_params.size,
                ttl: add_params.ttl,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
            ExitCode::OK,
        );
        expect_emitted_add_event(&rt, &add_params);
        let result = rt
            .call::<Actor>(
                Method::AddObject as u64,
                IpldBlock::serialize_cbor(&add_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Object>()
            .unwrap();
        assert_eq!(add_params.hash, result.hash);
        assert_eq!(add_params.metadata, result.metadata);
        assert_eq!(add_params.recovery_hash, result.recovery_hash);
        assert_eq!(add_params.size, result.size);
        rt.verify();

        // Overwrite object (old blob is deleted)
        let hash = new_hash(256);
        let add_params2 = AddParams {
            source: add_params.source,
            key: add_params.key,
            hash: hash.0,
            recovery_hash: new_hash(256).0,
            size: hash.1,
            ttl: None,
            metadata: HashMap::new(),
            overwrite: true,
        };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::OverwriteBlob as MethodNum,
            IpldBlock::serialize_cbor(&OverwriteBlobParams {
                old_hash: add_params.hash,
                add: AddBlobParams {
                    id: sub_id,
                    hash: add_params2.hash,
                    sponsor: Some(origin),
                    source: add_params2.source,
                    metadata_hash: add_params2.recovery_hash,
                    size: add_params2.size,
                    ttl: add_params2.ttl,
                },
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
            ExitCode::OK,
        );
        expect_emitted_add_event(&rt, &add_params2);
        let result = rt
            .call::<Actor>(
                Method::AddObject as u64,
                IpldBlock::serialize_cbor(&add_params2).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Object>()
            .unwrap();
        assert_eq!(add_params2.hash, result.hash);
        assert_eq!(add_params2.metadata, result.metadata);
        assert_eq!(add_params2.recovery_hash, result.recovery_hash);
        assert_eq!(add_params2.size, result.size);
        rt.verify();
    }

    #[test]
    pub fn test_add_overwrite_fail() {
        let (rt, origin) = get_runtime();

        // Add an object
        let hash = new_hash(256);
        let key = vec![0, 1, 2];
        let add_params: AddParams = AddParams {
            source: new_pk(),
            key: key.clone(),
            hash: hash.0,
            size: hash.1,
            recovery_hash: new_hash(256).0,
            ttl: None,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let state = rt.state::<State>().unwrap();
        let sub_id = get_blob_id(&state, key).unwrap();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                sponsor: Some(origin),
                source: add_params.source,
                hash: add_params.hash,
                metadata_hash: add_params.recovery_hash,
                id: sub_id,
                size: add_params.size,
                ttl: add_params.ttl,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
            ExitCode::OK,
        );
        expect_emitted_add_event(&rt, &add_params);
        let result = rt
            .call::<Actor>(
                Method::AddObject as u64,
                IpldBlock::serialize_cbor(&add_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Object>()
            .unwrap();
        let state = rt.state::<State>().unwrap();
        assert_eq!(add_params.hash, result.hash);
        assert_eq!(add_params.metadata, result.metadata);
        assert_eq!(add_params.recovery_hash, result.recovery_hash);
        assert_eq!(add_params.size, result.size);
        rt.verify();

        // Try to overwrite
        let hash = new_hash(256);
        let add_params2 = AddParams {
            source: add_params.source,
            key: add_params.key,
            hash: hash.0,
            size: hash.1,
            recovery_hash: new_hash(256).0,
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
        let (rt, origin) = get_runtime();

        // Add an object
        let key = vec![0, 1, 2];
        let hash = new_hash(256);
        let add_params: AddParams = AddParams {
            source: new_pk(),
            key: key.clone(),
            hash: hash.0,
            size: hash.1,
            recovery_hash: new_hash(256).0,
            ttl: None,
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let state = rt.state::<State>().unwrap();
        let sub_id = get_blob_id(&state, key.clone()).unwrap();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                sponsor: Some(origin),
                source: add_params.source,
                hash: add_params.hash,
                id: sub_id.clone(),
                size: add_params.size,
                metadata_hash: add_params.recovery_hash,
                ttl: add_params.ttl,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
            ExitCode::OK,
        );
        expect_emitted_add_event(&rt, &add_params);
        let result_add = rt
            .call::<Actor>(
                Method::AddObject as u64,
                IpldBlock::serialize_cbor(&add_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Object>()
            .unwrap();
        assert_eq!(add_params.hash, result_add.hash);
        assert_eq!(add_params.metadata, result_add.metadata);
        assert_eq!(add_params.recovery_hash, result_add.recovery_hash);
        assert_eq!(add_params.size, result_add.size);
        rt.verify();

        // Delete object
        let delete_params = DeleteParams(key);
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::DeleteBlob as MethodNum,
            IpldBlock::serialize_cbor(&DeleteBlobParams {
                sponsor: Some(origin),
                hash: add_params.hash,
                id: sub_id,
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
        let (rt, _) = get_runtime();

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
        let (rt, origin) = get_runtime();

        // Add an object
        let key = vec![0, 1, 2];
        let hash = new_hash(256);
        let ttl = ChainEpoch::from(3600);
        let add_params: AddParams = AddParams {
            source: new_pk(),
            key: key.clone(),
            hash: hash.0,
            size: hash.1,
            recovery_hash: new_hash(256).0,
            ttl: Some(ttl),
            metadata: HashMap::new(),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let state = rt.state::<State>().unwrap();
        let sub_id = get_blob_id(&state, key.clone()).unwrap();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                sponsor: Some(origin),
                source: add_params.source,
                hash: add_params.hash,
                id: sub_id.clone(),
                size: add_params.size,
                metadata_hash: add_params.recovery_hash,
                ttl: add_params.ttl,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
            ExitCode::OK,
        );
        expect_emitted_add_event(&rt, &add_params);
        rt.call::<Actor>(
            Method::AddObject as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        )
        .unwrap()
        .unwrap()
        .deserialize::<Object>()
        .unwrap();
        rt.verify();

        // Get the object
        let blob = Blob {
            size: add_params.size,
            subscribers: HashMap::from([(
                origin.to_string(),
                SubscriptionGroup {
                    subscriptions: HashMap::from([(
                        sub_id.to_string(),
                        Subscription {
                            added: 0,
                            expiry: ttl,
                            source: add_params.source,
                            delegate: Some(origin),
                            failed: false,
                        },
                    )]),
                },
            )]),
            status: BlobStatus::Resolved,
            metadata_hash: add_params.recovery_hash,
        };
        rt.expect_validate_caller_any();
        rt.expect_send(
            BLOBS_ACTOR_ADDR,
            BlobMethod::GetBlob as MethodNum,
            IpldBlock::serialize_cbor(&GetBlobParams(add_params.hash)).unwrap(),
            TokenAmount::from_whole(0),
            None,
            SendFlags::READ_ONLY,
            IpldBlock::serialize_cbor(&Some(blob)).unwrap(),
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
                recovery_hash: add_params.recovery_hash,
                size: add_params.size,
                expiry: ttl,
                metadata: add_params.metadata,
            })
        );
        rt.verify();
    }

    #[test]
    pub fn test_update_object_metadata() {
        let (rt, origin) = get_runtime();

        // Add an object
        let hash = new_hash(256);
        let key = vec![0, 1, 2];
        let add_params: AddParams = AddParams {
            source: new_pk(),
            key: key.clone(),
            hash: hash.0,
            size: hash.1,
            recovery_hash: new_hash(256).0,
            ttl: None,
            metadata: HashMap::from([("foo".into(), "bar".into()), ("foo2".into(), "bar".into())]),
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let state = rt.state::<State>().unwrap();
        let sub_id = get_blob_id(&state, key.clone()).unwrap();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::AddBlob as MethodNum,
            IpldBlock::serialize_cbor(&AddBlobParams {
                sponsor: Some(origin),
                source: add_params.source,
                hash: add_params.hash,
                metadata_hash: add_params.recovery_hash,
                id: sub_id,
                size: add_params.size,
                ttl: add_params.ttl,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription::default()).unwrap(),
            ExitCode::OK,
        );
        expect_emitted_add_event(&rt, &add_params);
        let result = rt
            .call::<Actor>(
                Method::AddObject as u64,
                IpldBlock::serialize_cbor(&add_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Object>()
            .unwrap();
        let state = rt.state::<State>().unwrap();
        assert_eq!(add_params.hash, result.hash);
        assert_eq!(add_params.metadata, result.metadata);
        assert_eq!(add_params.recovery_hash, result.recovery_hash);
        assert_eq!(add_params.size, result.size);
        rt.verify();

        // Update metadata
        let update_object_params = UpdateObjectMetadataParams {
            key: add_params.key,
            metadata: HashMap::from([
                ("foo".into(), Some("zar".into())),
                ("foo2".into(), None),
                ("foo3".into(), Some("bar".into())),
            ]),
        };
        rt.expect_validate_caller_any();
        let result = rt.call::<Actor>(
            Method::UpdateObjectMetadata as u64,
            IpldBlock::serialize_cbor(&update_object_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Get the object and check metadata
        let sub_id = get_blob_id(&state, key.clone()).unwrap();
        let blob = Blob {
            size: add_params.size,
            subscribers: HashMap::from([(
                origin.to_string(),
                SubscriptionGroup {
                    subscriptions: HashMap::from([(
                        sub_id.to_string(),
                        Subscription {
                            added: 0,
                            expiry: ChainEpoch::from(3600),
                            source: add_params.source,
                            delegate: Some(origin),
                            failed: false,
                        },
                    )]),
                },
            )]),
            status: BlobStatus::Resolved,
            metadata_hash: add_params.recovery_hash,
        };
        rt.expect_validate_caller_any();
        rt.expect_send(
            BLOBS_ACTOR_ADDR,
            BlobMethod::GetBlob as MethodNum,
            IpldBlock::serialize_cbor(&GetBlobParams(add_params.hash)).unwrap(),
            TokenAmount::from_whole(0),
            None,
            SendFlags::READ_ONLY,
            IpldBlock::serialize_cbor(&Some(blob)).unwrap(),
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
                recovery_hash: add_params.recovery_hash,
                size: add_params.size,
                expiry: ChainEpoch::from(3600),
                metadata: HashMap::from([
                    ("foo".into(), "zar".into()),
                    ("foo3".into(), "bar".into())
                ]),
            })
        );
        rt.verify();
    }
}
