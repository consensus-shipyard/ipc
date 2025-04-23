// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::str::from_utf8;

use fendermint_actor_blobs_shared::blobs::{Subscription, SubscriptionId};
use fil_actors_runtime::ActorError;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::clock::ChainEpoch;
use log::debug;
use recall_ipld::{hamt, hamt::map::TrackedFlushResult};

use super::AddBlobStateParams;
use crate::caller::Caller;

/// Represents the result of a subscription upsert.
#[derive(Debug, Clone)]
pub struct UpsertSubscriptionResult {
    /// New or updated subscription.
    pub subscription: Subscription,
    /// Previous subscription expiry if the subscription was updated.
    pub previous_expiry: Option<ChainEpoch>,
}

/// HAMT wrapper tracking blob [`Subscription`]s by subscription ID.
#[derive(Debug, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Subscriptions {
    /// The HAMT root.
    pub root: hamt::Root<SubscriptionId, Subscription>,
    /// The size of the collection.
    size: u64,
}

impl Subscriptions {
    /// Returns a subscription collection.
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<SubscriptionId, Subscription>::new(store, "subscription_group")?;
        Ok(Self { root, size: 0 })
    }

    /// Returns the underlying [`hamt::map::Hamt`].
    pub fn hamt<BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<BS, SubscriptionId, Subscription>, ActorError> {
        self.root.hamt(store, self.size)
    }

    /// Saves the state from the [`TrackedFlushResult`].
    pub fn save_tracked(
        &mut self,
        tracked_flush_result: TrackedFlushResult<SubscriptionId, Subscription>,
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

    /// Calculates the current maximum expiry and the new maximum expiry after a potential update.
    ///
    /// This function serves two purposes:
    /// 1. It finds the current maximum expiry among all non-failed subscriptions
    /// 2. It calculates what the new maximum expiry would be if the subscription with `target_id`
    ///    had its expiry updated to `new_value`
    ///
    /// This is particularly useful for determining if group expiry boundaries need to be updated
    /// when a single subscription's expiry changes.
    pub fn max_expiries<BS: Blockstore>(
        &self,
        store: &BS,
        target_id: &SubscriptionId,
        new_value: Option<ChainEpoch>,
    ) -> Result<(Option<ChainEpoch>, Option<ChainEpoch>), ActorError> {
        let mut max = None;
        let mut new_max = None;
        let subscriptions = self.hamt(store)?;
        for val in subscriptions.iter() {
            let (id, sub) = deserialize_iter_sub(val)?;
            if sub.failed {
                continue;
            }
            if sub.expiry > max.unwrap_or(0) {
                max = Some(sub.expiry);
            }
            let new_value = if &id == target_id {
                new_value.unwrap_or_default()
            } else {
                sub.expiry
            };
            if new_value > new_max.unwrap_or(0) {
                new_max = Some(new_value);
            }
        }
        // Target ID may not be in the current group
        if let Some(new_value) = new_value {
            if new_value > new_max.unwrap_or(0) {
                new_max = Some(new_value);
            }
        }
        Ok((max, new_max))
    }

    /// Determines if a subscription has the earliest added timestamp and finds the next earliest
    /// timestamp.
    ///
    /// This function checks if the subscription identified by `trim_id` has the earliest "added"
    /// timestamp among all active, non-failed subscriptions. It also identifies what would be the
    /// new earliest timestamp if this subscription were removed.
    ///
    /// This is typically used when deciding if a subscription can be safely removed without
    /// affecting the overall data retention requirements of the system.
    pub fn is_min_added<BS: Blockstore>(
        &self,
        store: &BS,
        trim_id: &SubscriptionId,
    ) -> Result<(bool, Option<ChainEpoch>), ActorError> {
        let subscriptions = self.hamt(store)?;
        let trim = subscriptions
            .get(trim_id)?
            .ok_or(ActorError::not_found(format!(
                "subscription id {} not found",
                trim_id
            )))?;

        let mut next_min = None;
        for val in subscriptions.iter() {
            let (id, sub) = deserialize_iter_sub(val)?;
            if sub.failed || &id == trim_id {
                continue;
            }
            if sub.added < trim.added {
                return Ok((false, None));
            }
            if sub.added < next_min.unwrap_or(ChainEpoch::MAX) {
                next_min = Some(sub.added);
            }
        }
        Ok((true, next_min))
    }

    /// Creates a new subscription or updates an existing one with the provided parameters.
    ///
    /// This function handles both the creation and update cases for blob subscriptions:
    /// - If a subscription with the given ID already exists, it updates its properties
    /// - If no subscription exists with the ID, it creates a new one
    ///
    /// When updating an existing subscription, it preserves the original subscription's
    /// added timestamp but updates the expiry, source, delegate, and resets the failed flag.
    pub fn upsert<BS: Blockstore>(
        &mut self,
        store: &BS,
        caller: &Caller<BS>,
        params: &AddBlobStateParams,
        expiry: ChainEpoch,
    ) -> Result<UpsertSubscriptionResult, ActorError> {
        let mut subscriptions = self.hamt(store)?;
        if let Some(mut subscription) = subscriptions.get(&params.id)? {
            let previous_expiry = subscription.expiry;
            subscription.expiry = expiry;
            subscription.source = params.source; // subscriber can retry from a different source
            subscription.delegate = caller.delegate_address();
            subscription.failed = false;

            self.save_tracked(
                subscriptions.set_and_flush_tracked(&params.id, subscription.clone())?,
            );

            debug!(
                "updated subscription to blob {} for {} (key: {})",
                params.hash,
                caller.subscriber_address(),
                params.id
            );

            Ok(UpsertSubscriptionResult {
                subscription,
                previous_expiry: Some(previous_expiry),
            })
        } else {
            let subscription = Subscription {
                added: params.epoch,
                expiry,
                source: params.source,
                delegate: caller.delegate_address(),
                failed: false,
            };

            self.save_tracked(
                subscriptions.set_and_flush_tracked(&params.id, subscription.clone())?,
            );

            debug!(
                "created new subscription to blob {} for {} (key: {})",
                params.hash,
                caller.subscriber_address(),
                params.id
            );

            Ok(UpsertSubscriptionResult {
                subscription,
                previous_expiry: None,
            })
        }
    }

    /// Saves a subscription with the given ID to the blockstore.
    ///
    /// This is a helper function that simplifies the process of saving a subscription
    /// by handling the HAMT operations internally. It creates or updates the subscription
    /// in the HAMT and saves the changes to the blockstore.
    pub fn save_subscription<BS: Blockstore>(
        &mut self,
        store: &BS,
        id: &SubscriptionId,
        subscription: Subscription,
    ) -> Result<(), ActorError> {
        let mut subscriptions = self.hamt(store)?;
        self.save_tracked(subscriptions.set_and_flush_tracked(id, subscription)?);
        Ok(())
    }
}

fn deserialize_iter_sub<'a>(
    val: Result<(&hamt::BytesKey, &'a Subscription), hamt::Error>,
) -> Result<(SubscriptionId, &'a Subscription), ActorError> {
    let (id_bytes, sub) = val.map_err(|e| {
        ActorError::illegal_state(format!(
            "failed to deserialize subscription from iter: {}",
            e
        ))
    })?;
    let id = from_utf8(id_bytes).map_err(|e| {
        ActorError::illegal_state(format!(
            "failed to deserialize subscription ID from iter: {}",
            e
        ))
    })?;
    let subscription_id = SubscriptionId::new(id).map_err(|e| {
        ActorError::illegal_state(format!("failed to decode subscription ID from iter: {}", e))
    })?;
    Ok((subscription_id, sub))
}
