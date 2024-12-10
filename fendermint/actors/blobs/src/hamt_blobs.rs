// Copyright 2024 Hoku Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fendermint_actor_blobs_shared::state::{Blob, Hash};
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use hoku_ipld::map::{Map, DEFAULT_HAMT_CONFIG};

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct HamtBlobsRoot {
    cid: Cid,
}

impl HamtBlobsRoot {
    pub fn load<BS: Blockstore>(&self, store: BS) -> Result<HamtBlobs<BS>, ActorError> {
        HamtBlobs::load(store, &self.cid)
    }

    pub fn flush_empty<'a, BS: Blockstore>(store: BS) -> Result<Self, ActorError> {
        let cid = Map::<BS, Hash, Blob>::flush_empty(store, DEFAULT_HAMT_CONFIG)?;
        Ok(Self { cid })
    }
}

pub struct HamtBlobs<BS: Blockstore> {
    pub map: Map<BS, Hash, Blob>
}

impl<BS> HamtBlobs<BS>
where
    BS: Blockstore,
{
    pub fn flush_empty(store: BS) -> Result<Cid, ActorError> {
        Map::<BS, Hash, Blob>::flush_empty(store, DEFAULT_HAMT_CONFIG)
    }

    pub fn load(store: BS, root: &Cid) -> Result<Self, ActorError> {
        let map = Map::<BS, Hash, Blob>::load(store, root, DEFAULT_HAMT_CONFIG, "blobs")?;
        Ok(Self { map })
    }

    pub fn get(&self, key: &Hash) -> Result<Option<Blob>, ActorError> {
        self.map.get(key).map(|a| a.cloned())
    }

    pub fn get_or_err(&self, key: &Hash) -> Result<Blob, ActorError> {
        self.get(key)?
            .ok_or(ActorError::not_found(format!("entry {} not found", key)))
    }

    pub fn set_and_flush(&mut self, key: &Hash, value: Blob) -> Result<HamtBlobsRoot, ActorError> {
        self.map.set(key, value)?;
        let cid = self.map.flush()?;
        Ok(HamtBlobsRoot { cid })
    }

    pub fn delete_and_flush(&mut self, key: &Hash) -> Result<HamtBlobsRoot, ActorError> {
        self.map.delete(key)?;
        let cid = self.map.flush()?;
        Ok(HamtBlobsRoot { cid })
    }

    /// Consumes the underlying map's HAMT and returns the Blockstore it owns.
    pub fn into_store(self) -> BS {
        self.map.into_store()
    }
}
