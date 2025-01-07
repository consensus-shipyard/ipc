// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod config;
pub mod nonce_manager;
pub mod reporting;

pub use reporting::*;

use crate::bencher::Bencher;
use futures::FutureExt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

pub async fn execute<F>(cfg: config::Execution, test_factory: F) -> Vec<Vec<TestResult>>
where
    F: Fn(usize, Bencher) -> Pin<Box<dyn Future<Output = anyhow::Result<Bencher>> + Send>>,
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
            let bencher = Bencher::new();
            let task = test_factory(test_id, bencher).boxed();
            let step_results = step_results.clone();
            let handle = tokio::spawn(async move {
                let res = task.await;
                let (bencher, err) = match res {
                    Ok(bencher) => (Some(bencher), None),
                    Err(err) => (None, Some(err)),
                };
                step_results.lock().await.push(TestResult {
                    test_id,
                    step_id,
                    bencher,
                    err,
                });
                drop(permit);
            });
            handles.push(handle);
            test_id += 1;
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
