// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct Execution {
    pub steps: Vec<ExecutionStep>,
    pub timeout: Option<Duration>,
}

impl Execution {
    pub fn new_baseline() -> Self {
        Execution::default()
            .with_timeout(Duration::from_secs(10))
            .add_step(1, 5)
            .add_step(10, 5)
            .add_step(50, 5)
            .add_step(100, 5)
            .add_step(200, 5)
            .add_step(300, 5)
            .add_step(400, 5)
            .add_step(500, 5)
    }

    pub fn add_step(mut self, max_concurrency: usize, secs: u64) -> Self {
        self.steps.push(ExecutionStep {
            max_concurrency,
            duration: Duration::from_secs(secs),
        });
        self
    }

    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub max_concurrency: usize,
    pub duration: Duration,
}
