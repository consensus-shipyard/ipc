// Copyright 2022-2024 Protocol Labs
// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::{BTreeMap, HashSet};

use fendermint_actor_blobs_shared::state::{Account, PublicKey, SubscriptionId};
use fendermint_actor_blobs_shared::state::{Blob, Hash};
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use hoku_ipld::hamt;
use hoku_ipld::hamt::map::TrackedFlushResult;

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct AccountsState {
    pub root: hamt::Root<Address, Account>,
    size: u64,
}

impl AccountsState {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Address, Account>::new(store, "accounts")?;
        Ok(Self { root, size: 0 })
    }

    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, Address, Account>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<Address, Account>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size
    }

    pub fn len(&self) -> u64 {
        self.size
    }
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct BlobsState {
    pub root: hamt::Root<Hash, Blob>,
    size: u64,
}

impl BlobsState {
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Hash, Blob>::new(store, "blobs")?;
        Ok(Self { root, size: 0 })
    }

    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, Hash, Blob>, ActorError> {
        self.root.hamt(store, self.size)
    }

    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<Hash, Blob>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size
    }

    pub fn len(&self) -> u64 {
        self.size
    }
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct BlobsProgressCollection {
    map: BTreeMap<Hash, BlobsProgressValue>,
    bytes_size: u64,
}

type BlobsProgressValue = HashSet<(Address, SubscriptionId, PublicKey)>;

impl BlobsProgressCollection {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            bytes_size: 0,
        }
    }

    /// Amount of bytes for blobs in the collection
    pub fn bytes_size(&self) -> u64 {
        self.bytes_size
    }

    /// Number of entries
    pub fn len(&self) -> u64 {
        self.map.len() as u64
    }

    /// Add/update added with hash and its source
    pub fn upsert(
        &mut self,
        hash: Hash,
        subscriber: Address,
        id: SubscriptionId,
        source: PublicKey,
        blob_size: u64,
    ) {
        self.map
            .entry(hash)
            .and_modify(|sources| {
                sources.insert((subscriber, id.clone(), source));
            })
            .or_insert_with(|| {
                self.bytes_size += blob_size;
                HashSet::from([(subscriber, id, source)])
            });
    }

    pub fn take_page(&self, size: u32) -> Vec<(Hash, BlobsProgressValue)> {
        self.map
            .iter()
            .take(size as usize)
            .map(|element| (*element.0, element.1.clone()))
            .collect::<Vec<_>>()
    }

    pub fn remove_entry(
        &mut self,
        hash: Hash,
        subscriber: Address,
        sub_id: SubscriptionId,
        source: PublicKey,
        blob_size: u64,
    ) {
        if let Some(entry) = self.map.get_mut(&hash) {
            entry.remove(&(subscriber, sub_id, source));
            self.bytes_size -= blob_size;
            if entry.is_empty() {
                self.map.remove(&hash);
            }
        }
    }

    pub fn insert(
        &mut self,
        hash: Hash,
        value: BlobsProgressValue,
        blob_size: u64,
    ) -> Option<BlobsProgressValue> {
        let result = self.map.insert(hash, value);
        self.bytes_size += blob_size;
        result
    }

    pub fn remove(&mut self, hash: &Hash, blob_size: u64) -> Option<BlobsProgressValue> {
        let result = self.map.remove(hash);
        if result.is_some() {
            self.bytes_size -= blob_size;
        }
        result
    }
}
