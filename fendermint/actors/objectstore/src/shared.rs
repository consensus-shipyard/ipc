// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_ipld_kamt::{id::Identity, Kamt};
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;

// The state stores the blockhashes of the last `lookback_len` epochs
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct State {
    // the KAMT root cid of keys
    pub keys: Cid,
}

impl State {
    pub fn new<BS: Blockstore>(store: &BS) -> anyhow::Result<Self> {
        // Note(sander): I don't know if these generics are correct... maybe K should be Cid?
        let empty_keys_cid = match Kamt::<_, String, Vec<u8>, Identity, 32>::new(store).flush() {
            Ok(cid) => cid,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "objectstore actor failed to create empty Kamt: {}",
                    e
                ))
            }
        };

        Ok(Self {
            keys: empty_keys_cid,
        })
    }
}

pub const OBJECTSTORE_ACTOR_NAME: &str = "objectstore";

#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    pub foo: u64,
}

// Note(sander): Example of method params
// #[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
// pub struct PushBlockParams {
//
// }

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    // PushBlockHash = frc42_dispatch::method_hash!("PushBlockHash"),
    // LookbackLen = frc42_dispatch::method_hash!("LookbackLen"),
    // GetBlockHash = frc42_dispatch::method_hash!("GetBlockHash"),
}
