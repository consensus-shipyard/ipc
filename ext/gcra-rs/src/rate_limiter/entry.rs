use std::{
    ops::{Deref, DerefMut},
    time::Instant,
};

use crate::{GcraState, RateLimit};

#[derive(Default, Debug, Clone)]
pub struct RateLimitEntry {
    pub gcra_state: GcraState,
    pub expires_at: Option<Instant>,
}

impl Deref for RateLimitEntry {
    type Target = GcraState;

    fn deref(&self) -> &Self::Target {
        &self.gcra_state
    }
}

impl DerefMut for RateLimitEntry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.gcra_state
    }
}

impl RateLimitEntry {
    pub(super) fn update_expiration(&mut self, rate_limit: &RateLimit) {
        let expires_at = self.tat.unwrap_or_else(Instant::now) + rate_limit.period;
        self.expires_at = Some(expires_at);
    }
}
