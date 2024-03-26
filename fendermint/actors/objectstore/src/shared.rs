// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub use crate::state::*;

pub const OBJECTSTORE_ACTOR_NAME: &str = "objectstore";

#[derive(Clone, Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ObjectParams {
    #[serde(with = "strict_bytes")]
    pub key: Vec<u8>,
    pub value: Cid,
}

/// Options for listing objects.
#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ListOptions {
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
    ResolveObject = frc42_dispatch::method_hash!("ResolveObject"),
    DeleteObject = frc42_dispatch::method_hash!("DeleteObject"),
    GetObject = frc42_dispatch::method_hash!("GetObject"),
    ListObjects = frc42_dispatch::method_hash!("ListObjects"),
}
