use dashmap::DashMap;
use rustc_hash::FxHasher;
use std::{
    fmt::Display,
    hash::{BuildHasher, BuildHasherDefault, Hash},
    time::Instant,
};

use super::{
    clock::{Clock, InstantClock},
    entry::RateLimitEntry,
};
use crate::{GcraError, RateLimit};

pub type FxBuildHasher = BuildHasherDefault<FxHasher>;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct RateLimitRequest<T: Eq + Hash> {
    key: T,
}

impl<T> Display for RateLimitRequest<T>
where
    T: Display + Eq + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RateLimitRequest={}", self.key)
    }
}

/// A sharded rate limiter implementation using an internal [GcraState] per entry.
/// It is `Send + Sync + Clone` and manages an internal LRU with expiration.
#[derive(Clone)]
pub struct RateLimiter<T: Eq + Hash, C = InstantClock, S = FxBuildHasher> {
    clock: C,
    map: DashMap<RateLimitRequest<T>, RateLimitEntry, S>,
}

impl<Key> RateLimiter<Key, InstantClock, FxBuildHasher>
where
    Key: Send + Clone + Hash + Eq + Display + 'static,
{
    /// Constructs an sharded instance of a rate limiter.
    pub fn new(max_data_capacity: usize) -> Self {
        Self {
            clock: InstantClock,
            map: DashMap::with_capacity_and_hasher(max_data_capacity, FxBuildHasher::default()),
        }
    }

    /// Constructs an sharded instance of a rate limiter with a specific amount of shards.
    pub fn with_shards(max_data_capacity: usize, num_shards: usize) -> Self {
        Self {
            clock: InstantClock,
            map: DashMap::with_capacity_and_hasher_and_shard_amount(
                max_data_capacity,
                FxBuildHasher::default(),
                num_shards,
            ),
        }
    }
}

impl<Key, C, S> RateLimiter<Key, C, S>
where
    Key: Send + Clone + Hash + Eq + Display + 'static,
    C: Clock,
    S: Default + BuildHasher + Clone,
{
    pub fn with_clock(clock: C) -> Self {
        Self {
            clock,
            map: DashMap::default(),
        }
    }

    /// Check to see if [key] is rate limited.
    /// # Errors
    /// - [GcraError::DeniedUntil] if the request can succeed after the [Instant] returned.
    /// - [GcraError::DeniedIndefinitely] if the request can never succeed
    #[inline]
    pub async fn check(
        &mut self,
        key: Key,
        rate_limit: RateLimit,
        cost: u32,
    ) -> Result<Instant, GcraError> {
        self.check_at(key, rate_limit, cost, self.clock.now()).await
    }

    /// Check to see if [key] is rate limited.
    ///
    /// # Errors
    /// - [GcraError::DeniedUntil] if the request can succeed after the [Instant] returned.
    /// - [GcraError::DeniedIndefinitely] if the request can never succeed
    pub async fn check_at(
        &mut self,
        key: Key,
        rate_limit: RateLimit,
        cost: u32,
        arrived_at: Instant,
    ) -> Result<Instant, GcraError> {
        let request_key = RateLimitRequest { key };

        let mut entry = self.map.entry(request_key.clone()).or_default();
        match entry.check_and_modify_at(&rate_limit, arrived_at, cost) {
            Ok(_) => {
                entry.update_expiration(&rate_limit);
                // Guaranteed to be set from update_expiration
                let expires_at = entry.expires_at.unwrap();
                Ok(expires_at)
            }
            Err(e @ GcraError::DeniedUntil { .. }) => Err(e),
            Err(e @ GcraError::DeniedIndefinitely { .. }) => {
                // Free the lock so we can remove the entry
                drop(entry);
                // No need to keep this in the map
                self.map.remove(&request_key);
                Err(e)
            }
        }
    }

    /// Removes entries that have expired
    pub fn prune_expired(&mut self) {
        let now = self.clock.now();

        self.map.retain(|_key, entry| match entry.expires_at {
            Some(expires_at) => expires_at > now,
            None => true,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::rate_limiter::clock::tests::FakeClock;
    use core::panic;
    use std::time::{Duration, Instant};

    use super::*;

    #[tokio::test]
    async fn rate_limiter_run_until_denied() {
        let rate_limit = RateLimit::new(3, Duration::from_secs(3));
        let mut rl = RateLimiter::with_shards(4, 2);

        for _ in 0..rate_limit.resource_limit {
            assert!(
                rl.check("key", rate_limit.clone(), 1).await.is_ok(),
                "Shouldn't be rate limited yet"
            );
        }

        match rl.check("key", rate_limit, 1).await {
            Ok(_) => panic!("We should be rate limited"),
            Err(GcraError::DeniedUntil { next_allowed_at }) => {
                assert!(next_allowed_at > Instant::now())
            }
            Err(_) => panic!("Unexpected error"),
        }
    }

    #[tokio::test]
    async fn rate_limiter_indefinitly_denied() {
        let rate_limit = RateLimit::new(3, Duration::from_secs(3));
        let mut rl = RateLimiter::with_shards(4, 2);

        match rl.check("key", rate_limit.clone(), 9).await {
            Ok(_) => panic!("We should be rate limited"),
            Err(GcraError::DeniedIndefinitely {
                cost,
                rate_limit: err_rate_limit,
            }) => {
                assert_eq!(cost, 9);
                assert_eq!(err_rate_limit, rate_limit);
            }
            Err(_) => panic!("Unexpected error"),
        }
    }

    #[tokio::test]
    async fn rate_limiter_leaks() {
        let rate_limit = RateLimit::per_sec(2);
        let mut rl = RateLimiter::with_shards(4, 2);

        let now = Instant::now();
        assert!(rl.check_at("key", rate_limit.clone(), 1, now).await.is_ok());
        assert!(
            rl.check_at(
                "key",
                rate_limit.clone(),
                1,
                now + Duration::from_millis(250)
            )
            .await
            .is_ok(),
            "delay the 2nd check"
        );
        assert!(
            rl.check_at(
                "key",
                rate_limit.clone(),
                1,
                now + Duration::from_millis(251)
            )
            .await
            .is_err(),
            "check we are denied start"
        );
        assert!(
            rl.check_at(
                "key",
                rate_limit.clone(),
                1,
                now + Duration::from_millis(499)
            )
            .await
            .is_err(),
            "check we are denied end"
        );
        assert!(
            rl.check_at(
                "key",
                rate_limit.clone(),
                1,
                now + Duration::from_millis(501)
            )
            .await
            .is_ok(),
            "1st use should be released"
        )
    }

    #[tokio::test]
    async fn rate_limiter_prune_expired() {
        let clock = FakeClock::new();

        let rate_limit = RateLimit::per_sec(3);
        let mut rl: RateLimiter<_, _, FxBuildHasher> = RateLimiter::with_clock(clock.clone());

        for index in 0..rate_limit.resource_limit {
            assert!(
                rl.check(index, rate_limit.clone(), 1).await.is_ok(),
                "Shouldn't be rate limited yet"
            );
        }

        let before_len = rl.map.len();
        rl.prune_expired();
        let after_len = rl.map.len();
        assert_eq!(
            before_len, after_len,
            "Nothing has expired, no elements should be removed"
        );

        clock.advance_by(Duration::from_secs(10));
        rl.prune_expired();
        let after_len = rl.map.len();
        assert_eq!(
            0, after_len,
            "All entries have expired, no elements expected"
        );
    }
}
