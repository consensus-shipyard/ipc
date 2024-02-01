// Copyright 2024 Textile Inc
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_ipld_hamt::Hamt;
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

pub const BIT_WIDTH: u32 = 8;

// The state represents an object store backed by a Hamt
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    // The root cid of the Hamt
    pub root: Cid,
}

// TODO: (@carsonfarmer) We'll likely want to define the metadata type that will actually be placed in the Hamt

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self> {
        let root = match Hamt::<_, Vec<u8>>::new_with_bit_width(store, BIT_WIDTH).flush() {
            Ok(cid) => cid,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "objectstore actor failed to create empty Hamt: {}",
                    e
                ))
            }
        };

        Ok(Self { root })
    }
}

pub const OBJECTSTORE_ACTOR_NAME: &str = "objectstore";

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct PutObjectParams {
    pub key: Vec<u8>,
    pub content: Vec<u8>,
}

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct DeleteObjectParams {
    pub key: Vec<u8>,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    PutObject = frc42_dispatch::method_hash!("PutObject"),
    AppendObject = frc42_dispatch::method_hash!("AppendObject"),
    GetObject = frc42_dispatch::method_hash!("GetObject"),
    DeleteObject = frc42_dispatch::method_hash!("DeleteObject"),
    ListObjects = frc42_dispatch::method_hash!("ListObjects"),
}
