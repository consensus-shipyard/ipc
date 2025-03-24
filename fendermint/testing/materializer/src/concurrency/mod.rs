// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod collect;
pub mod config;
pub mod nonce_manager;
pub mod reporting;

use crate::bencher::Bencher;
use crate::concurrency::reporting::TestResult;
use anyhow::anyhow;
use ethers::types::H256;
use futures::stream::FuturesUnordered;
use futures::FutureExt;
use futures::StreamExt;
use std::future::Future;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use tokio::time::timeout;

#[derive(Debug)]
pub struct TestInput {
    pub test_id: usize,
    pub bencher: Bencher,
    pub token: CancellationToken,
}

#[derive(Debug)]
pub struct TestOutput {
    pub bencher: Bencher,
    pub tx_hash: H256,
}

use tokio_util::sync::CancellationToken;

pub async fn run_concurrent<F, Fut>(cfg: config::Execution, test_factory: F) -> Vec<Vec<TestResult>>
where
    F: Fn(TestInput) -> Fut,
    Fut: Future<Output = anyhow::Result<TestOutput>> + Send + 'static,
{
    let mut test_id = 0;
    let mut results = Vec::new();
    for (step_id, step) in cfg.steps.iter().enumerate() {
        let semaphore = Arc::new(Semaphore::new(step.max_concurrency));
        let mut tasks = FuturesUnordered::new();
        let execution_start = Instant::now();

        while execution_start.elapsed() < step.duration {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let bencher = Bencher::new();
            let token = CancellationToken::new();
            let token_clone = token.clone();
            let test_input = TestInput {
                test_id,
                bencher,
                token: token_clone,
            };
            let task = test_factory(test_input).boxed();

            tasks.push(tokio::spawn(async move {
                let timeout_result = match cfg.timeout {
                    Some(timeout_duration) => timeout(timeout_duration, task).await,
                    None => Ok(task.await),
                };

                drop(permit);

                let (bencher, tx_hash, err) = match timeout_result {
                    Err(_) => {
                        token.cancel();
                        (None, None, Some(anyhow!("test timed out")))
                    }
                    Ok(test_result) => match test_result {
                        Err(err) => (None, None, Some(err)),
                        Ok(test_output) => {
                            (Some(test_output.bencher), Some(test_output.tx_hash), None)
                        }
                    },
                };

                TestResult {
                    test_id,
                    step_id,
                    bencher,
                    tx_hash,
                    err,
                }
            }));
            test_id += 1;
        }

        // Collect results as tasks complete (unordered).
        let mut step_results = Vec::new();
        while let Some(Ok(result)) = tasks.next().await {
            step_results.push(result);
        }
        results.push(step_results);
    }
    results
}
