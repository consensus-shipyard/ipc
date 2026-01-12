// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fendermint_actor_blobs_shared::{
    blobs::{
        AddBlobParams, Blob, BlobStatus, DeleteBlobParams, GetBlobParams, OverwriteBlobParams,
        SubscriptionId,
    },
    sdk::{add_blob, delete_blob, get_blob, has_credit_approval, overwrite_blob},
};
use fendermint_actor_machine::MachineActor;
use fil_actors_runtime::{
    actor_dispatch_unrestricted, actor_error,
    runtime::{ActorCode, Runtime},
    ActorError,
};
use fvm_shared::address::Address;
use ipc_storage_actor_sdk::evm::{
    emit_evm_event, InputData, InvokeContractParams, InvokeContractReturn,
};
use ipc_storage_ipld::hamt::BytesKey;

use crate::shared::{
    AddParams, DeleteParams, GetParams, ListObjectsReturn, ListParams, Method, Object,
    BUCKET_ACTOR_NAME,
};
use crate::sol_facade as sol;
use crate::sol_facade::AbiCall;
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

        let from = rt.message().caller();

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
                    OverwriteBlobParams {
                        old_hash: object.hash,
                        add: AddBlobParams {
                            from,
                            sponsor: Some(state.owner),
                            source: params.source,
                            hash: params.hash,
                            metadata_hash: params.recovery_hash,
                            id: sub_id,
                            size: params.size,
                            ttl: params.ttl,
                        },
                    },
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
                AddBlobParams {
                    from,
                    sponsor: Some(state.owner),
                    source: params.source,
                    hash: params.hash,
                    metadata_hash: params.recovery_hash,
                    id: sub_id,
                    size: params.size,
                    ttl: params.ttl,
                },
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
            sol::ObjectAdded::new(&params.key, &params.hash, &params.metadata),
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

        let from = rt.message().caller();

        let state = rt.state::<State>()?;
        let sub_id = get_blob_id(&state, &params.0)?;
        let key = BytesKey(params.0);
        let object = state
            .get(rt.store(), &key)?
            .ok_or(ActorError::illegal_state("object not found".into()))?;

        // Delete blob for object
        delete_blob(
            rt,
            DeleteBlobParams {
                from,
                sponsor: Some(state.owner),
                hash: object.hash,
                id: sub_id,
            },
        )?;

        rt.transaction(|st: &mut State, rt| st.delete(rt.store(), &key))?;

        emit_evm_event(rt, sol::ObjectDeleted::new(&key, &object.hash))?;

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
            if let Some(blob) = get_blob(rt, GetBlobParams(object_state.hash))? {
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
            |key: Vec<u8>, object_state: ObjectState| -> Result<(), ActorError> {
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

        let from = rt.message().caller();

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

        emit_evm_event(rt, sol::ObjectMetadataUpdated::new(&params.key, &metadata))?;

        Ok(())
    }

    fn invoke_contract(
        rt: &impl Runtime,
        params: InvokeContractParams,
    ) -> Result<InvokeContractReturn, ActorError> {
        let input_data: InputData = params.try_into()?;
        if sol::can_handle(&input_data) {
            let output_data = match sol::parse_input(&input_data)? {
                sol::Calls::addObject_0(call) => {
                    // function addObject(bytes32 source, string memory key, bytes32 hash, bytes32 recoveryHash, uint64 size) external;
                    let params = call.params();
                    Self::add_object(rt, params)?;
                    call.returns(())
                }
                sol::Calls::addObject_1(call) => {
                    // function addObject(AddObjectParams memory params) external;
                    let params = call.params();
                    Self::add_object(rt, params)?;
                    call.returns(())
                }
                sol::Calls::deleteObject(call) => {
                    // function deleteObject(string memory key) external;
                    let params = call.params();
                    Self::delete_object(rt, params)?;
                    call.returns(())
                }
                sol::Calls::getObject(call) => {
                    // function getObject(string memory key) external view returns (ObjectValue memory);
                    let params = call.params();
                    let object = Self::get_object(rt, params)?;
                    call.returns(object)
                }
                sol::Calls::queryObjects_0(call) => {
                    // function queryObjects(string memory prefix, string memory delimiter, string memory startKey, uint64 limit) external view returns (Query memory);
                    let params = call.params();
                    let list = Self::list_objects(rt, params)?;
                    call.returns(list)
                }
                sol::Calls::queryObjects_1(call) => {
                    // function queryObjects(string memory prefix, string memory delimiter, string memory startKey) external view returns (Query memory);
                    let params = call.params();
                    let list = Self::list_objects(rt, params)?;
                    call.returns(list)
                }
                sol::Calls::queryObjects_2(call) => {
                    // function queryObjects(string memory prefix) external view returns (Query memory);
                    let params = call.params();
                    let list = Self::list_objects(rt, params)?;
                    call.returns(list)
                }
                sol::Calls::queryObjects_3(call) => {
                    // function queryObjects() external view returns (Query memory);
                    let params = call.params();
                    let list = Self::list_objects(rt, params)?;
                    call.returns(list)
                }
                sol::Calls::queryObjects_4(call) => {
                    // function queryObjects(string memory prefix, string memory delimiter) external view returns (Query memory);
                    let params = call.params();
                    let list = Self::list_objects(rt, params)?;
                    call.returns(list)
                }
                sol::Calls::updateObjectMetadata(call) => {
                    // function updateObjectMetadata(string memory key, KeyValue[] memory metadata) external;
                    let params = call.params();
                    Self::update_object_metadata(rt, params)?;
                    call.returns(())
                }
            };
            Ok(InvokeContractReturn { output_data })
        } else {
            Err(actor_error!(illegal_argument, "invalid call".to_string()))
        }
    }
}

/// Returns a blob subscription ID specific to this machine and object key.
fn get_blob_id(state: &State, key: &[u8]) -> Result<SubscriptionId, ActorError> {
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
) -> Result<Option<Object>, ActorError> {
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

fn validate_metadata(metadata: &HashMap<String, String>) -> Result<(), ActorError> {
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
) -> Result<(), ActorError> {
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

    actor_dispatch_unrestricted! {
        Constructor => constructor,
        Init => init,
        GetAddress => get_address,
        GetMetadata => get_metadata,
        AddObject => add_object,
        DeleteObject => delete_object,
        GetObject => get_object,
        ListObjects => list_objects,
        UpdateObjectMetadata => update_object_metadata,
        // EVM interop
        InvokeContract => invoke_contract,
        _ => fallback,
    }
}
