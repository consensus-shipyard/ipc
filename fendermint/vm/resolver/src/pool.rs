// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{collections::HashSet, hash::Hash};

use async_stm::{
    queues::{tchan::TChan, TQueueLike},
    StmResult, TVar,
};
use cid::Cid;
use ipc_sdk::subnet_id::SubnetID;

/// CIDs we need to resolve from a specific source subnet, or our own.
pub type ResolveKey = (SubnetID, Cid);

/// Ongoing status of a resolution.
///
/// The status also keeps track of which original items mapped to the same resolution key.
/// These could be for example checkpoint of the same data with slightly different signatories.
/// Once resolved, they all become available at the same time.
#[derive(Clone)]
pub struct ResolveStatus<T> {
    /// Indicate whether the content has been resolved.
    ///
    /// If needed we can expand on this to include failure states.
    is_resolved: TVar<bool>,
    /// The collection of items that all resolve to the same root CID and subnet.
    items: TVar<im::HashSet<T>>,
}

impl<T> ResolveStatus<T>
where
    T: Clone + Hash + Eq + PartialEq + Sync + Send + 'static,
{
    pub fn new(item: T) -> Self {
        let mut items = im::HashSet::new();
        items.insert(item);
        Self {
            is_resolved: TVar::new(false),
            items: TVar::new(items),
        }
    }

    pub fn is_resolved(&self) -> StmResult<bool> {
        self.is_resolved.read_clone()
    }
}

/// A data structure used to communicate resolution requirements and outcomes
/// between the resolver running in the background and the application waiting
/// for the results.
///
/// It is designed to resolve a single CID from a single subnet, per item,
/// with the possibility of multiple items mapping to the same CID.
///
/// If items needed to have multiple CIDs, the completion of all resolutions
/// culminating in the availability of the item, then we have to refactor this
/// component to track dependencies in a different way. For now I am assuming
/// that we can always design our messages in a way that there is a single root.
/// We can also use technical wrappers to submit the same item under different
/// guises and track the completion elsewhere.
#[derive(Clone, Default)]
pub struct ResolvePool<T>
where
    T: Clone + Sync + Send + 'static,
{
    /// The resolution status of each item.
    items: TVar<im::HashMap<ResolveKey, ResolveStatus<T>>>,
    /// Items queued for resolution.
    queue: TChan<(ResolveKey, ResolveStatus<T>)>,
}

impl<T> ResolvePool<T>
where
    for<'a> ResolveKey: From<&'a T>,
    T: Sync + Send + Clone + Hash + Eq + PartialEq + 'static,
{
    pub fn new() -> Self {
        Self {
            items: Default::default(),
            queue: Default::default(),
        }
    }

    /// Add an item to the resolution targets.
    ///
    /// If the item is new, enqueue it from background resolution, otherwise just return its existing status.
    pub fn add(&self, item: T) -> StmResult<ResolveStatus<T>> {
        let key = ResolveKey::from(&item);
        let mut items = self.items.read_clone()?;

        if items.contains_key(&key) {
            let status = items.get(&key).cloned().unwrap();
            status.items.update_mut(|items| {
                items.insert(item);
            })?;
            Ok(status)
        } else {
            let status = ResolveStatus::new(item);
            items.insert(key.clone(), status.clone());
            self.items.write(items)?;
            self.queue.write((key, status.clone()))?;
            Ok(status)
        }
    }

    /// Return the status of an item. It can be queried for completion.
    pub fn get_status(&self, item: &T) -> StmResult<Option<ResolveStatus<T>>> {
        let key = ResolveKey::from(item);
        Ok(self.items.read()?.get(&key).cloned())
    }

    /// Collect resolved items, ready for execution.
    ///
    /// The items removed are not removed, in case they need to be proposed again.
    pub fn collect_resolved(&self) -> StmResult<HashSet<T>> {
        let mut resolved = HashSet::new();
        let items = self.items.read()?;
        for item in items.values() {
            if item.is_resolved()? {
                let items = item.items.read()?;
                resolved.extend(items.iter().cloned());
            }
        }
        Ok(resolved)
    }

    // TODO #197: Implement methods to remove executed items.
}

#[cfg(test)]
mod tests {
    use async_stm::{atomically, queues::TQueueLike};
    use cid::Cid;
    use ipc_sdk::subnet_id::SubnetID;

    #[derive(Clone, Hash, Eq, PartialEq, Debug)]
    struct TestItem {
        subnet_id: SubnetID,
        cid: Cid,
    }

    impl TestItem {
        pub fn dummy(root_id: u64) -> Self {
            Self {
                subnet_id: SubnetID::new_root(root_id),
                cid: Cid::default(),
            }
        }
    }

    impl From<&TestItem> for ResolveKey {
        fn from(value: &TestItem) -> Self {
            (value.subnet_id.clone(), value.cid)
        }
    }

    use super::{ResolveKey, ResolvePool};

    #[tokio::test]
    async fn add_new_item() {
        let pool = ResolvePool::new();
        let item = TestItem::dummy(0);

        atomically(|| pool.add(item.clone())).await;
        atomically(|| {
            assert!(pool.get_status(&item)?.is_some());
            assert!(!pool.queue.is_empty()?);
            assert_eq!(pool.queue.read()?.0, ResolveKey::from(&item));
            Ok(())
        })
        .await;
    }

    #[tokio::test]
    async fn add_existing_item() {
        let pool = ResolvePool::new();
        let item = TestItem::dummy(0);

        // Add once.
        atomically(|| pool.add(item.clone())).await;

        // Consume it from the queue.
        atomically(|| {
            assert!(!pool.queue.is_empty()?);
            let _ = pool.queue.read()?;
            Ok(())
        })
        .await;

        // Add again.
        atomically(|| pool.add(item.clone())).await;

        // Should not be queued a second time.
        atomically(|| {
            assert!(pool.get_status(&item)?.is_some());
            assert!(pool.queue.is_empty()?);
            Ok(())
        })
        .await;
    }

    #[tokio::test]
    async fn get_status() {
        let pool = ResolvePool::new();
        let item = TestItem::dummy(0);

        let status1 = atomically(|| pool.add(item.clone())).await;
        let status2 = atomically(|| pool.get_status(&item))
            .await
            .expect("status exists");

        // Complete the item.
        atomically(|| {
            assert!(!pool.queue.is_empty()?);
            let (_, status) = pool.queue.read()?;
            status.is_resolved.write(true)
        })
        .await;

        // Check status.
        atomically(|| {
            assert!(status1.items.read()?.contains(&item));
            assert!(status1.is_resolved()?);
            assert!(status2.is_resolved()?);
            Ok(())
        })
        .await;
    }

    #[tokio::test]
    async fn collect_resolved() {
        let pool = ResolvePool::new();
        let item = TestItem::dummy(0);

        atomically(|| {
            let status = pool.add(item.clone())?;
            status.is_resolved.write(true)?;

            let resolved1 = pool.collect_resolved()?;
            let resolved2 = pool.collect_resolved()?;
            assert_eq!(resolved1, resolved2);
            assert!(resolved1.contains(&item));
            Ok(())
        })
        .await;
    }
}
