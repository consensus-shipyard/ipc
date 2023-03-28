use std::collections::HashMap;

// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::Context;
use cid::Cid;
use fendermint_vm_genesis::{Actor, ActorMeta};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_ipld_hamt::Hamt;
use fvm_shared::{address::Address, ActorID, HAMT_BIT_WIDTH};

/// Defines first available ID address after builtin actors
pub const FIRST_NON_SINGLETON_ADDR: ActorID = 100;

define_singleton!(INIT { id: 1, code_id: 2 });

pub type AddressMap = HashMap<Address, ActorID>;

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
    /// Create new state instance.
    pub fn new<BS: Blockstore>(
        store: &BS,
        network_name: String,
        accounts: &[Actor],
    ) -> anyhow::Result<(Self, AddressMap)> {
        let mut allocated_ids = AddressMap::new();
        let mut address_map = Hamt::<_, ActorID>::new_with_bit_width(store, HAMT_BIT_WIDTH);

        let addresses = accounts.iter().flat_map(|a| match &a.meta {
            ActorMeta::Account(acc) => {
                vec![acc.owner.0]
            }
            ActorMeta::Multisig(ms) => ms.signers.iter().map(|a| a.0).collect(),
        });

        let mut next_id = FIRST_NON_SINGLETON_ADDR;
        for addr in addresses {
            if allocated_ids.contains_key(&addr) {
                continue;
            }
            allocated_ids.insert(addr, next_id);
            address_map
                .set(addr.to_bytes().into(), next_id)
                .context("cannot set ID of address")?;
            next_id += 1;
        }

        // We will need to allocate an ID for each multisig account, however,
        // these do not have to be recorded in the map, because their addr->ID
        // mapping is trivial (it's an ID type address). To avoid the init actor
        // using the same ID for something else, give it a higher ID to use next.
        for a in accounts.iter() {
            if let ActorMeta::Multisig { .. } = a.meta {
                next_id += 1;
            }
        }

        #[cfg(feature = "m2-native")]
        let installed_actors = store.put_cbor(&Vec::<Cid>::new(), Code::Blake2b256)?;

        let state = Self {
            address_map: address_map.flush()?,
            next_id,
            network_name,
            #[cfg(feature = "m2-native")]
            installed_actors,
        };

        Ok((state, allocated_ids))
    }
}
