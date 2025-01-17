use std::time::Instant;

/// Abstraction for getting time.
pub trait Clock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstantClock;
impl Clock for InstantClock {}

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::{
        sync::{Arc, Mutex},
        time::{Duration, Instant},
    };

    #[derive(Debug, Clone)]
    pub struct FakeClock {
        now: Instant,
        delta: Arc<Mutex<Duration>>,
    }

    impl Clock for FakeClock {
        fn now(&self) -> Instant {
            self.now + *self.delta.lock().unwrap()
        }
    }

    impl FakeClock {
        pub fn new() -> Self {
            Self {
                now: Instant::now(),
                delta: Arc::new(Mutex::new(Duration::default())),
            }
        }

        pub fn advance_by(&self, duration: Duration) {
            let mut delta = self.delta.lock().unwrap();
            *delta += duration;
        }
    }
}
