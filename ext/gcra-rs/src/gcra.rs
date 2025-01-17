use std::time::Instant;
use thiserror::Error;

use crate::rate_limit::RateLimit;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum GcraError {
    /// Cost of the increment exceeds the rate limit and  will never succeed
    #[error("Cost of the increment ({cost}) exceeds the rate limit ({rate_limit:?}) and will never succeed)")]
    DeniedIndefinitely { cost: u32, rate_limit: RateLimit },
    /// Limited request until after the [Instant]
    #[error("Denied until {next_allowed_at:?}")]
    DeniedUntil { next_allowed_at: Instant },
}

/// Holds the minmum amount of state necessary to implement a GRCA leaky buckets.
/// Refer to: [understanding GCRA](https://blog.ian.stapletoncordas.co/2018/12/understanding-generic-cell-rate-limiting.html)
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct GcraState {
    /// GCRA's Theoretical Arrival Time (**TAT**)
    /// An unset value signals a new state
    pub tat: Option<Instant>,
}

impl GcraState {
    /// Check if we are allowed to proceed. If so updated our internal state and return true.
    ///
    /// Simply passes the current Instant to [`check_and_modify_at()`]
    #[inline]
    pub fn check_and_modify(&mut self, rate_limit: &RateLimit, cost: u32) -> Result<(), GcraError> {
        let arrived_at = Instant::now();
        self.check_and_modify_at(rate_limit, arrived_at, cost)
    }

    /// Check if we are allowed to proceed at the given arrival time.
    /// If so updated our internal state and return true.
    /// Explaination of GCRA can be found [here](https://blog.ian.stapletoncordas.co/2018/12/understanding-generic-cell-rate-limiting.html)
    ///
    /// # Returns
    /// If denied, will return an [Result::Err] where the value is the next allowed timestamp.
    pub fn check_and_modify_at(
        &mut self,
        rate_limit: &RateLimit,
        arrived_at: Instant,
        cost: u32,
    ) -> Result<(), GcraError> {
        let increment_interval = rate_limit.increment_interval(cost);

        let compute_tat = |new_tat: Instant| {
            if increment_interval > rate_limit.period {
                return Err(GcraError::DeniedIndefinitely {
                    cost,
                    rate_limit: rate_limit.clone(),
                });
            }

            Ok(new_tat + increment_interval)
        };

        let tat = match self.tat {
            Some(tat) => tat,
            None => {
                // First ever request. Allow passage and update self.
                self.tat = Some(compute_tat(arrived_at)?);
                return Ok(());
            }
        };

        // We had a previous request
        if tat < arrived_at {
            // prev request was really old
            let new_tat = std::cmp::max(tat, arrived_at);
            self.tat = Some(compute_tat(new_tat)?);
            Ok(())
        } else {
            // prev request was recent and there's a possibility that we've reached the limit
            let delay_variation_tolerance = rate_limit.period;
            let new_tat = compute_tat(tat)?;

            let next_allowed_at = new_tat - delay_variation_tolerance;
            if next_allowed_at <= arrived_at {
                self.tat = Some(new_tat);
                Ok(())
            } else {
                // Denied, must wait until next_allowed_at
                Err(GcraError::DeniedUntil { next_allowed_at })
            }
        }
    }

    /// Reverts rate_limit by cost, and updated our internal state.
    ///
    /// Simply passes the current Instant to [`revert_at()`]
    #[inline]
    pub fn revert(&mut self, rate_limit: &RateLimit, cost: u32) -> Result<(), GcraError> {
        let arrived_at = Instant::now();
        self.revert_at(rate_limit, arrived_at, cost)
    }

    /// Reverts rate_limit by cost, and updated our internal state.
    ///
    /// This is a hack that substracts the incremental cost from the TAT.
    pub fn revert_at(
        &mut self,
        rate_limit: &RateLimit,
        arrived_at: Instant,
        cost: u32,
    ) -> Result<(), GcraError> {
        let increment_interval = rate_limit.increment_interval(cost);

        let compute_revert_tat = |new_tat: Instant| new_tat - increment_interval;

        let tat = match self.tat {
            Some(tat) => tat,
            None => {
                // First ever request. Nothing to do.
                return Ok(());
            }
        };

        // We had a previous request
        if tat < arrived_at {
            // Reset state: prev request was really old
            self.tat = None;
        } else {
            // prev request was recent
            self.tat = Some(compute_revert_tat(tat));
        }
        Ok(())
    }

    pub fn remaining_resources(&self, rate_limit: &RateLimit, now: Instant) -> u32 {
        if rate_limit.period.is_zero() {
            return 0;
        }

        let time_to_tat = match self.tat.and_then(|tat| tat.checked_duration_since(now)) {
            Some(duration_until) => duration_until,
            None => return rate_limit.resource_limit,
        };

        // Logically this makes more sense as:
        //   consumed_resources = time_to_tat * (resource_limit/period)
        // but we run it this way because of Duration's arithmetic functions

        // Requires nightly
        // let consumed_resources =
        //     (time_to_tat * rate_limit.resource_limit).div_duration_f32(rate_limit.period);

        let consumed_resources = time_to_tat.as_nanos() as f64
            / rate_limit.period.as_nanos() as f64
            * rate_limit.resource_limit as f64;

        rate_limit.resource_limit - consumed_resources.ceil() as u32
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_rate_limit_unused_counts() {
        let base_tat = Instant::now();
        let rate_limit = RateLimit::new(10, Duration::from_secs(1));

        assert_eq!(
            4,
            GcraState {
                tat: Some(base_tat + Duration::from_millis(550))
            }
            .remaining_resources(&rate_limit, base_tat),
            "Remaining count should ceiled"
        );
        assert_eq!(
            0,
            GcraState {
                tat: Some(base_tat + Duration::from_millis(950))
            }
            .remaining_resources(&rate_limit, base_tat),
            "Remaining count should ceiled, thus preventing any additional requests"
        );

        assert_eq!(
            9,
            GcraState {
                tat: Some(base_tat + Duration::from_millis(100))
            }
            .remaining_resources(&rate_limit, base_tat),
            "Remaining count is based on max_period timeout"
        );
    }

    #[test]
    fn gcra_basics() {
        let mut gcra = GcraState::default();
        let rate_limit = RateLimit::new(1, Duration::from_secs(1));

        let first_req_ts = Instant::now();
        assert_eq!(
            Ok(()),
            gcra.check_and_modify(&rate_limit, 1),
            "request #1 should pass"
        );
        let after_first_tat = gcra.tat;
        assert!(
            after_first_tat.is_some(),
            "state should be modified and have a TAT in the future"
        );

        let next_allowed_ts = match gcra.check_and_modify(&rate_limit, 1) {
            Err(GcraError::DeniedUntil { next_allowed_at }) => next_allowed_at,
            _ => panic!("request #2 should be denied temporarily"),
        };
        assert!(
            next_allowed_ts >= first_req_ts + Duration::from_secs(1),
            "we should only be allowed after the burst period"
        );
        assert_eq!(after_first_tat, gcra.tat, "State should be unchanged.")
    }

    #[test]
    fn gcra_limited() {
        const LIMIT: u32 = 5;
        let mut gcra = GcraState::default();
        let rate_limit = RateLimit::new(LIMIT, Duration::from_secs(1));

        let req_ts = Instant::now();
        for i in 0..LIMIT {
            assert_eq!(
                Ok(()),
                gcra.check_and_modify_at(&rate_limit, req_ts, 1),
                "request #{} should pass",
                i + 1
            );
        }

        assert_eq!(
            Some(req_ts + rate_limit.period),
            gcra.tat,
            "state should be modified and have a TAT for the full period",
        );

        // Trigger another event
        let denied_result = gcra.check_and_modify_at(&rate_limit, req_ts, 1);

        assert_eq!(
            Some(req_ts + rate_limit.period),
            gcra.tat,
            "state should not have changed when at limit",
        );

        assert_eq!(
            Err(GcraError::DeniedUntil {
                next_allowed_at: req_ts + rate_limit.emission_interval
            }),
            denied_result,
            "next request should be denied",
        );
    }

    #[test]
    fn gcra_revert_new() {
        const LIMIT: u32 = 5;
        let mut gcra = GcraState::default();
        let rate_limit = RateLimit::new(LIMIT, Duration::from_secs(1));

        let req_ts = Instant::now();
        // Revert before any calls
        assert!(
            gcra.revert_at(&rate_limit, req_ts, 1).is_ok(),
            "revert should have released resources"
        );
        assert_eq!(None, gcra.tat, "state should not have changed at all",);
    }

    #[test]
    fn gcra_revert_existing() {
        const LIMIT: u32 = 5;
        let mut gcra = GcraState::default();
        let rate_limit = RateLimit::new(LIMIT, Duration::from_secs(1));

        let req_ts = Instant::now();
        assert_eq!(
            Ok(()),
            gcra.check_and_modify_at(&rate_limit, req_ts, 5),
            "use up all resources",
        );

        assert_eq!(
            Some(req_ts + rate_limit.period),
            gcra.tat,
            "state should be modified and have a TAT for the full period",
        );

        // Revert
        assert!(
            gcra.revert_at(&rate_limit, req_ts, 1).is_ok(),
            "revert should have released resources"
        );
        assert_eq!(
            Some(req_ts + rate_limit.period - rate_limit.increment_interval(1)),
            gcra.tat,
            "state should not have changed when at limit",
        );

        // Confirm revert re-enables
        assert_eq!(
            Ok(()),
            gcra.check_and_modify_at(&rate_limit, req_ts, 1),
            "additional resources should have been freed",
        );
    }

    #[test]
    fn gcra_revert_existing_ancient() {
        const LIMIT: u32 = 5;
        let mut gcra = GcraState::default();
        let rate_limit = RateLimit::new(LIMIT, Duration::from_secs(1));

        let past_req_ts = Instant::now() - Duration::from_secs(100);
        assert_eq!(
            Ok(()),
            gcra.check_and_modify_at(&rate_limit, past_req_ts, 5),
            "use up all resources, but in distant past",
        );
        assert_eq!(
            Some(past_req_ts + rate_limit.period),
            gcra.tat,
            "state should be modified and have a TAT for the past",
        );

        // Revert using current time
        let req_ts = Instant::now();
        assert!(
            gcra.revert_at(&rate_limit, req_ts, 1).is_ok(),
            "revert should have released resources"
        );
        assert_eq!(
            None, gcra.tat,
            "state should have reset since it was so old",
        );

        // Confirm revert had 0 effect
        assert_eq!(
            Ok(()),
            gcra.check_and_modify_at(&rate_limit, req_ts, 1),
            "additional resources should have been freed",
        );
        assert_eq!(
            gcra.tat,
            Some(req_ts + rate_limit.increment_interval(1)),
            "new TAT state should have been moved forward according to cost like normal"
        );
    }

    #[test]
    fn gcra_leaky() {
        // const INCREMENT_INTERVAL: u64 = 500;
        const INCREMENT_INTERVAL: Duration = Duration::from_millis(500);

        let mut gcra = GcraState::default();
        let rate_limit = RateLimit::new(10, 10 * INCREMENT_INTERVAL);
        assert_eq!(INCREMENT_INTERVAL, rate_limit.emission_interval);

        let arrived_at = Instant::now();
        assert_eq!(
            Ok(()),
            gcra.check_and_modify_at(&rate_limit, arrived_at, 1),
            "request #1 should pass"
        );
        assert_eq!(
            gcra.tat,
            Some(arrived_at + INCREMENT_INTERVAL),
            "new TAT state should have been moved forward according to cost"
        );

        assert_eq!(
            Ok(()),
            gcra.check_and_modify(&rate_limit, 9),
            "request #2 should consume all remaining resources and pass"
        );
        assert!(
            matches!(gcra.check_and_modify(&rate_limit, 1), Err(_allowed_at)),
            "request #3 should fail since all resources consumed"
        );

        let current_tat = gcra.tat.expect("should have a tat state after use");
        assert!(current_tat > Instant::now(), "tat in the future");

        assert!(
            matches!(
                // manually force time check that we know will fail
                gcra.check_and_modify_at(
                    &rate_limit,
                    current_tat - rate_limit.period - Duration::from_millis(1),
                    1
                ),
                Err(_allowed_at)
            ),
            "request #4 before leak period should fail. INCREMENT_INTERVAL has not passed yet."
        );

        assert!(
            matches!(
                gcra.check_and_modify_at(&rate_limit, current_tat - rate_limit.period, 1),
                Err(_allowed_at)
            ),
            "request #5 after leak period should pass. INCREMENT_INTERVAL has passed"
        );
    }

    #[test]
    fn gcra_cost_indefinitely_denied() {
        let mut gcra = GcraState::default();
        let rate_limit = RateLimit::new(5, Duration::from_secs(1));

        assert_eq!(
            Ok(()),
            gcra.check_and_modify(&rate_limit, 1),
            "request #1 should pass"
        );

        let over_limit_cost = rate_limit.resource_limit + 1;
        match gcra.check_and_modify(&rate_limit, over_limit_cost) {
            Err(GcraError::DeniedIndefinitely {
                cost,
                rate_limit: rl,
            }) => {
                assert_eq!(over_limit_cost, cost);
                assert_eq!(rate_limit, rl);
            }
            e => panic!("request #2 would never succeed {:?}", e),
        };
    }

    #[test]
    fn gcra_cost_temporarily_denied() {
        let mut gcra = GcraState::default();
        let rate_limit = RateLimit::new(5, Duration::from_secs(1));

        let first_req_ts = Instant::now();
        assert_eq!(
            Ok(()),
            gcra.check_and_modify(&rate_limit, 1),
            "request #1 should pass"
        );

        let after_first_tat = gcra.tat;
        assert!(
            after_first_tat.is_some(),
            "state should be modified and have a TAT in the future"
        );

        let next_allowed_ts = match gcra.check_and_modify(&rate_limit, rate_limit.resource_limit) {
            Err(GcraError::DeniedUntil { next_allowed_at }) => next_allowed_at,
            _ => panic!("request #2 is only temporarily denied"),
        };

        assert!(
            next_allowed_ts >= first_req_ts + rate_limit.increment_interval(1),
            "we should only be allowed after the burst period {:?} >= {:?}",
            next_allowed_ts,
            first_req_ts + rate_limit.period
        );
        assert_eq!(after_first_tat, gcra.tat, "State should be unchanged.")
    }

    #[test]
    fn gcra_refreshed_after_period() {
        let past_time = Instant::now() - Duration::from_millis(1001);
        let mut gcra = GcraState {
            tat: Some(past_time),
        };
        let rate_limit = RateLimit::new(1, Duration::from_secs(1));
        assert_eq!(
            Ok(()),
            gcra.check_and_modify(&rate_limit, 1),
            "request #1 should pass"
        );

        assert!(
            matches!(gcra.check_and_modify(&rate_limit, 1), Err(_allowed_at)),
            "request #2 should fail"
        );
    }
}
