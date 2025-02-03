// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashSet;

use fendermint_actor_blobs_shared::state::{Blob, Hash};
use fendermint_actor_blobs_shared::state::{PublicKey, SubscriptionId};
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use recall_ipld::hamt;
use recall_ipld::hamt::map::TrackedFlushResult;

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
        self.size = tracked_flush_result.size;
    }

    pub fn len(&self) -> u64 {
        self.size
    }
}

#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct BlobsProgressCollection {
    pub root: hamt::Root<Hash, BlobSourceSet>,
    /// Number of blobs in the collection.
    /// A blob with multiple sources is only counted once.
    size: u64,
    /// Number of blob bytes in the collection.
    /// A blob with multiple sources is only counted once.
    bytes_size: u64,
}

/// Blob source is a tuple of subscriber [`Address`], blob [`SubscriptionId`],
/// and an Iroh node [`PublicKey`].
type BlobSource = (Address, SubscriptionId, PublicKey);

/// A set of [`BlobSource`]s.
/// A blob in the collection may have multiple sources.
type BlobSourceSet = HashSet<(Address, SubscriptionId, PublicKey)>;

impl BlobsProgressCollection {
    /// Returns a new progress collection.
    pub fn new<BS: Blockstore>(store: &BS, name: &str) -> Result<Self, ActorError> {
        let root = hamt::Root::<Hash, BlobSourceSet>::new(store, name)?;
        Ok(Self {
            root,
            size: 0,
            bytes_size: 0,
        })
    }

    /// Returns the underlying [`hamt::map::Hamt`].
    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, Hash, BlobSourceSet>, ActorError> {
        self.root.hamt(store, self.size)
    }

    /// Saves the state from the [`TrackedFlushResult`].
    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<Hash, BlobSourceSet>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }

    /// Number of blobs in the collection.
    /// A blob with multiple sources is only counted once.
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Returns the number of blob bytes in the collection.
    /// A blob with multiple sources is only counted once.
    pub fn bytes_size(&self) -> u64 {
        self.bytes_size
    }

    /// Adds/updates an entry in the collection.
    pub fn upsert<BS: Blockstore>(
        &mut self,
        store: BS,
        hash: Hash,
        source: BlobSource,
        blob_size: u64,
    ) -> Result<(), ActorError> {
        let mut map = self.hamt(store)?;
        if !map.set_if_absent(&hash, HashSet::from([source.clone()]))? {
            // Modify existing entry
            let mut entry = map.get(&hash)?.expect("entry should exist");
            entry.insert(source);
            map.set(&hash, entry)?;
        } else {
            // Entry did not exist, add to tracked bytes size
            self.bytes_size += blob_size;
        }
        self.save_tracked(map.flush_tracked()?);
        Ok(())
    }

    /// Returns a page of entries from the collection.
    pub fn take_page<BS: Blockstore>(
        &self,
        store: BS,
        size: u32,
    ) -> Result<Vec<(Hash, BlobSourceSet)>, ActorError> {
        let map = self.hamt(store)?;
        let mut page = Vec::with_capacity(size as usize);
        map.for_each_ranged(None, Some(size as usize), |hash, set| {
            page.push((hash, set.clone()));
            Ok(())
        })?;
        page.shrink_to_fit();
        Ok(page)
    }

    /// Removes a source from an entry in the collection.
    /// If the entry is empty after removing the source, the entry is also removed.
    pub fn remove_source<BS: Blockstore>(
        &mut self,
        store: BS,
        hash: Hash,
        source: BlobSource,
        blob_size: u64,
    ) -> Result<(), ActorError> {
        let mut map = self.hamt(store)?;
        if let Some(mut set) = map.get(&hash)? {
            if set.remove(&source) {
                if set.is_empty() {
                    map.delete(&hash)?;
                    self.bytes_size -= blob_size;
                } else {
                    map.set(&hash, set)?;
                }
                self.save_tracked(map.flush_tracked()?);
            }
        }
        Ok(())
    }

    /// Removes an entry from the collection.
    pub fn remove_entry<BS: Blockstore>(
        &mut self,
        store: BS,
        hash: &Hash,
        blob_size: u64,
    ) -> Result<(), ActorError> {
        let mut map = self.hamt(store)?;
        let (res, deleted) = map.delete_and_flush_tracked(hash)?;
        self.save_tracked(res);
        if deleted.is_some() {
            self.bytes_size -= blob_size;
        }
        Ok(())
    }
}
