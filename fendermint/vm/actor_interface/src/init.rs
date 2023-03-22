// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_hamt::Hamt;
use fvm_shared::{ActorID, HAMT_BIT_WIDTH};

/// Defines first available ID address after builtin actors
pub const FIRST_NON_SINGLETON_ADDR: ActorID = 100;

define_singleton!(INIT { id: 1, code_id: 2 });

#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug)]
pub struct State {
    pub address_map: Cid,
    pub next_id: ActorID,
    pub network_name: String,
    #[cfg(feature = "m2-native")]
    pub installed_actors: Cid,
}

// TODO: Not happy about having to copy this. Maybe we should use project references after all.
impl State {
    /// Create empty state instance.
    pub fn new<BS: Blockstore>(store: &BS, network_name: String) -> anyhow::Result<Self> {
        let empty_map = Hamt::<_, ()>::new_with_bit_width(store, HAMT_BIT_WIDTH).flush()?;

        #[cfg(feature = "m2-native")]
        let installed_actors = store.put_cbor(&Vec::<Cid>::new(), Code::Blake2b256)?;

        Ok(Self {
            address_map: empty_map,
            next_id: FIRST_NON_SINGLETON_ADDR,
            network_name,
            #[cfg(feature = "m2-native")]
            installed_actors,
        })
    }
}
