// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fendermint_actor_blobs_shared::state::{Hash, PublicKey};
use fendermint_actor_machine::{
    GET_ADDRESS_METHOD, GET_METADATA_METHOD, INIT_METHOD, METHOD_CONSTRUCTOR,
};
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

pub use crate::state::{ObjectState, State};

pub const BUCKET_ACTOR_NAME: &str = "bucket";
pub const MAX_METADATA_ENTRIES: u32 = 20;
pub const MAX_METADATA_KEY_SIZE: u32 = 32;
pub const MAX_METADATA_VALUE_SIZE: u32 = 128;
#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    Init = INIT_METHOD,
    GetAddress = GET_ADDRESS_METHOD,
    GetMetadata = GET_METADATA_METHOD,
    AddObject = frc42_dispatch::method_hash!("AddObject"),
    DeleteObject = frc42_dispatch::method_hash!("DeleteObject"),
    GetObject = frc42_dispatch::method_hash!("GetObject"),
    ListObjects = frc42_dispatch::method_hash!("ListObjects"),
    UpdateObjectMetadata = frc42_dispatch::method_hash!("UpdateObjectMetadata"),
}

/// Params for adding an object.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AddParams {
    /// Source Iroh node ID used for ingestion.
    pub source: PublicKey,
    /// Object key.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    /// Object blake3 hash.
    pub hash: Hash,
    /// Blake3 hash of the metadata to use for object recovery.
    pub recovery_hash: Hash,
    /// Object size.
    pub size: u64,
    /// Object time-to-live epochs.
    /// If not specified, the current default TTL from the config actor is used.
    pub ttl: Option<ChainEpoch>,
    /// Object metadata.
    pub metadata: HashMap<String, String>,
    /// Whether to overwrite a key if it already exists.
    pub overwrite: bool,
    /// Account address that initiated the call
    pub from: Address,
}

/// Params for deleting an object.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct DeleteParams {
    /// Key of the object to delete from a bucket.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    /// Account address that initiated the call
    pub from: Address,
}

/// Params for getting an object.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GetParams(#[serde(with = "strict_bytes")] pub Vec<u8>);

/// Params for listing objects.
#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ListParams {
    /// The prefix to filter objects by.
    #[serde(with = "strict_bytes")]
    pub prefix: Vec<u8>,
    /// The delimiter used to define object hierarchy.
    #[serde(with = "strict_bytes")]
    pub delimiter: Vec<u8>,
    /// The key to start listing objects from.
    pub start_key: Option<Vec<u8>>,
    /// The maximum number of objects to list.
    pub limit: u64,
}

/// The stored representation of an object in the bucket.
#[derive(Clone, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Object {
    /// The object blake3 hash.
    pub hash: Hash,
    /// Blake3 hash of the metadata to use for object recovery.
    pub recovery_hash: Hash,
    /// The object size.
    pub size: u64,
    /// Expiry block.
    pub expiry: ChainEpoch,
    /// User-defined object metadata (e.g., last modified timestamp, etc.).
    pub metadata: HashMap<String, String>,
}

/// A list of objects and their common prefixes.
#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ListObjectsReturn {
    /// List of key-values matching the list query.
    pub objects: Vec<(Vec<u8>, ObjectState)>,
    /// When a delimiter is used in the list query, this contains common key prefixes.
    pub common_prefixes: Vec<Vec<u8>>,
    /// Next key to use for paginating when there are more objects to list.
    pub next_key: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct UpdateObjectMetadataParams {
    /// Object key.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    /// Object metadata to be inserted/updated/deleted.
    ///
    /// If a key-value is present, we'll update the entry (or insert if it does not exist)
    /// If only the key is present, we will delete the metadata entry
    pub metadata: HashMap<String, Option<String>>,
    /// Account address that initiated the call
    pub from: Address,
}
