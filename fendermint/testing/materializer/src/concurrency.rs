use std::collections::HashMap;
use futures::{FutureExt};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore};
use crate::bencher::Bencher;

pub async fn execute<F>(cfg: Config, test: F) -> (ExecutionSummary, Vec<ExecutionResult>)
where
    F: Fn(usize, Arc<Bencher>) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>,
{
    let semaphore = Arc::new(Semaphore::new(cfg.max_concurrency));
    let mut test_id = 0;
    let mut handles = Vec::new();
    let results = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let execution_start = Instant::now();
    loop {
        if execution_start.elapsed() > cfg.duration {
            break;
        }
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let bencher = Arc::new(Bencher::new());
        let task = test(test_id, bencher.clone()).boxed();
        let results = results.clone();
        let handle = tokio::spawn(async move {
            let result = task.await;
            let records = bencher.records.lock().await.clone();
            results.lock().await.push(ExecutionResult {
                test_id,
                records,
                err: result.err(),
            });
            drop(permit);
        });
        handles.push(handle);
        test_id = test_id + 1;
    }

    // Exhaust unfinished handles.
    for handle in handles {
        handle.await.unwrap();
    }

    let results = Arc::try_unwrap(results).unwrap().into_inner();
    let summary = ExecutionSummary::new(cfg, &results);
    (summary, results)
}

#[derive(Debug)]
pub struct Config {
    pub max_concurrency: usize,
    pub duration: Duration,
}

impl Config {
    pub fn new() -> Self {
        Self {
            max_concurrency: 1,
            duration: Duration::from_secs(1),
        }
    }

    pub fn with_max_concurrency(mut self, max_concurrency: usize) -> Self {
        self.max_concurrency = max_concurrency;
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}
#[derive(Debug)]
pub struct ExecutionResult {
    pub test_id: usize,
    pub records: HashMap<String, Duration>,
    pub err: Option<anyhow::Error>,
}

#[derive(Debug)]
pub struct ExecutionSummary {
    pub cfg: Config,
    pub avg_latencies: HashMap<String, Duration>,
    pub num_failures: usize,
}

impl ExecutionSummary {
    fn new(cfg: Config, results: &Vec<ExecutionResult>) -> Self {
        let num_failures = results.iter().filter(|res| res.err.is_some()).count();

        let mut total_durations: HashMap<String, Duration> = HashMap::new();
        let mut counts: HashMap<String, usize> = HashMap::new();
        for res in results {
            for (key, duration) in res.records.clone() {
                *total_durations.entry(key.clone()).or_insert(Duration::ZERO) += duration;
                *counts.entry(key).or_insert(0) += 1;
            }
        }

        let avg_latencies = total_durations
            .into_iter()
            .map(|(key, total)| {
                let count = counts[&key];
                (key, total / count as u32)
            })
            .collect();

        Self {
            cfg,
            avg_latencies,
            num_failures,
        }
    }
}

