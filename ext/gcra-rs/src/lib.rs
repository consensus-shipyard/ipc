//! Library which implements the core
//! [GCRA](https://en.wikipedia.org/wiki/Generic_cell_rate_algorithm) functionality in rust.
//!
//! # Features
//! - `rate-limiter` a LRU + expiring rate limiter. Implements `Send + Sync` so
//!   can be used asynchronously.
//!
//! # Usage
//!
//! ```rust
//! use gcra::{GcraState, RateLimit};
//!
//! fn check_rate_limit() {
//!   const LIMIT: u32 = 1;
//!   // Create a rate limit that allows `1/1s`
//!   let rate_limit = RateLimit::per_sec(LIMIT);
//!
//!   let mut user_state = GcraState::default();
//!   assert!(user_state.check_and_modify(&rate_limit, 1).is_ok());
//!   assert!(
//!       user_state.check_and_modify(&rate_limit, 1).is_err(),
//!       "We should be over the limit now"
//!   );
//! }
//! ```
//!
//! ## With `rate-limiter`
//!
//! ```rust
//! use gcra::{GcraError, RateLimit, RateLimiter};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), GcraError> {
//!     let rate_limit = RateLimit::per_sec(2);
//!     let mut rl = RateLimiter::new(4);
//!
//!     rl.check("key", rate_limit.clone(), 1).await?;
//!     rl.check("key", rate_limit.clone(), 1).await?;
//!
//!     match rl.check("key", rate_limit.clone(), 1).await {
//!         Err(GcraError::DeniedUntil { next_allowed_at }) => {
//!             print!("Denied: Request next at {:?}", next_allowed_at);
//!             Ok(())
//!         }
//!         unexpected => panic!("Opps something went wrong! {:?}", unexpected),
//!     }
//! }
//! ```

mod gcra;
mod rate_limit;
#[cfg(feature = "rate-limiter")]
mod rate_limiter;

pub use crate::gcra::{GcraError, GcraState};
pub use crate::rate_limit::RateLimit;
#[cfg(feature = "rate-limiter")]
pub use crate::rate_limiter::{RateLimitEntry, RateLimitRequest, RateLimiter};
