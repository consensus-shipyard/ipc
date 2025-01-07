use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Bencher {
    pub start_time: Option<Instant>,
    pub records: HashMap<String, Duration>,
}

impl Bencher {
    pub fn new() -> Self {
        Self {
            start_time: None,
            records: HashMap::new(),
        }
    }

    pub async fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub async fn record(&mut self, label: String) {
        let duration = self.start_time.unwrap().elapsed();
        self.records.insert(label, duration);
    }
}
