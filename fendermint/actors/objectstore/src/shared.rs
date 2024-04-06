// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use crate::state::{Object, ObjectKind, ObjectList, ObjectListItem, State};

pub const OBJECTSTORE_ACTOR_NAME: &str = "objectstore";

/// Params for putting an object.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectPutParams {
    /// Object key.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    /// Kind of object.
    pub kind: ObjectKind,
    /// Whether to overwrite a key if it already exists.
    pub overwrite: bool,
}

/// Params for resolving an external object.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectResolveExternalParams {
    /// Object key.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    /// External object value.
    pub value: Cid,
}

/// Params for deleting an object.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectDeleteParams {
    /// Object key.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
}

/// Params for getting an object.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectGetParams {
    /// Object key.
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
}

/// Params for listing objects.
#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectListParams {
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
    PutObject = frc42_dispatch::method_hash!("PutObject"),
    ResolveExternalObject = frc42_dispatch::method_hash!("ResolveExternalObject"),
    DeleteObject = frc42_dispatch::method_hash!("DeleteObject"),
    GetObject = frc42_dispatch::method_hash!("GetObject"),
    ListObjects = frc42_dispatch::method_hash!("ListObjects"),
}
