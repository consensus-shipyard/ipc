use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Defines the configuration for a GCRA rate limit.
pub struct RateLimit {
    // Amount of resources that are allowed in a given period.
    pub resource_limit: u32,
    // The length of which to allow access to the resource.
    pub period: Duration,

    /// Incremental duration cost of a single resource check
    pub emission_interval: Duration,
}

impl RateLimit {
    pub fn new(resource_limit: u32, period: Duration) -> Self {
        let emission_interval = period / resource_limit;
        Self {
            resource_limit,
            period,
            emission_interval,
        }
    }

    #[inline]
    pub fn per_sec(resource_limit: u32) -> Self {
        Self::new(resource_limit, Duration::from_secs(1))
    }

    /// Given a `cost`, calculates the increment interval.
    pub fn increment_interval(&self, cost: u32) -> Duration {
        self.emission_interval * cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_emission_interal() {
        let rate_limit = RateLimit::new(10, Duration::from_secs(20));
        assert_eq!(Duration::from_secs(2), rate_limit.emission_interval)
    }
}
