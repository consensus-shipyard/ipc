// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::blobs::Subscription;
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, clock::ChainEpoch};
use recall_ipld::{hamt, hamt::map::TrackedFlushResult};

use super::{AddBlobStateParams, Subscriptions};
use crate::caller::Caller;

/// Represents the result of a subscriber upsert.
#[derive(Debug, Clone)]
pub struct UpsertSubscriberResult {
    /// New or updated subscription.
    pub subscription: Subscription,
    /// Whether the subscriber was added or updated.
    pub subscriber_added: bool,
    /// Previous subscription expiry if the subscription was updated.
    pub previous_subscription_expiry: Option<ChainEpoch>,
    /// Duration for the new credit commitment.
    pub commit_duration: ChainEpoch,
    /// Duration for the returned credit commitment.
    pub return_duration: ChainEpoch,
}

/// HAMT wrapper tracking blob [`Subscriptions`]s by subscriber address.
#[derive(Debug, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Subscribers {
    /// The HAMT root.
    pub root: hamt::Root<Address, Subscriptions>,
    /// The size of the collection.
    size: u64,
}

impl Subscribers {
    /// Returns a subscriber collection.
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Address, Subscriptions>::new(store, "blob_subscribers")?;
        Ok(Self { root, size: 0 })
    }

    /// Returns the underlying [`hamt::map::Hamt`].
    pub fn hamt<'a, BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<'a, BS, Address, Subscriptions>, ActorError> {
        self.root.hamt(store, self.size)
    }

    /// Saves the state from the [`TrackedFlushResult`].
    pub fn save_tracked(
        &mut self,
        tracked_flush_result: TrackedFlushResult<Address, Subscriptions>,
    ) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size;
    }

    /// The size of the collection.
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Creates or updates a subscriber's subscription to a blob, managing all related state
    /// changes.
    ///
    /// This function handles both the creation of new subscribers and updating existing
    /// subscribers' subscriptions. It calculates credit commitment and return durations based on
    /// the subscription's expiry and the group's maximum expiry.
    pub fn upsert<BS: Blockstore>(
        &mut self,
        store: &BS,
        caller: &Caller<BS>,
        params: &AddBlobStateParams,
        expiry: ChainEpoch,
    ) -> Result<UpsertSubscriberResult, ActorError> {
        let mut subscribers = self.hamt(store)?;
        let mut subscriptions =
            if let Some(subscriptions) = subscribers.get(&caller.subscriber_address())? {
                subscriptions
            } else {
                Subscriptions::new(store)?
            };

        // If the subscriber has been debited after the group's max expiry, we need to
        // determine the duration for which credits will be returned.
        // The return duration can only extend up to the current epoch.
        let (group_expiry, new_group_expiry) =
            subscriptions.max_expiries(store, &params.id, Some(expiry))?;
        let return_duration = group_expiry
            .filter(|&expiry| params.epoch > expiry)
            .map_or(0, |expiry| params.epoch - expiry);

        // Determine the duration for which credits will be committed, considering the subscription
        // group may have expiries that cover a portion of the added duration.
        // Duration can be negative if the subscriber is reducing expiry.
        let new_group_expiry = new_group_expiry.unwrap(); // safe here
        let commit_start = group_expiry.map_or(params.epoch, |e| e.max(params.epoch));
        let commit_duration = new_group_expiry - commit_start;
        let overlap = commit_start - group_expiry.unwrap_or(params.epoch);

        // Add/update subscription
        let result = subscriptions.upsert(store, caller, params, overlap, expiry)?;

        self.save_tracked(
            subscribers.set_and_flush_tracked(&caller.subscriber_address(), subscriptions)?,
        );

        Ok(UpsertSubscriberResult {
            subscription: result.subscription,
            subscriber_added: group_expiry.is_none(),
            previous_subscription_expiry: result.previous_expiry,
            commit_duration,
            return_duration,
        })
    }

    /// Saves a subscriber's subscriptions to the blockstore.
    ///
    /// This is a helper function that simplifies the process of saving a subscriber's subscription
    /// data by handling the HAMT operations internally. It creates or updates the subscriber entry
    /// in the HAMT and saves the changes to the blockstore.
    pub fn save_subscriptions<BS: Blockstore>(
        &mut self,
        store: &BS,
        subscriber: Address,
        subscriptions: Subscriptions,
    ) -> Result<(), ActorError> {
        let mut subscribers = self.hamt(store)?;
        self.save_tracked(subscribers.set_and_flush_tracked(&subscriber, subscriptions)?);
        Ok(())
    }
}
