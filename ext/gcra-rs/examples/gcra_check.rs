use std::time::Instant;

use chrono::{DateTime, Duration, Utc};
use gcra::{GcraError, GcraState, RateLimit};

fn check_rate_limit(rate_limit: &RateLimit, gcra_state: &mut GcraState) -> bool {
    const COST: u32 = 1;
    match gcra_state.check_and_modify(rate_limit, COST) {
        Ok(_) => {
            println!("allowed");
            true
        }
        Err(GcraError::DeniedUntil { next_allowed_at }) => {
            println!("denied. Try again at {:?}", to_date_time(next_allowed_at));
            false
        }

        Err(error) => {
            println!("denied: {:?}", error);
            false
        }
    }
}

fn to_date_time(instant: Instant) -> DateTime<Utc> {
    let diff = instant - Instant::now();
    Utc::now() + Duration::from_std(diff).unwrap()
}

fn main() {
    const LIMIT: u32 = 3;
    // Create a rate limit that allows `3/1s`
    let rate_limit = RateLimit::per_sec(LIMIT);

    let mut user_state = GcraState::default();
    for i in 0..LIMIT {
        assert!(
            check_rate_limit(&rate_limit, &mut user_state),
            "Attempt #{} should be allowed",
            i + 1
        );
    }
    assert!(
        !check_rate_limit(&rate_limit, &mut user_state),
        "We should be over the limit now"
    );
}
