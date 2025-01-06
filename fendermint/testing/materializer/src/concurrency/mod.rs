pub mod config;
pub mod reporting;
pub mod nonce_manager;

pub use reporting::*;

use futures::{FutureExt};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use ethers::types::spoof::nonce;
use tokio::sync::{Semaphore};
use crate::bencher::Bencher;
use crate::concurrency::nonce_manager::NonceManager;

pub async fn execute<F>(cfg: config::Execution, test: F) -> Vec<Vec<TestResult>>
where
    F: Fn(usize, Arc<Bencher>) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>,
{
    let mut test_id = 0;
    let mut results = Vec::new();
    for (step_id, step) in cfg.steps.iter().enumerate() {
        let semaphore = Arc::new(Semaphore::new(step.max_concurrency));
        let mut handles = Vec::new();
        let step_results = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let execution_start = Instant::now();
        loop {
            if execution_start.elapsed() > step.duration {
                break;
            }
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let bencher = Arc::new(Bencher::new());
            let task = test(test_id, bencher.clone()).boxed();
            let step_results = step_results.clone();
            let handle = tokio::spawn(async move {
                let result = task.await;
                let records = bencher.records.lock().await.clone();
                step_results.lock().await.push(TestResult {
                    test_id,
                    step_id,
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

        let step_results = Arc::try_unwrap(step_results).unwrap().into_inner();
        results.push(step_results)
    }
    results
}

