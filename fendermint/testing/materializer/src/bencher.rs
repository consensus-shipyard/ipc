use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// TODO: remove the Arc<Mutex<T>> wrappers.
// they are not needed, as a Bencher instance shouldn't be shared among tests.
// it's just a workaround to issues involving mutable references lifetime in futures.
#[derive(Debug, Clone)]
pub struct Bencher {
    pub start_time: Arc<Mutex<Option<Instant>>>,
    pub records: Arc<Mutex<HashMap<String, Duration>>>,
}

impl Bencher {
    pub fn new() -> Self {
        Self {
            start_time: Arc::new(Mutex::new(None)),
            records: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&self) {
        let mut start_time = self.start_time.lock().await;
        *start_time = Some(Instant::now());
    }

    pub async fn record(&self, label: String) {
        let start_time = self.start_time.lock().await;
        let duration = start_time.unwrap().elapsed();
        let mut records = self.records.lock().await;
        records.insert(label, duration);
    }
}
