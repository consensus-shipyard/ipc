use futures::{FutureExt};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

pub async fn execute<F>(cfg: Config, test: F) -> (ExecutionSummary, Vec<ExecutionResult>)
where
    F: Fn(usize) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>,
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
        let task = test(test_id).boxed();
        let results = results.clone();
        let handle = tokio::spawn(async move {
            let start = Instant::now();
            let result = task.await;
            let duration = start.elapsed();

            results.lock().await.push(ExecutionResult {
                test_id,
                duration,
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
    pub duration: Duration,
    pub err: Option<anyhow::Error>,
}

#[derive(Debug)]
pub struct ExecutionSummary {
    pub cfg: Config,
    pub avg_duration: Duration,
    pub num_failures: usize,
}

impl ExecutionSummary {
    fn new(cfg: Config, results: &Vec<ExecutionResult>) -> Self {
        let total_duration: Duration = results.iter().map(|res| res.duration).sum();
        let avg_duration = total_duration / results.len() as u32;
        let num_failures = results.iter().filter(|res| res.err.is_some()).count();

        Self {
            cfg,
            avg_duration,
            num_failures,
        }
    }
}
