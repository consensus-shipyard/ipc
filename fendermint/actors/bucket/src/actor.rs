// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fendermint_actor_blobs_shared::{
    add_blob, delete_blob, get_blob, has_credit_approval, overwrite_blob,
    state::{BlobInfo, BlobStatus, SubscriptionId},
};
use fendermint_actor_machine::{
    events::emit_evm_event,
    util::{require_addr_is_origin_or_caller, to_id_address},
    MachineActor,
};
use fil_actors_runtime::{
    actor_dispatch, actor_error,
    runtime::{ActorCode, Runtime},
    ActorError,
};
use fvm_ipld_hamt::BytesKey;
use fvm_shared::address::Address;
use recall_sol_facade::bucket::{object_added, object_deleted, object_metadata_updated};

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

pub struct Actor;

impl Actor {
    /// Adds an object to a bucket.
    ///
    /// Access control will be enforced by the Blobs actor.
    /// We will pass the bucket owner as the `subscriber`,
    /// and the Blobs actor will enforce that the `from` address is either
    /// the `subscriber` or has a valid credit delegation from the `subscriber`.
    /// The `from` address must be the origin or the caller.
    fn add_object(rt: &impl Runtime, params: AddParams) -> Result<Object, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let from = to_id_address(rt, params.from, false)?;
        require_addr_is_origin_or_caller(rt, from)?;

        let state = rt.state::<State>()?;
        let sub_id = get_blob_id(&state, &params.key)?;
        let key = BytesKey(params.key.clone());

        validate_metadata(&params.metadata)?;

        let sub = if let Some(object) = state.get(rt.store(), &key)? {
            // If we have existing blob and it's not expired
            let expired = object.expiry <= rt.curr_epoch();
            if params.overwrite || expired {
                // Overwrite if the flag is passed
                overwrite_blob(
                    rt,
                    from,
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
                params.from,
                sub_id,
                params.hash,
                Some(state.owner),
                params.source,
                params.recovery_hash,
                params.size,
                params.ttl,
            )?
        };

        rt.transaction(|st: &mut State, rt| {
            st.add(
                rt.store(),
                key,
                params.hash,
                params.size,
                sub.expiry,
                params.metadata.clone(),
                params.overwrite,
            )
        })?;

        emit_evm_event(
            rt,
            object_added(params.key, &params.hash.0, &params.metadata),
        )?;

        Ok(Object {
            hash: params.hash,
            recovery_hash: params.recovery_hash,
            size: params.size,
            expiry: sub.expiry,
            metadata: params.metadata,
        })
    }

    /// Deletes an object from a bucket.
    ///
    /// Access control will be enforced by the Blobs actor.
    /// We will pass the bucket owner as the `subscriber`,
    /// and the Blobs actor will enforce that the `from` address is either
    /// the `subscriber` or has a valid credit delegation from the `subscriber`.
    /// The `from` address must be the origin or the caller.
    fn delete_object(rt: &impl Runtime, params: DeleteParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let from = to_id_address(rt, params.from, false)?;
        require_addr_is_origin_or_caller(rt, from)?;

        let state = rt.state::<State>()?;
        let sub_id = get_blob_id(&state, &params.key)?;
        let key = BytesKey(params.key);
        let object = state
            .get(rt.store(), &key)?
            .ok_or(ActorError::illegal_state("object not found".into()))?;

        // Delete blob for object
        delete_blob(rt, from, sub_id, object.hash, Some(state.owner))?;

        rt.transaction(|st: &mut State, rt| st.delete(rt.store(), &key))?;

        emit_evm_event(rt, object_deleted(key.0, &object.hash.0))?;

        Ok(())
    }

    /// Returns an object.
    fn get_object(rt: &impl Runtime, params: GetParams) -> Result<Option<Object>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let state = rt.state::<State>()?;
        let owner = state.owner;
        let sub_id = get_blob_id(&state, &params.0)?;
        let key = BytesKey(params.0);
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

    /// Lists bucket objects.
    fn list_objects(
        rt: &impl Runtime,
        params: ListParams,
    ) -> Result<ListObjectsReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let current_epoch = rt.curr_epoch();
        let mut objects = Vec::new();
        let start_key = params.start_key.map(BytesKey::from);
        let state = rt.state::<State>()?;
        let (prefixes, next_key) = state.list(
            rt.store(),
            params.prefix,
            params.delimiter,
            start_key.as_ref(),
            params.limit,
            |key: Vec<u8>, object_state: ObjectState| -> anyhow::Result<(), ActorError> {
                if object_state.expiry > current_epoch {
                    objects.push((key, object_state));
                }
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

    /// Updates object metadata.
    ///
    /// Only the bucket owner or an account with a credit delegation
    /// from the bucket owner can update object metadata.
    /// The `from` address must be the origin or the caller.
    fn update_object_metadata(
        rt: &impl Runtime,
        params: UpdateObjectMetadataParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let from = to_id_address(rt, params.from, false)?;
        require_addr_is_origin_or_caller(rt, from)?;

        let key = BytesKey(params.key.clone());
        let state = rt.state::<State>()?;
        let mut object = state
            .get(rt.store(), &key)?
            .ok_or(ActorError::illegal_state("object not found".into()))?;

        let bucket_owner = state.owner;
        if !has_credit_approval(rt, bucket_owner, from)? {
            return Err(actor_error!(
                forbidden;
                format!("Unauthorized: missing delegation from bucket owner {} to {}", bucket_owner, from)));
        }

        validate_metadata_optional(&params.metadata)?;

        let metadata = rt.transaction(|st: &mut State, rt| {
            for (key, val) in params.metadata {
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
                object.expiry,
                object.metadata.clone(),
                true,
            )?;

            Ok(object.metadata)
        })?;

        emit_evm_event(rt, object_metadata_updated(params.key, &metadata))?;

        Ok(())
    }
}

/// Returns a blob subscription ID specific to this machine and object key.
fn get_blob_id(state: &State, key: &[u8]) -> anyhow::Result<SubscriptionId, ActorError> {
    let mut data = state.address.get()?.payload_bytes();
    data.extend(key);
    let id = blake3::hash(&data).to_hex().to_string();
    SubscriptionId::new(&id)
}

/// Build an object from its state and blob.
fn build_object(
    blob: &BlobInfo,
    object_state: &ObjectState,
    sub_id: SubscriptionId,
    subscriber: Address,
) -> anyhow::Result<Option<Object>, ActorError> {
    match blob.status {
        BlobStatus::Resolved => {
            blob.subscribers.get(&sub_id).cloned().ok_or_else(|| {
                ActorError::illegal_state(format!(
                    "owner {} is not subscribed to blob {}; this should not happen",
                    subscriber, object_state.hash
                ))
            })?;
            Ok(Some(Object {
                hash: object_state.hash,
                recovery_hash: blob.metadata_hash,
                size: blob.size,
                expiry: object_state.expiry,
                metadata: object_state.metadata.clone(),
            }))
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

    use fendermint_actor_blobs_shared::{
        params::{
            AddBlobParams, DeleteBlobParams, GetBlobParams, GetCreditApprovalParams,
            OverwriteBlobParams,
        },
        state::{CreditApproval, Hash, Subscription},
        Method as BlobMethod, BLOBS_ACTOR_ADDR,
    };
    use fendermint_actor_blobs_testing::{new_hash, new_pk, setup_logs};
    use fendermint_actor_machine::{events::to_actor_event, ConstructorParams, InitParams, Kind};
    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::runtime::Runtime;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ADM_ACTOR_CODE_ID, ETHACCOUNT_ACTOR_CODE_ID, INIT_ACTOR_CODE_ID,
    };
    use fil_actors_runtime::{ADM_ACTOR_ADDR, INIT_ACTOR_ADDR};
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::{
        clock::ChainEpoch, econ::TokenAmount, error::ExitCode, sys::SendFlags, MethodNum,
    };
    use recall_sol_facade::machine::{machine_created, machine_initialized};

    fn get_runtime() -> (MockRuntime, Address) {
        let origin_id_addr = Address::new_id(110);
        let rt = construct_and_verify(origin_id_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, origin_id_addr);
        rt.set_origin(origin_id_addr);
        (rt, origin_id_addr)
    }

    fn construct_and_verify(owner_id_addr: Address) -> MockRuntime {
        let owner_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let owner_delegated_addr = Address::new_delegated(10, &owner_eth_addr.0).unwrap();

        let buck_addr = Address::new_id(111);
        let rt = MockRuntime {
            receiver: buck_addr,
            ..Default::default()
        };
        rt.set_delegated_address(owner_id_addr.id().unwrap(), owner_delegated_addr);

        rt.set_caller(*INIT_ACTOR_CODE_ID, INIT_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![INIT_ACTOR_ADDR]);
        let metadata = HashMap::new();
        let event = to_actor_event(
            machine_created(Kind::Bucket as u8, owner_delegated_addr, &metadata).unwrap(),
        )
        .unwrap();
        rt.expect_emitted_event(event);
        let actor_construction = rt
            .call::<Actor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&ConstructorParams {
                    owner: owner_id_addr,
                    metadata,
                })
                .unwrap(),
            )
            .unwrap();
        expect_empty(actor_construction);
        rt.verify();

        rt.set_caller(*ADM_ACTOR_CODE_ID, ADM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![ADM_ACTOR_ADDR]);
        let event =
            to_actor_event(machine_initialized(Kind::Bucket as u8, buck_addr).unwrap()).unwrap();
        rt.expect_emitted_event(event);
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

    fn expect_emitted_add_event(rt: &MockRuntime, params: &AddParams) {
        let event = to_actor_event(
            object_added(params.key.clone(), &params.hash.0, &params.metadata).unwrap(),
        )
        .unwrap();
        rt.expect_emitted_event(event);
    }

    fn expect_emitted_delete_event(rt: &MockRuntime, params: &DeleteParams, hash: Hash) {
        let event = to_actor_event(object_deleted(params.key.clone(), &hash.0).unwrap()).unwrap();
        rt.expect_emitted_event(event);
    }

    #[test]
    pub fn test_add_object() {
        setup_logs();
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
            from: origin,
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let state = rt.state::<State>().unwrap();
        let sub_id = get_blob_id(&state, &key).unwrap();
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
                from: origin,
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
            from: origin,
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let state = rt.state::<State>().unwrap();
        let sub_id = get_blob_id(&state, &key).unwrap();
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
                from: origin,
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
            from: origin,
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
                    from: origin,
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
            from: origin,
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let state = rt.state::<State>().unwrap();
        let sub_id = get_blob_id(&state, &key).unwrap();
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
                from: origin,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription {
                added: 0,
                expiry: ChainEpoch::from(3600),
                source: add_params.source,
                delegate: None,
                failed: false,
            })
            .unwrap(),
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
            from: origin,
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
            from: origin,
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let state = rt.state::<State>().unwrap();
        let sub_id = get_blob_id(&state, &key).unwrap();
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
                from: origin,
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
        let delete_params = DeleteParams { key, from: origin };
        rt.expect_validate_caller_any();
        rt.expect_send_simple(
            BLOBS_ACTOR_ADDR,
            BlobMethod::DeleteBlob as MethodNum,
            IpldBlock::serialize_cbor(&DeleteBlobParams {
                sponsor: Some(origin),
                hash: add_params.hash,
                id: sub_id,
                from: origin,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            None,
            ExitCode::OK,
        );
        expect_emitted_delete_event(&rt, &delete_params, add_params.hash);
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
            from: origin,
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let state = rt.state::<State>().unwrap();
        let sub_id = get_blob_id(&state, &key).unwrap();
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
                from: origin,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription {
                added: 0,
                expiry: ttl,
                source: add_params.source,
                delegate: None,
                failed: false,
            })
            .unwrap(),
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
        let blob = BlobInfo {
            size: add_params.size,
            subscribers: HashMap::from([(sub_id, ttl)]),
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
        let ttl = ChainEpoch::from(3600);
        let add_params: AddParams = AddParams {
            source: new_pk(),
            key: key.clone(),
            hash: hash.0,
            size: hash.1,
            recovery_hash: new_hash(256).0,
            ttl: Some(ttl),
            metadata: HashMap::from([("foo".into(), "bar".into()), ("foo2".into(), "bar".into())]),
            from: origin,
            overwrite: false,
        };
        rt.expect_validate_caller_any();
        let state = rt.state::<State>().unwrap();
        let sub_id = get_blob_id(&state, &key).unwrap();
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
                from: origin,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            IpldBlock::serialize_cbor(&Subscription {
                added: 0,
                expiry: ttl,
                source: add_params.source,
                delegate: None,
                failed: false,
            })
            .unwrap(),
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

        // Update metadata
        let update_object_params = UpdateObjectMetadataParams {
            from: origin,
            key: add_params.key.clone(),
            metadata: HashMap::from([
                ("foo".into(), Some("zar".into())),
                ("foo2".into(), None),
                ("foo3".into(), Some("bar".into())),
            ]),
        };
        rt.expect_validate_caller_any();
        let event = to_actor_event(
            object_metadata_updated(
                add_params.key,
                &HashMap::from([("foo".into(), "zar".into()), ("foo3".into(), "bar".into())]),
            )
            .unwrap(),
        )
        .unwrap();
        rt.expect_emitted_event(event);
        let result = rt.call::<Actor>(
            Method::UpdateObjectMetadata as u64,
            IpldBlock::serialize_cbor(&update_object_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Fail if "from" is neither origin nor caller
        let alien_id_addr = Address::new_id(112);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, alien_id_addr);
        rt.set_origin(alien_id_addr);
        rt.expect_validate_caller_any();
        let result = rt.call::<Actor>(
            Method::UpdateObjectMetadata as u64,
            IpldBlock::serialize_cbor(&update_object_params).unwrap(),
        );
        assert!(result.is_err());
        rt.verify();

        // Fail if "from" is not the owner, and has no delegation.
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, alien_id_addr);
        rt.set_origin(alien_id_addr);
        rt.expect_validate_caller_any();
        let alien_update = UpdateObjectMetadataParams {
            from: alien_id_addr,
            key: update_object_params.key,
            metadata: update_object_params.metadata,
        };
        rt.expect_send(
            BLOBS_ACTOR_ADDR,
            BlobMethod::GetCreditApproval as MethodNum,
            IpldBlock::serialize_cbor(&GetCreditApprovalParams {
                from: origin,
                to: alien_id_addr,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            None,
            SendFlags::READ_ONLY,
            IpldBlock::serialize_cbor::<Option<CreditApproval>>(&None).unwrap(),
            ExitCode::OK,
            None,
        );
        let result = rt.call::<Actor>(
            Method::UpdateObjectMetadata as u64,
            IpldBlock::serialize_cbor(&alien_update).unwrap(),
        );
        assert!(result.is_err());
        rt.verify();

        // Allowed if there is a delegation
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, alien_id_addr);
        rt.set_origin(alien_id_addr);
        rt.expect_validate_caller_any();
        rt.expect_send(
            BLOBS_ACTOR_ADDR,
            BlobMethod::GetCreditApproval as MethodNum,
            IpldBlock::serialize_cbor(&GetCreditApprovalParams {
                from: origin,
                to: alien_id_addr,
            })
            .unwrap(),
            TokenAmount::from_whole(0),
            None,
            SendFlags::READ_ONLY,
            // We do not care what is inside credit approval. We only care if it is present.
            IpldBlock::serialize_cbor::<Option<CreditApproval>>(&Some(CreditApproval {
                credit_limit: None,
                gas_fee_limit: None,
                expiry: None,
                credit_used: TokenAmount::from_whole(0),
                gas_fee_used: TokenAmount::from_whole(0),
            }))
            .unwrap(),
            ExitCode::OK,
            None,
        );
        let event = to_actor_event(
            object_metadata_updated(
                alien_update.key.clone(),
                &HashMap::from([("foo".into(), "zar".into()), ("foo3".into(), "bar".into())]),
            )
            .unwrap(),
        )
        .unwrap();
        rt.expect_emitted_event(event);
        let result = rt.call::<Actor>(
            Method::UpdateObjectMetadata as u64,
            IpldBlock::serialize_cbor(&alien_update).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Get the object and check metadata
        let blob = BlobInfo {
            size: add_params.size,
            subscribers: HashMap::from([(sub_id, ttl)]),
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
