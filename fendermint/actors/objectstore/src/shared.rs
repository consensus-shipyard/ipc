// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::ext::blobs::Blob;
pub use crate::state::{Object, ObjectList, State};
use cid::Cid;
use fendermint_actor_machine::GET_METADATA_METHOD;
use fvm_ipld_encoding::serde_bytes::ByteBuf;
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_shared::address::Address;
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;
use std::collections::HashMap;

pub const OBJECTSTORE_ACTOR_NAME: &str = "objectstore";

/// Params for adding an object.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AddParams {
    /// Target object store address.
    pub store: Address,
    /// Object key.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    /// Object value.
    pub cid: Cid,
    /// Object size.
    pub size: usize,
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

/// The stored representation of an object in the object store.
#[derive(Clone, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct GotObject {
    pub blob: Blob,
    /// The object content identifier.
    pub cid: ByteBuf,
    /// User-defined object metadata (e.g., last modified timestamp, etc.).
    pub metadata: HashMap<String, String>,
}
