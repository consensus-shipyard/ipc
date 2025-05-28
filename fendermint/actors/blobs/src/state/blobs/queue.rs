// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashSet;

use fendermint_actor_blobs_shared::{self as shared, blobs::SubscriptionId, bytes::B256};
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{tuple::*, RawBytes};
use fvm_shared::address::Address;
use recall_ipld::hamt::{self, map::TrackedFlushResult, MapKey};

/// Key used to namespace a blob source set.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize_tuple, Deserialize_tuple)]
pub struct BlobSource {
    /// Blob subscriber.
    pub subscriber: Address,
    /// Subscription ID.
    pub id: SubscriptionId,
    /// Source Iroh node ID.
    pub source: B256,
}

impl BlobSource {
    /// Create a new blob source.
    pub fn new(subscriber: Address, id: SubscriptionId, source: B256) -> Self {
        Self {
            subscriber,
            id,
            source,
        }
    }
}

impl std::fmt::Display for BlobSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BlobSource(subscriber: {}, id: {}, source: {})",
            self.subscriber, self.id, self.source
        )
    }
}

impl MapKey for BlobSource {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        let raw_bytes = RawBytes::from(b.to_vec());
        fil_actors_runtime::cbor::deserialize(&raw_bytes, "BlobSource")
            .map_err(|e| format!("Failed to deserialize BlobSource {}", e))
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let raw_bytes = fil_actors_runtime::cbor::serialize(self, "BlobSource")
            .map_err(|e| format!("Failed to serialize BlobSource {}", e))?;
        Ok(raw_bytes.to_vec())
    }
}

/// A set of [`shared::blobs::BlobSource`]s.
/// A blob in the collection may have multiple sources.
type BlobSourceSet = HashSet<shared::blobs::BlobSource>;

/// A collection of blobs used for progress queues.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Queue {
    /// The HAMT root.
    pub root: hamt::Root<B256, hamt::Root<BlobSource, ()>>,
    /// Number of sources in the collection.
    size: u64,
    /// Number of blob bytes in the collection.
    /// A blob with multiple sources is only counted once.
    bytes_size: u64,
}

impl Queue {
    /// Returns a new progress collection.
    pub fn new<BS: Blockstore>(store: &BS, name: &str) -> Result<Self, ActorError> {
        let root = hamt::Root::<B256, hamt::Root<BlobSource, ()>>::new(store, name)?;
        Ok(Self {
            root,
            size: 0,
            bytes_size: 0,
        })
    }

    /// Returns a store name for the inner root.
    fn store_name_per_hash(&self, hash: B256) -> String {
        format!("{}.{}", self.root.name(), hash)
    }

    /// Returns the underlying [`hamt::map::Hamt`].
    pub fn hamt<'a, BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<'a, BS, B256, hamt::Root<BlobSource, ()>>, ActorError> {
        self.root.hamt(store, self.size)
    }

    /// Saves the state from the [`TrackedFlushResult`].
    pub fn save_tracked(
        &mut self,
        tracked_flush_result: TrackedFlushResult<B256, hamt::Root<BlobSource, ()>>,
    ) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }

    /// Number of sources in the collection.
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
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
        hash: B256,
        source: BlobSource,
        blob_size: u64,
    ) -> Result<(), ActorError> {
        let mut collection = self.hamt(&store)?;
        let sources_root = if let Some(sources_root) = collection.get(&hash)? {
            // Modify the existing entry
            let mut sources = sources_root.hamt(&store, 0)?;
            sources.set_and_flush(&source, ())?
        } else {
            // Entry did not exist, add and increase tracked bytes size
            let sources_root =
                hamt::Root::<BlobSource, ()>::new(&store, &self.store_name_per_hash(hash))?;
            let mut sources = sources_root.hamt(&store, 0)?;
            self.bytes_size = self.bytes_size.saturating_add(blob_size);
            sources.set_and_flush(&source, ())?
        };
        self.save_tracked(collection.set_and_flush_tracked(&hash, sources_root)?);
        Ok(())
    }

    /// Returns a page of entries from the collection.
    pub fn take_page<BS: Blockstore>(
        &self,
        store: BS,
        size: u32,
    ) -> Result<Vec<(B256, BlobSourceSet)>, ActorError> {
        let collection = self.hamt(&store)?;
        let mut page = Vec::with_capacity(size as usize);
        collection.for_each_ranged(None, Some(size as usize), |hash, sources_root| {
            let sources = sources_root.hamt(&store, 0)?;
            let mut set = HashSet::new();
            sources.for_each(|source, _| {
                set.insert((source.subscriber, source.id, source.source));
                Ok(())
            })?;
            page.push((hash, set));
            Ok(true)
        })?;
        page.shrink_to_fit();
        Ok(page)
    }

    /// Removes a source from an entry in the collection.
    /// If the entry is empty after removing the source, the entry is also removed.
    pub fn remove_source<BS: Blockstore>(
        &mut self,
        store: BS,
        hash: &B256,
        size: u64,
        source: BlobSource,
    ) -> Result<(), ActorError> {
        let mut collection = self.hamt(&store)?;
        if let Some(mut source_root) = collection.get(hash)? {
            let mut sources = source_root.hamt(&store, 1)?;
            (source_root, _) = sources.delete_and_flush(&source)?;
            if sources.is_empty() {
                self.save_tracked(collection.delete_and_flush_tracked(hash)?.0);
                self.bytes_size = self.bytes_size.saturating_sub(size);
            } else {
                self.save_tracked(collection.set_and_flush_tracked(hash, source_root)?);
            }
        }
        Ok(())
    }

    /// Removes an entry from the collection.
    pub fn remove_entry<BS: Blockstore>(
        &mut self,
        store: BS,
        hash: &B256,
        size: u64,
    ) -> Result<(), ActorError> {
        let mut collection = self.hamt(&store)?;
        let (res, deleted) = collection.delete_and_flush_tracked(hash)?;
        self.save_tracked(res);
        if deleted.is_some() {
            self.bytes_size = self.bytes_size.saturating_sub(size);
        }
        Ok(())
    }
}
