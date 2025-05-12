// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;
use std::string::ToString;

use anyhow::Error;
use fendermint_actor_blobs_shared::bytes::B256;
use fil_actors_runtime::{actor_error, ActorError};
use fvm_shared::clock::ChainEpoch;
use num_traits::Zero;
use recall_actor_sdk::{declare_abi_call, evm::TryIntoEVMEvent};
pub use recall_sol_facade::bucket::Calls;
use recall_sol_facade::{
    bucket as sol,
    types::{SolCall, SolInterface},
};

use crate::{
    AddParams, DeleteParams, GetParams, ListObjectsReturn, ListParams, Object,
    UpdateObjectMetadataParams,
};

declare_abi_call!();

// ----- Events ----- //

pub struct ObjectAdded<'a> {
    pub key: &'a Vec<u8>,
    pub blob_hash: &'a B256,
    pub metadata: &'a HashMap<String, String>,
}
impl<'a> ObjectAdded<'a> {
    pub fn new(
        key: &'a Vec<u8>,
        blob_hash: &'a B256,
        metadata: &'a HashMap<String, String>,
    ) -> Self {
        Self {
            key,
            blob_hash,
            metadata,
        }
    }
}
impl TryIntoEVMEvent for ObjectAdded<'_> {
    type Target = sol::Events;

    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        let metadata = fvm_ipld_encoding::to_vec(self.metadata)?;
        Ok(sol::Events::ObjectAdded(sol::ObjectAdded {
            key: self.key.clone().into(),
            blobHash: self.blob_hash.0.into(),
            metadata: metadata.into(),
        }))
    }
}

pub struct ObjectMetadataUpdated<'a> {
    pub key: &'a Vec<u8>,
    pub metadata: &'a HashMap<String, String>,
}
impl<'a> ObjectMetadataUpdated<'a> {
    pub fn new(key: &'a Vec<u8>, metadata: &'a HashMap<String, String>) -> Self {
        Self { key, metadata }
    }
}
impl<'a> TryIntoEVMEvent for ObjectMetadataUpdated<'a> {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        let metadata = fvm_ipld_encoding::to_vec(self.metadata)?;
        Ok(sol::Events::ObjectMetadataUpdated(
            sol::ObjectMetadataUpdated {
                key: self.key.clone().into(),
                metadata: metadata.into(),
            },
        ))
    }
}

pub struct ObjectDeleted<'a> {
    pub key: &'a Vec<u8>,
    pub blob_hash: &'a B256,
}
impl<'a> ObjectDeleted<'a> {
    pub fn new(key: &'a Vec<u8>, blob_hash: &'a B256) -> Self {
        Self { key, blob_hash }
    }
}
impl TryIntoEVMEvent for ObjectDeleted<'_> {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<Self::Target, Error> {
        Ok(sol::Events::ObjectDeleted(sol::ObjectDeleted {
            key: self.key.clone().into(),
            blobHash: self.blob_hash.0.into(),
        }))
    }
}

// ----- Calls ----- //

pub fn can_handle(input_data: &recall_actor_sdk::evm::InputData) -> bool {
    Calls::valid_selector(input_data.selector())
}

pub fn parse_input(input: &recall_actor_sdk::evm::InputData) -> Result<Calls, ActorError> {
    Calls::abi_decode_raw(input.selector(), input.calldata(), true)
        .map_err(|e| actor_error!(illegal_argument, format!("invalid call: {}", e)))
}

impl AbiCall for sol::addObject_0Call {
    type Params = AddParams;
    type Returns = ();
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let source = B256(self.source.into());
        let key: Vec<u8> = self.key.clone().into_bytes();
        let hash = B256(self.hash.into());
        let recovery_hash = B256(self.recoveryHash.into());
        let size = self.size;
        AddParams {
            source,
            key,
            hash,
            recovery_hash,
            size,
            ttl: None,
            metadata: HashMap::default(),
            overwrite: false,
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

impl AbiCall for sol::addObject_1Call {
    type Params = AddParams;
    type Returns = ();
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {
        let source = B256(self.source.into());
        let key: Vec<u8> = self.key.clone().into_bytes();
        let hash = B256(self.hash.into());
        let recovery_hash = B256(self.recoveryHash.into());
        let size = self.size;
        let ttl = if self.ttl.clone().is_zero() {
            None
        } else {
            Some(self.ttl as ChainEpoch)
        };
        let mut metadata: HashMap<String, String> = HashMap::with_capacity(self.metadata.len());
        for kv in self.metadata.iter().cloned() {
            metadata.insert(kv.key, kv.value);
        }
        let overwrite = self.overwrite;
        AddParams {
            source,
            key,
            hash,
            recovery_hash,
            size,
            ttl,
            metadata,
            overwrite,
        }
    }
    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

impl AbiCall for sol::deleteObjectCall {
    type Params = DeleteParams;
    type Returns = ();
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let key: Vec<u8> = self.key.clone().into_bytes();
        DeleteParams(key)
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

impl AbiCall for sol::getObjectCall {
    type Params = GetParams;
    type Returns = Option<Object>;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let key = self.key.clone().into_bytes();
        GetParams(key)
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let object = returns
            .map(|object| sol::ObjectValue {
                blobHash: object.hash.0.into(),
                recoveryHash: object.recovery_hash.0.into(),
                size: object.size,
                expiry: object.expiry as u64,
                metadata: sol_metadata(object.metadata),
            })
            .unwrap_or(sol::ObjectValue {
                blobHash: [0u8; 32].into(),
                recoveryHash: [0u8; 32].into(),
                size: 0,
                expiry: 0,
                metadata: vec![],
            });
        Self::abi_encode_returns(&(object,))
    }
}

fn sol_metadata(metadata: HashMap<String, String>) -> Vec<sol::KeyValue> {
    metadata
        .iter()
        .map(|(k, v)| sol::KeyValue {
            key: k.clone(),
            value: v.clone(),
        })
        .collect()
}

fn sol_query(list: ListObjectsReturn) -> sol::Query {
    sol::Query {
        objects: list
            .objects
            .iter()
            .map(|(key, object_state)| sol::Object {
                key: String::from_utf8_lossy(key.as_slice()).to_string(),
                state: sol::ObjectState {
                    blobHash: object_state.hash.0.into(),
                    size: object_state.size,
                    expiry: object_state.expiry as u64,
                    metadata: sol_metadata(object_state.metadata.clone()),
                },
            })
            .collect(),
        commonPrefixes: list
            .common_prefixes
            .iter()
            .map(|prefix| String::from_utf8_lossy(prefix.as_slice()).to_string())
            .collect(),
        nextKey: list
            .next_key
            .map(|k| String::from_utf8_lossy(k.as_slice()).to_string())
            .unwrap_or_default(),
    }
}

const DEFAULT_DELIMITER: &[u8] = b"/"; // "/" in ASCII and UTF-8
const DEFAULT_START_KEY: Option<Vec<u8>> = None; //= ""
const DEFAULT_PREFIX: Vec<u8> = vec![]; //= ""
const DEFAULT_LIMIT: u64 = 0;

impl AbiCall for sol::queryObjects_0Call {
    type Params = ListParams;
    type Returns = ListObjectsReturn;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let prefix = self.prefix.clone().into_bytes();
        let delimiter = self.delimiter.clone().into_bytes();
        let start_key = if self.startKey.is_empty() {
            None
        } else {
            Some(self.startKey.clone().into_bytes())
        };
        let limit = self.limit;
        ListParams {
            prefix,
            delimiter,
            start_key,
            limit,
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let query = sol_query(returns);
        Self::abi_encode_returns(&(query,))
    }
}

impl AbiCall for sol::queryObjects_1Call {
    type Params = ListParams;
    type Returns = ListObjectsReturn;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let prefix = self.prefix.clone().into_bytes();
        let delimiter = self.delimiter.clone().into_bytes();
        let start_key = if self.startKey.is_empty() {
            None
        } else {
            Some(self.startKey.clone().into_bytes())
        };
        let limit = DEFAULT_LIMIT;
        ListParams {
            prefix,
            delimiter,
            start_key,
            limit,
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let query = sol_query(returns);
        Self::abi_encode_returns(&(query,))
    }
}

impl AbiCall for sol::queryObjects_2Call {
    type Params = ListParams;
    type Returns = ListObjectsReturn;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let prefix = self.prefix.clone().into_bytes();
        let delimiter = DEFAULT_DELIMITER.to_vec();
        let start_key = DEFAULT_START_KEY;
        let limit = DEFAULT_LIMIT;
        ListParams {
            prefix,
            delimiter,
            start_key,
            limit,
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let query = sol_query(returns);
        Self::abi_encode_returns(&(query,))
    }
}

impl AbiCall for sol::queryObjects_3Call {
    type Params = ListParams;
    type Returns = ListObjectsReturn;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let prefix = DEFAULT_PREFIX;
        let delimiter = DEFAULT_DELIMITER.to_vec();
        let start_key = DEFAULT_START_KEY;
        let limit = 0;
        ListParams {
            prefix,
            delimiter,
            start_key,
            limit,
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let query = sol_query(returns);
        Self::abi_encode_returns(&(query,))
    }
}

impl AbiCall for sol::queryObjects_4Call {
    type Params = ListParams;
    type Returns = ListObjectsReturn;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let prefix = self.prefix.clone().into_bytes();
        let delimiter = self.delimiter.clone().into_bytes();
        let start_key = DEFAULT_START_KEY;
        let limit = DEFAULT_LIMIT;
        ListParams {
            prefix,
            delimiter,
            start_key,
            limit,
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let query = sol_query(returns);
        Self::abi_encode_returns(&(query,))
    }
}

impl AbiCall for sol::updateObjectMetadataCall {
    type Params = UpdateObjectMetadataParams;
    type Returns = ();
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let mut metadata: HashMap<String, Option<String>> = HashMap::default();
        for kv in self.metadata.iter().cloned() {
            let key = kv.key;
            let value = kv.value;
            let value = if value.is_empty() { None } else { Some(value) };
            metadata.insert(key, value);
        }
        UpdateObjectMetadataParams {
            key: self.key.clone().into_bytes(),
            metadata,
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}
