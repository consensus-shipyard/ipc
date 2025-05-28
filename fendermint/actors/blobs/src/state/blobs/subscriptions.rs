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
        overlap: ChainEpoch,
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
                overlap,
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

#[cfg(test)]
mod tests {
    use super::*;
    use fendermint_actor_blobs_shared::blobs::{Subscription, SubscriptionId};
    use fendermint_actor_blobs_testing::new_pk;
    use fvm_ipld_blockstore::MemoryBlockstore;
    use fvm_shared::clock::ChainEpoch;

    fn create_test_subscription(
        id: &str,
        added: ChainEpoch,
        expiry: ChainEpoch,
        failed: bool,
    ) -> (SubscriptionId, Subscription) {
        let subscription_id = SubscriptionId::new(id).unwrap();
        let subscription = Subscription {
            added,
            overlap: 0,
            expiry,
            source: new_pk(),
            delegate: None,
            failed,
        };
        (subscription_id, subscription)
    }

    #[test]
    fn test_max_expiries_empty_group() {
        let store = MemoryBlockstore::default();
        let subscriptions = Subscriptions::new(&store).unwrap();

        let target_id = SubscriptionId::new("not-exists").unwrap();
        let (max, new_max) = subscriptions
            .max_expiries(&store, &target_id, Some(100))
            .unwrap();

        assert_eq!(max, None, "Max expiry should be None for empty group");
        assert_eq!(
            new_max,
            Some(100),
            "New max should be the new value when group is empty"
        );
    }

    #[test]
    fn test_max_expiries_single_subscription() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add a single subscription
        let (id, subscription) = create_test_subscription("test1", 0, 50, false);
        subscriptions
            .save_subscription(&store, &id, subscription)
            .unwrap();

        // Test with existing ID
        let (max, new_max) = subscriptions.max_expiries(&store, &id, Some(100)).unwrap();
        assert_eq!(
            max,
            Some(50),
            "Max should be the existing subscription's expiry"
        );
        assert_eq!(new_max, Some(100), "New max should be the new value");

        // Test with non-existing ID
        let non_existing_id = SubscriptionId::new("not-exists").unwrap();
        let (max, new_max) = subscriptions
            .max_expiries(&store, &non_existing_id, Some(80))
            .unwrap();
        assert_eq!(
            max,
            Some(50),
            "Max should be the existing subscription's expiry"
        );
        assert_eq!(
            new_max,
            Some(80),
            "New max should be the new value for non-existing ID"
        );
    }

    #[test]
    fn test_max_expiries_multiple_subscriptions() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add multiple subscriptions with different expiries
        let (id1, sub1) = create_test_subscription("test1", 0, 50, false);
        let (id2, sub2) = create_test_subscription("test2", 0, 70, false);
        let (id3, sub3) = create_test_subscription("test3", 0, 30, false);
        subscriptions.save_subscription(&store, &id1, sub1).unwrap();
        subscriptions.save_subscription(&store, &id2, sub2).unwrap();
        subscriptions.save_subscription(&store, &id3, sub3).unwrap();

        // Test updating the middle expiry
        let (max, new_max) = subscriptions.max_expiries(&store, &id1, Some(60)).unwrap();
        assert_eq!(max, Some(70), "Max should be the highest expiry (70)");
        assert_eq!(
            new_max,
            Some(70),
            "New max should still be 70 after update to 60"
        );

        // Test updating to the new highest expiry
        let (max, new_max) = subscriptions.max_expiries(&store, &id1, Some(100)).unwrap();
        assert_eq!(max, Some(70), "Max should be the highest expiry (70)");
        assert_eq!(new_max, Some(100), "New max should be 100 after update");

        // Test with non-existing ID
        let non_existing_id = SubscriptionId::new("not-exists").unwrap();
        let (max, new_max) = subscriptions
            .max_expiries(&store, &non_existing_id, Some(120))
            .unwrap();
        assert_eq!(max, Some(70), "Max should be the highest expiry (70)");
        assert_eq!(
            new_max,
            Some(120),
            "New max should be 120 for non-existing ID"
        );
    }

    #[test]
    fn test_max_expiries_with_failed_subscriptions() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add a mix of failed and non-failed subscriptions
        let (id1, sub1) = create_test_subscription("test1", 0, 50, true); // Failed
        let (id2, sub2) = create_test_subscription("test2", 0, 70, false); // Not failed
        let (id3, sub3) = create_test_subscription("test3", 0, 90, true); // Failed (highest)
        subscriptions.save_subscription(&store, &id1, sub1).unwrap();
        subscriptions.save_subscription(&store, &id2, sub2).unwrap();
        subscriptions.save_subscription(&store, &id3, sub3).unwrap();

        // Failed subscriptions should be ignored in max calculation
        let (max, new_max) = subscriptions.max_expiries(&store, &id2, Some(60)).unwrap();
        assert_eq!(
            max,
            Some(70),
            "Max should only consider non-failed subscriptions (70)"
        );
        assert_eq!(new_max, Some(60), "New max should be 60 after update");

        // Test updating a failed subscription
        let (max, new_max) = subscriptions.max_expiries(&store, &id1, Some(100)).unwrap();
        assert_eq!(
            max,
            Some(70),
            "Max should only consider non-failed subscriptions (70)"
        );
        assert_eq!(
            new_max,
            Some(100),
            "New max should be 100 after updating a failed subscription"
        );
    }

    #[test]
    fn test_max_expiries_with_none_new_value() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add subscriptions
        let (id1, sub1) = create_test_subscription("test1", 0, 50, false);
        let (id2, sub2) = create_test_subscription("test2", 0, 70, false);
        subscriptions.save_subscription(&store, &id1, sub1).unwrap();
        subscriptions.save_subscription(&store, &id2, sub2).unwrap();

        // Test with None as new_value - should calculate without modifying
        let (max, new_max) = subscriptions.max_expiries(&store, &id1, None).unwrap();
        assert_eq!(max, Some(70), "Max should be the highest expiry (70)");
        assert_eq!(
            new_max,
            Some(70),
            "New max should remain 70 when target expiry is None"
        );

        // Test with target_id that doesn't exist and None as new_value
        let non_existing_id = SubscriptionId::new("not-exists").unwrap();
        let (max, new_max) = subscriptions
            .max_expiries(&store, &non_existing_id, None)
            .unwrap();
        assert_eq!(max, Some(70), "Max should be the highest expiry (70)");
        assert_eq!(
            new_max,
            Some(70),
            "New max should remain 70 for non-existing ID with None value"
        );
    }

    #[test]
    fn test_max_expiries_with_zero_new_value() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add subscriptions
        let (id1, sub1) = create_test_subscription("test1", 0, 50, false);
        let (id2, sub2) = create_test_subscription("test2", 0, 70, false);
        subscriptions.save_subscription(&store, &id1, sub1).unwrap();
        subscriptions.save_subscription(&store, &id2, sub2).unwrap();

        // Test with zero as new_value for the highest expiry
        let (max, new_max) = subscriptions.max_expiries(&store, &id2, Some(0)).unwrap();
        assert_eq!(max, Some(70), "Max should be the highest expiry (70)");
        assert_eq!(
            new_max,
            Some(50),
            "New max should be 50 after setting highest to 0"
        );

        // Test with zero as new_value for the lowest expiry
        let (max, new_max) = subscriptions.max_expiries(&store, &id1, Some(0)).unwrap();
        assert_eq!(max, Some(70), "Max should be the highest expiry (70)");
        assert_eq!(
            new_max,
            Some(70),
            "New max should be the highest expiry (70)"
        );
    }

    #[test]
    fn test_max_expiries_with_one_zero_new_value() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add subscriptions
        let (id1, sub1) = create_test_subscription("test1", 0, 50, true);
        let (id2, sub2) = create_test_subscription("test2", 0, 70, false);
        subscriptions.save_subscription(&store, &id1, sub1).unwrap();
        subscriptions.save_subscription(&store, &id2, sub2).unwrap();

        // Test with zero as new_value for the highest expiry
        let (max, new_max) = subscriptions.max_expiries(&store, &id2, Some(0)).unwrap();
        assert_eq!(max, Some(70), "Max should be the highest expiry (70)");
        assert_eq!(
            new_max, None,
            "New max should be None after setting highest to 0"
        );

        // Test with zero as new_value for the lowest expiry
        let (max, new_max) = subscriptions.max_expiries(&store, &id1, Some(0)).unwrap();
        assert_eq!(max, Some(70), "Max should be the highest expiry (70)");
        assert_eq!(
            new_max,
            Some(70),
            "New max should be the highest expiry (70)"
        );
    }

    #[test]
    fn test_is_min_added_empty_group() {
        let store = MemoryBlockstore::default();
        let subscriptions = Subscriptions::new(&store).unwrap();

        let target_id = SubscriptionId::new("nonexistent").unwrap();
        let result = subscriptions.is_min_added(&store, &target_id);

        // This should return not found error since no subscription exists
        assert!(result.is_err());

        // Verify it's the expected error type
        match result {
            Err(e) => {
                assert!(e.to_string().contains("not found"));
                assert!(e.to_string().contains("nonexistent"));
            }
            _ => panic!("Expected not found error"),
        }
    }

    #[test]
    fn test_is_min_added_single_subscription() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add a single subscription
        let (id, subscription) = create_test_subscription("test1", 100, 200, false);
        subscriptions
            .save_subscription(&store, &id, subscription)
            .unwrap();

        // Check if it's the minimum (it should be since it's the only one)
        let (is_min, next_min) = subscriptions.is_min_added(&store, &id).unwrap();
        assert!(is_min, "Single subscription should be minimum");
        assert_eq!(next_min, None, "No next minimum should exist");
    }

    #[test]
    fn test_is_min_added_multiple_subscriptions_is_min() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add multiple subscriptions with the first having the earliest added timestamp
        let (id1, sub1) = create_test_subscription("test1", 100, 200, false);
        let (id2, sub2) = create_test_subscription("test2", 150, 250, false);
        let (id3, sub3) = create_test_subscription("test3", 200, 300, false);
        subscriptions.save_subscription(&store, &id1, sub1).unwrap();
        subscriptions.save_subscription(&store, &id2, sub2).unwrap();
        subscriptions.save_subscription(&store, &id3, sub3).unwrap();

        // Check if id1 is the minimum (it should be)
        let (is_min, next_min) = subscriptions.is_min_added(&store, &id1).unwrap();
        assert!(
            is_min,
            "Subscription with earliest added timestamp should be minimum"
        );
        assert_eq!(next_min, Some(150), "Next minimum should be 150 (from id2)");
    }

    #[test]
    fn test_is_min_added_multiple_subscriptions_not_min() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add multiple subscriptions with the second one not being the earliest
        let (id1, sub1) = create_test_subscription("test1", 100, 200, false);
        let (id2, sub2) = create_test_subscription("test2", 150, 250, false);
        let (id3, sub3) = create_test_subscription("test3", 200, 300, false);
        subscriptions.save_subscription(&store, &id1, sub1).unwrap();
        subscriptions.save_subscription(&store, &id2, sub2).unwrap();
        subscriptions.save_subscription(&store, &id3, sub3).unwrap();

        // Check if id2 is the minimum (it shouldn't be)
        let (is_min, next_min) = subscriptions.is_min_added(&store, &id2).unwrap();
        assert!(
            !is_min,
            "Subscription with later added timestamp should not be minimum"
        );
        assert_eq!(
            next_min, None,
            "Next minimum should be None when not the minimum"
        );
    }

    #[test]
    fn test_is_min_added_equal_timestamps() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add multiple subscriptions with equal earliest timestamps
        let (id1, sub1) = create_test_subscription("test1", 100, 200, false);
        let (id2, sub2) = create_test_subscription("test2", 100, 250, false);
        let (id3, sub3) = create_test_subscription("test3", 200, 300, false);
        subscriptions.save_subscription(&store, &id1, sub1).unwrap();
        subscriptions.save_subscription(&store, &id2, sub2).unwrap();
        subscriptions.save_subscription(&store, &id3, sub3).unwrap();

        // Check id1 - both id1 and id2 have the same timestamp
        let (is_min, next_min) = subscriptions.is_min_added(&store, &id1).unwrap();
        assert!(
            is_min,
            "Subscription with equal earliest timestamp should be minimum"
        );
        assert_eq!(next_min, Some(100), "Next minimum should be 100 (from id2)");

        // Check id2 - both id1 and id2 have the same timestamp
        let (is_min, next_min) = subscriptions.is_min_added(&store, &id2).unwrap();
        assert!(
            is_min,
            "Subscription with equal earliest timestamp should be minimum"
        );
        assert_eq!(next_min, Some(100), "Next minimum should be 100 (from id1)");
    }

    #[test]
    fn test_is_min_added_with_failed_subscriptions() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add multiple subscriptions with failed ones having earlier timestamps
        let (id1, sub1) = create_test_subscription("test1", 50, 150, true); // Failed (earliest)
        let (id2, sub2) = create_test_subscription("test2", 100, 200, false); // Not failed (should be min)
        let (id3, sub3) = create_test_subscription("test3", 75, 175, true); // Failed (between id1 and id2)
        let (id4, sub4) = create_test_subscription("test4", 150, 250, false); // Not failed (later)
        subscriptions.save_subscription(&store, &id1, sub1).unwrap();
        subscriptions.save_subscription(&store, &id2, sub2).unwrap();
        subscriptions.save_subscription(&store, &id3, sub3).unwrap();
        subscriptions.save_subscription(&store, &id4, sub4).unwrap();

        // Check if id2 is the minimum (it should be since failed ones are ignored)
        let (is_min, next_min) = subscriptions.is_min_added(&store, &id2).unwrap();
        assert!(
            is_min,
            "Non-failed subscription with earliest timestamp should be minimum"
        );
        assert_eq!(next_min, Some(150), "Next minimum should be 150 (from id4)");

        // Check a failed subscription
        let (is_min, next_min) = subscriptions.is_min_added(&store, &id1).unwrap();
        assert!(is_min, "Failed subscription is checked against itself"); // This is somewhat counterintuitive
        assert_eq!(next_min, Some(100), "Next minimum should be 100 (from id2)");
    }

    #[test]
    fn test_is_min_added_all_other_subscriptions_are_failed() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add multiple subscriptions where all others are failed
        let (id1, sub1) = create_test_subscription("test1", 100, 200, true); // Failed
        let (id2, sub2) = create_test_subscription("test2", 150, 250, false); // Only non-failed subscription
        let (id3, sub3) = create_test_subscription("test3", 50, 150, true); // Failed, earliest
        subscriptions.save_subscription(&store, &id1, sub1).unwrap();
        subscriptions.save_subscription(&store, &id2, sub2).unwrap();
        subscriptions.save_subscription(&store, &id3, sub3).unwrap();

        // Check if id2 is the minimum (it should be since all others are failed)
        let (is_min, next_min) = subscriptions.is_min_added(&store, &id2).unwrap();
        assert!(is_min, "Only non-failed subscription should be minimum");
        assert_eq!(
            next_min, None,
            "No next minimum should exist when all others are failed"
        );
    }

    #[test]
    fn test_is_min_added_with_nonexistent_id() {
        let store = MemoryBlockstore::default();
        let mut subscriptions = Subscriptions::new(&store).unwrap();

        // Add some subscriptions
        let (id1, sub1) = create_test_subscription("test1", 100, 200, false);
        let (id2, sub2) = create_test_subscription("test2", 150, 250, false);
        subscriptions.save_subscription(&store, &id1, sub1).unwrap();
        subscriptions.save_subscription(&store, &id2, sub2).unwrap();

        // Check with nonexistent ID
        let nonexistent_id = SubscriptionId::new("nonexistent").unwrap();
        let result = subscriptions.is_min_added(&store, &nonexistent_id);

        // Should return a "not found" error
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert!(e.to_string().contains("not found"));
                assert!(e.to_string().contains("nonexistent"));
            }
            _ => panic!("Expected not found error"),
        }
    }
}
