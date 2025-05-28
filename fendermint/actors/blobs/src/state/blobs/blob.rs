// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fendermint_actor_blobs_shared::blobs::SubscriptionId;
use fendermint_actor_blobs_shared::{
    self as shared,
    blobs::{BlobStatus, Subscription},
    bytes::B256,
};
use fil_actors_runtime::{runtime::Runtime, ActorError};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use log::debug;
use recall_ipld::hamt::{self, map::TrackedFlushResult};

use super::{
    AddBlobStateParams, BlobSource, Expiries, ExpiryUpdate, Queue, Subscribers, Subscriptions,
};
use crate::caller::Caller;

/// Represents the result of a blob upsert.
#[derive(Debug, Clone)]
pub struct UpsertBlobResult {
    /// New or updated subscription.
    pub subscription: Subscription,
    /// New capacity used by the caller.
    pub capacity_used: u64,
    /// Duration for the new credit commitment.
    pub commit_duration: ChainEpoch,
    /// Duration for the returned credit commitment.
    pub return_duration: ChainEpoch,
}

/// The stored representation of a blob.
#[derive(Clone, PartialEq, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Blob {
    /// The size of the content.
    pub size: u64,
    /// Blob metadata that contains information for blob recovery.
    pub metadata_hash: B256,
    /// Active subscribers (accounts) that are paying for the blob.
    pub subscribers: Subscribers,
    /// Blob status.
    pub status: BlobStatus,
}

impl Blob {
    /// Returns a new [`Blob`].
    pub fn new<BS: Blockstore>(
        store: &BS,
        size: u64,
        metadata_hash: B256,
    ) -> Result<Self, ActorError> {
        Ok(Self {
            size,
            metadata_hash,
            subscribers: Subscribers::new(store)?,
            status: BlobStatus::Added,
        })
    }

    /// Returns a [`shared::blobs::Blob`] that is safe to return from actor methods.
    /// TODO: HAMTs should carry max expiry such that we don't full scan here.
    pub fn to_shared(&self, rt: &impl Runtime) -> Result<shared::blobs::Blob, ActorError> {
        let store = rt.store();
        let mut subscribers = HashMap::new();
        self.subscribers.hamt(store)?.for_each(|_, group| {
            group.hamt(store)?.for_each(|id, sub| {
                subscribers.insert(id, sub.expiry);
                Ok(())
            })?;
            Ok(())
        })?;
        Ok(shared::blobs::Blob {
            size: self.size,
            metadata_hash: self.metadata_hash,
            subscribers,
            status: self.status.clone(),
        })
    }
}

/// HAMT wrapper for blobs state.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Blobs {
    /// The HAMT root.
    pub root: hamt::Root<B256, Blob>,
    /// Map of expiries to blob hashes.
    pub expiries: Expiries,
    /// Map of currently added blob hashes to account and source Iroh node IDs.
    pub added: Queue,
    /// Map of currently pending blob hashes to account and source Iroh node IDs.
    pub pending: Queue,
    /// Number of blobs in the collection.
    /// A blob with multiple subscribers and/or subscriptions is only counted once.
    size: u64,
    /// Number of blob bytes in the collection.
    /// A blob with multiple subscribers and/or subscriptions is only counted once.
    bytes_size: u64,
}

/// Return type used when getting and hydrating a blob.
#[derive(Debug)]
pub struct GetBlobResult {
    /// The blob that was retrieved.
    pub blob: Blob,
    /// The blob's subscriber subscriptions.
    pub subscriptions: Subscriptions,
    /// The blob subscription.
    pub subscription: Subscription,
}

impl Blobs {
    /// Returns a blob collection.
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<B256, Blob>::new(store, "blobs")?;
        Ok(Self {
            root,
            expiries: Expiries::new(store)?,
            added: Queue::new(store, "added blobs queue")?,
            pending: Queue::new(store, "pending blobs queue")?,
            size: 0,
            bytes_size: 0,
        })
    }

    /// Returns the underlying [`hamt::map::Hamt`].
    pub fn hamt<'a, BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<'a, BS, B256, Blob>, ActorError> {
        self.root.hamt(store, self.size)
    }

    /// Saves the state from the [`TrackedFlushResult`].
    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<B256, Blob>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }

    /// Number of blobs in the collection.
    /// A blob with multiple subscribers and/or subscriptions is only counted once.
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Returns the number of blob bytes in the collection.
    /// A blob with multiple subscribers and/or subscriptions is only counted once.
    pub fn bytes_size(&self) -> u64 {
        self.bytes_size
    }

    /// Sets subnet bytes capacity.
    pub fn set_capacity(&mut self, size: u64) {
        self.bytes_size = size;
    }

    /// Releases subnet bytes capacity.
    pub fn release_capacity(&mut self, size: u64) {
        self.bytes_size = self.bytes_size.saturating_sub(size);

        debug!("released {} bytes to subnet", size);
    }

    /// Retrieves a blob and subscription information for a given subscriber, blob hash,
    /// and subscription ID.
    ///
    /// This function performs a series of lookups to locate both the requested blob and the
    /// specific subscription to that blob for the subscriber:
    /// 1. Retrieve the blob using its hash
    /// 2. Confirm the subscriber is a valid subscriber to blob
    /// 3. Locate the specific subscription by its ID
    pub fn get_and_hydrate<BS: Blockstore>(
        &self,
        store: &BS,
        subscriber: Address,
        hash: B256,
        id: &SubscriptionId,
    ) -> Result<Option<GetBlobResult>, ActorError> {
        let blobs_hamt = self.hamt(store)?;

        // Early return if the blob doesn't exist
        let blob = match blobs_hamt.get(&hash)? {
            Some(blob) => blob,
            None => return Ok(None),
        };

        // Get subscriber's subscriptions
        let subscribers_hamt = blob.subscribers.hamt(store)?;
        let subscriptions = match subscribers_hamt.get(&subscriber)? {
            Some(subscriptions) => subscriptions,
            None => {
                return Err(ActorError::forbidden(format!(
                    "subscriber {} is not subscribed to blob {}",
                    subscriber, hash
                )));
            }
        };

        // Get the subscription by ID
        let subscriptions_hamt = subscriptions.hamt(store)?;
        let subscription = match subscriptions_hamt.get(id)? {
            Some(subscription) => subscription,
            None => {
                return Err(ActorError::not_found(format!(
                    "subscription id {} not found",
                    id
                )));
            }
        };

        Ok(Some(GetBlobResult {
            blob,
            subscriptions,
            subscription,
        }))
    }

    /// Creates or updates a blob and subscription, managing all related state changes.
    ///
    /// This function performs several operations:
    /// 1. Check if the blob exists and create it if not
    /// 2. Add or update the caller's subscription to blob
    /// 3. Update the blob's status to "Added" if it's not already resolved
    /// 4. Update the blob source in the "added" queue
    /// 5. Update expiry indexes for subscription
    /// 6. Save all changes to storage
    ///
    /// The function handles both the creation of new blobs and updates to existing ones,
    /// as well as managing subscriptions, expiries, and status tracking.
    pub fn upsert<BS: Blockstore>(
        &mut self,
        store: &BS,
        caller: &Caller<BS>,
        params: &AddBlobStateParams,
        expiry: ChainEpoch,
    ) -> Result<UpsertBlobResult, ActorError> {
        let mut blobs = self.hamt(store)?;
        let (mut blob, blob_added) = if let Some(blob) = blobs.get(&params.hash)? {
            (blob, false)
        } else {
            (Blob::new(store, params.size, params.metadata_hash)?, true)
        };

        // Add/update subscriber and the subscription
        let result = blob.subscribers.upsert(store, caller, params, expiry)?;

        // Update blob status and added index if the blob is not already resolved
        if !matches!(blob.status, BlobStatus::Resolved) {
            // If failed, reset to added state
            if matches!(blob.status, BlobStatus::Failed) {
                blob.status = BlobStatus::Added;
            }

            // Add to or update the source in the added queue
            self.added.upsert(
                store,
                params.hash,
                BlobSource::new(
                    caller.subscriber_address(),
                    params.id.clone(),
                    params.source,
                ),
                blob.size,
            )?;
        }

        // Update expiry index
        let mut expiry_updates = vec![];
        if let Some(previous_expiry) = result.previous_subscription_expiry {
            if previous_expiry != expiry {
                expiry_updates.push(ExpiryUpdate::Remove(previous_expiry));
                expiry_updates.push(ExpiryUpdate::Add(expiry));
            }
        } else {
            expiry_updates.push(ExpiryUpdate::Add(expiry));
        }
        self.expiries.update(
            store,
            caller.subscriber_address(),
            params.hash,
            &params.id,
            expiry_updates,
        )?;

        self.save_tracked(blobs.set_and_flush_tracked(&params.hash, blob)?);

        // Update global state
        if blob_added {
            self.bytes_size = self.bytes_size.saturating_add(params.size);

            debug!("used {} bytes from subnet", params.size);
            debug!("created new blob {}", params.hash);
        } else {
            debug!("used 0 bytes from subnet");
        }

        Ok(UpsertBlobResult {
            subscription: result.subscription,
            capacity_used: if result.subscriber_added {
                params.size
            } else {
                0
            },
            commit_duration: result.commit_duration,
            return_duration: result.return_duration,
        })
    }

    /// Saves all state changes from a blob retrieval operation.
    ///
    /// This function updates multiple related data structures after a blob has been retrieved:
    /// 1. Update the subscription state in subscriptions collection
    /// 2. Update the subscription list for subscriber
    /// 3. Update the blob entry in the blobs HAMT
    ///
    /// This function ensures that all state changes from a blob retrieval operation are
    /// saved atomically, maintaining data consistency across the different collections.
    pub fn save_result<BS: Blockstore>(
        &mut self,
        store: &BS,
        subscriber: Address,
        hash: B256,
        id: &SubscriptionId,
        blob: &mut GetBlobResult,
    ) -> Result<(), ActorError> {
        blob.subscriptions
            .save_subscription(store, id, blob.subscription.clone())?;

        blob.blob
            .subscribers
            .save_subscriptions(store, subscriber, blob.subscriptions.clone())?;

        let mut blobs = self.hamt(store)?;
        self.save_tracked(blobs.set_and_flush_tracked(&hash, blob.blob.clone())?);

        Ok(())
    }

    /// Deletes a subscription to a blob for a specific caller and returns whether the blob was
    /// also deleted.
    ///
    /// This function removes a specific subscription identified by `id` for the given `caller` to
    /// the blob identified by `hash`. It performs multiple cleanup operations:
    /// 1. Update the expiry index by removing the subscription's expiry entry
    /// 2. Remove the blob source from the "added" queue
    /// 3. Remove the blob source from the "pending" queue
    /// 4. Delete the subscription from the subscriber's subscriptions
    /// 5. If the subscriber has no remaining subscriptions to the blob, remove subscriber
    /// 6. If no subscribers remain for the blob, delete the blob entirely
    pub fn delete_subscription<BS: Blockstore>(
        &mut self,
        store: &BS,
        caller: &Caller<BS>,
        hash: B256,
        id: SubscriptionId,
        blob_result: &mut GetBlobResult,
    ) -> Result<bool, ActorError> {
        // Update expiry index
        self.expiries.update(
            store,
            caller.subscriber_address(),
            hash,
            &id,
            vec![ExpiryUpdate::Remove(blob_result.subscription.expiry)],
        )?;

        // Remove the source from the added queue
        self.added.remove_source(
            store,
            &hash,
            blob_result.blob.size,
            BlobSource::new(
                caller.subscriber_address(),
                id.clone(),
                blob_result.subscription.source,
            ),
        )?;

        // Remove the source from the pending queue
        self.pending.remove_source(
            store,
            &hash,
            blob_result.blob.size,
            BlobSource::new(
                caller.subscriber_address(),
                id.clone(),
                blob_result.subscription.source,
            ),
        )?;

        // Delete subscription
        let mut subscriptions_hamt = blob_result.subscriptions.hamt(store)?;
        blob_result
            .subscriptions
            .save_tracked(subscriptions_hamt.delete_and_flush_tracked(&id)?.0);
        debug!(
            "deleted subscription to blob {} for {} (key: {})",
            hash,
            caller.subscriber_address(),
            id
        );

        // Delete the group if empty
        let mut blobs_hamt = self.hamt(store)?;
        let mut subscribers_hamt = blob_result.blob.subscribers.hamt(store)?;
        let blob_deleted = if blob_result.subscriptions.is_empty() {
            blob_result.blob.subscribers.save_tracked(
                subscribers_hamt
                    .delete_and_flush_tracked(&caller.subscriber_address())?
                    .0,
            );
            debug!(
                "deleted subscriber {} to blob {}",
                caller.subscriber_address(),
                hash
            );

            // Delete or update blob
            let blob_deleted = blob_result.blob.subscribers.is_empty();
            if blob_deleted {
                self.save_tracked(blobs_hamt.delete_and_flush_tracked(&hash)?.0);
                debug!("deleted blob {}", hash);
            } else {
                self.save_tracked(
                    blobs_hamt.set_and_flush_tracked(&hash, blob_result.blob.clone())?,
                );
            }
            blob_deleted
        } else {
            blob_result
                .blob
                .subscribers
                .save_tracked(subscribers_hamt.set_and_flush_tracked(
                    &caller.subscriber_address(),
                    blob_result.subscriptions.clone(),
                )?);
            self.save_tracked(blobs_hamt.set_and_flush_tracked(&hash, blob_result.blob.clone())?);
            false
        };

        Ok(blob_deleted)
    }
}
