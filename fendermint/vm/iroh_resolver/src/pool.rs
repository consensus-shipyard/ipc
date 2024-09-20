// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashSet;

use async_stm::{
    queues::{tchan::TChan, TQueueLike},
    Stm, TVar,
};
use iroh::blobs::Hash;
use iroh::net::{NodeAddr, NodeId};

/// The maximum number of times a task can be attempted.
/// TODO: make configurable
const MAX_RESOLVE_ATTEMPTS: u64 = 3;

/// Hashes we need to resolve.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolveKey {
    pub hash: Hash,
}

/// Hashes we need to resolve.
#[derive(Debug, Copy, Clone)]
pub struct ResolveSource {
    pub id: NodeId,
}

/// Ongoing status of a resolution.
///
/// The status also keeps track of which original items mapped to the same resolution key.
/// Once resolved, they all become available at the same time.
/// TODO: include failure mechanism
#[derive(Clone)]
pub struct ResolveStatus<T> {
    /// The collection of items that all resolve to the same hash.
    items: TVar<im::HashSet<T>>,
    /// Indicate whether the content has been resolved.
    is_resolved: TVar<bool>,
    /// Counter added to by items if they fail.
    num_failures: TVar<u64>,
}

impl<T> ResolveStatus<T>
where
    T: Clone + std::hash::Hash + Eq + PartialEq + Sync + Send + 'static,
{
    pub fn new(item: T) -> Self {
        let mut items = im::HashSet::new();
        items.insert(item);
        Self {
            is_resolved: TVar::new(false),
            num_failures: TVar::new(0),
            items: TVar::new(items),
        }
    }

    pub fn is_resolved(&self) -> Stm<bool> {
        self.is_resolved.read_clone()
    }

    pub fn is_failed(&self) -> Stm<bool> {
        let num_failures = self.num_failures.read_clone()?;
        let num_tasks = self.items.read_clone()?.len() as u64;
        Ok(num_failures == num_tasks)
    }
}

/// Tasks emitted by the pool for background resolution.
#[derive(Clone)]
pub struct ResolveTask {
    /// Content to resolve.
    key: ResolveKey,
    /// Source Iroh node ID from which to resolve the content.
    source: ResolveSource,
    /// Flag to flip when the task is done.
    is_resolved: TVar<bool>,
    /// Current number of resolve attempts.
    num_attempts: TVar<u64>,
    /// Counter to add to if all attempts are used.
    num_failures: TVar<u64>,
}

impl ResolveTask {
    pub fn hash(&self) -> Hash {
        self.key.hash
    }

    pub fn node_addr(&self) -> NodeAddr {
        NodeAddr::new(self.source.id)
    }

    pub fn set_resolved(&self) -> Stm<()> {
        self.is_resolved.write(true)
    }

    /// Adds an attempt and return whether a retry is available.
    pub fn add_attempt(&self) -> Stm<bool> {
        let attempts = self.num_attempts.modify(|mut a| {
            a += 1;
            (a, a)
        })?;
        Ok(attempts < MAX_RESOLVE_ATTEMPTS)
    }

    /// Increments failures on the parent status.
    pub fn add_failure(&self) -> Stm<()> {
        self.num_failures.update(|a| a + 1)
    }
}

pub type ResolveQueue = TChan<ResolveTask>;

/// A data structure used to communicate resolution requirements and outcomes
/// between the resolver running in the background and the application waiting
/// for the results.
///
/// It is designed to resolve a single hash, per item,
/// with the possibility of multiple items mapping to the same hash.
#[derive(Clone, Default)]
pub struct ResolvePool<T>
where
    T: Clone + Sync + Send + 'static,
{
    /// The resolution status of each item.
    items: TVar<im::HashMap<ResolveKey, ResolveStatus<T>>>,
    /// Items queued for resolution.
    queue: ResolveQueue,
}

impl<T> ResolvePool<T>
where
    for<'a> ResolveKey: From<&'a T>,
    for<'a> ResolveSource: From<&'a T>,
    T: Sync + Send + Clone + std::hash::Hash + Eq + PartialEq + 'static,
{
    pub fn new() -> Self {
        Self {
            items: Default::default(),
            queue: Default::default(),
        }
    }

    /// Queue to consume for task items.
    ///
    /// Exposed as-is to allow re-queueing items.
    pub fn queue(&self) -> ResolveQueue {
        self.queue.clone()
    }

    /// Add an item to the resolution targets.
    ///
    /// If the item is new, enqueue it from background resolution, otherwise return its existing status.
    pub fn add(&self, item: T) -> Stm<ResolveStatus<T>> {
        let key = ResolveKey::from(&item);
        let source = ResolveSource::from(&item);
        let mut items = self.items.read_clone()?;

        if items.contains_key(&key) {
            let status = items.get(&key).cloned().unwrap();
            status.items.update_mut(|items| {
                items.insert(item);
            })?;
            Ok(status)
        } else {
            let status = ResolveStatus::new(item);
            items.insert(key, status.clone());
            self.items.write(items)?;
            self.queue.write(ResolveTask {
                key,
                source,
                is_resolved: status.is_resolved.clone(),
                num_attempts: TVar::new(0),
                num_failures: status.num_failures.clone(),
            })?;
            Ok(status)
        }
    }

    /// Return item count.
    pub fn count(&self) -> Stm<usize> {
        Ok(self.items.read()?.len())
    }

    /// Return the status of an item. It can be queried for completion.
    pub fn get_status(&self, item: &T) -> Stm<Option<ResolveStatus<T>>> {
        let key = ResolveKey::from(item);
        Ok(self.items.read()?.get(&key).cloned())
    }

    /// Collect resolved and failed items, ready for execution.
    ///
    /// The items collected are not removed, in case they need to be proposed again.
    pub fn collect_done(&self) -> Stm<HashSet<T>> {
        let mut done = HashSet::new();
        let items = self.items.read()?;
        for item in items.values() {
            if item.is_resolved()? || item.is_failed()? {
                done.extend(item.items.read()?.iter().cloned());
            }
        }
        Ok(done)
    }

    /// Remove an item from the resolution targets.
    pub fn remove(&self, item: &T) -> Stm<()> {
        let key = ResolveKey::from(item);
        self.items.update_mut(|items| {
            items.remove(&key);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ResolveKey, ResolvePool, ResolveSource};

    use async_stm::{atomically, queues::TQueueLike};
    use iroh::base::key::SecretKey;
    use iroh::blobs::Hash;
    use iroh::net::NodeId;
    use rand::Rng;

    #[derive(Clone, Hash, Eq, PartialEq, Debug)]
    struct TestItem {
        hash: Hash,
        source: NodeId,
    }

    impl TestItem {
        pub fn dummy() -> Self {
            let mut rng = rand::thread_rng();
            let mut data = [0u8; 256];
            rng.fill(&mut data);
            let hash = Hash::new(data);
            let source = SecretKey::generate().public();
            Self { hash, source }
        }
    }

    impl From<&TestItem> for ResolveKey {
        fn from(value: &TestItem) -> Self {
            Self { hash: value.hash }
        }
    }

    impl From<&TestItem> for ResolveSource {
        fn from(value: &TestItem) -> Self {
            Self { id: value.source }
        }
    }

    #[tokio::test]
    async fn add_new_item() {
        let pool = ResolvePool::new();
        let item = TestItem::dummy();

        atomically(|| pool.add(item.clone())).await;
        atomically(|| {
            assert!(pool.get_status(&item)?.is_some());
            assert!(!pool.queue.is_empty()?);
            assert_eq!(pool.queue.read()?.key, ResolveKey::from(&item));
            Ok(())
        })
        .await;
    }

    #[tokio::test]
    async fn add_existing_item() {
        let pool = ResolvePool::new();
        let item = TestItem::dummy();

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
            let status = pool.get_status(&item)?;
            assert!(status.is_some());
            assert!(pool.queue.is_empty()?);
            Ok(())
        })
        .await;
    }

    #[tokio::test]
    async fn get_status() {
        let pool = ResolvePool::new();
        let item = TestItem::dummy();

        let status1 = atomically(|| pool.add(item.clone())).await;
        let status2 = atomically(|| pool.get_status(&item))
            .await
            .expect("status exists");

        // Complete the item.
        atomically(|| {
            assert!(!pool.queue.is_empty()?);
            let task = pool.queue.read()?;
            task.is_resolved.write(true)
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
        let item = TestItem::dummy();

        atomically(|| {
            let status = pool.add(item.clone())?;
            status.is_resolved.write(true)?;

            let resolved1 = pool.collect_done()?;
            let resolved2 = pool.collect_done()?;
            assert_eq!(resolved1, resolved2);
            assert!(resolved1.contains(&item));
            Ok(())
        })
        .await;
    }
}
