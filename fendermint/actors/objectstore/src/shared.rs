// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::{Hash, PublicKey};
use fendermint_actor_machine::GET_METADATA_METHOD;
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;
use std::collections::HashMap;

pub use crate::state::State;

pub const OBJECTSTORE_ACTOR_NAME: &str = "objectstore";

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    GetMetadata = GET_METADATA_METHOD,
    AddObject = frc42_dispatch::method_hash!("AddObject"),
    DeleteObject = frc42_dispatch::method_hash!("DeleteObject"),
    GetObject = frc42_dispatch::method_hash!("GetObject"),
    ListObjects = frc42_dispatch::method_hash!("ListObjects"),
}

/// Params for adding an object.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AddParams {
    /// Target object store address.
    pub to: Address,
    /// Source Iroh node ID used for ingestion.
    pub source: PublicKey,
    /// Object key.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    /// Object blake3 hash.
    pub hash: Hash,
    /// Object size.
    pub size: u64,
    /// Object metadata.
    pub metadata: HashMap<String, String>,
    /// Whether to overwrite a key if it already exists.
    pub overwrite: bool,
}

/// Params for deleting an object.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct DeleteParams {
    /// Object key.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
}

/// Params for getting an object.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct GetParams {
    /// Object key.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
}

/// Params for listing objects.
#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ListParams {
    /// The prefix to filter objects by.
    #[serde(with = "strict_bytes")]
    pub prefix: Vec<u8>,
    /// The delimiter used to define object hierarchy.
    #[serde(with = "strict_bytes")]
    pub delimiter: Vec<u8>,
    /// The offset to start listing objects from.
    pub offset: u64,
    /// The maximum number of objects to list.
    pub limit: u64,
}

/// The stored representation of an object in the object store.
#[derive(Clone, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Object {
    /// The object blake3 hash.
    pub hash: Hash,
    /// The object size.
    pub size: u64,
    /// Expiry block.
    pub expiry: ChainEpoch,
    /// User-defined object metadata (e.g., last modified timestamp, etc.).
    pub metadata: HashMap<String, String>,
    /// Whether the object's blob has been resolved.
    pub resolved: bool,
}

/// A list of objects and their common prefixes.
#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ListObjectsReturn {
    /// List of key-values matching the list query.
    pub objects: Vec<(Vec<u8>, Object)>,
    /// When a delimiter is used in the list query, this contains common key prefixes.
    pub common_prefixes: Vec<Vec<u8>>,
}
