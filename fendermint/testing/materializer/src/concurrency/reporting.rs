use crate::bencher::Bencher;
use crate::concurrency::config;
use crate::concurrency::config::ExecutionStep;
use anyhow::anyhow;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug)]
pub struct TestResult {
    pub test_id: usize,
    pub step_id: usize,
    pub bencher: Option<Bencher>,
    pub err: Option<anyhow::Error>,
}

#[derive(Debug)]
pub struct StepSummary {
    pub cfg: ExecutionStep,
    pub avg_latencies: HashMap<String, Duration>,
    pub errs: Vec<anyhow::Error>,
}

impl StepSummary {
    fn new(cfg: ExecutionStep, results: Vec<TestResult>) -> Self {
        let mut total_durations: HashMap<String, Duration> = HashMap::new();
        let mut counts: HashMap<String, usize> = HashMap::new();
        let mut errs = Vec::new();
        for res in results {
            let Some(bencher) = res.bencher else { continue };
            for (key, duration) in bencher.records.clone() {
                *total_durations.entry(key.clone()).or_insert(Duration::ZERO) += duration;
                *counts.entry(key).or_insert(0) += 1;
            }
            if let Some(err) = res.err {
                errs.push(err);
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
            errs,
        }
    }
}

#[derive(Debug)]
pub struct ExecutionSummary {
    pub summaries: Vec<StepSummary>,
}

impl ExecutionSummary {
    pub fn new(cfg: config::Execution, results: Vec<Vec<TestResult>>) -> Self {
        let mut summaries = Vec::new();
        for (i, step_results) in results.into_iter().enumerate() {
            let cfg = cfg.steps[i].clone();
            summaries.push(StepSummary::new(cfg, step_results));
        }

        Self { summaries }
    }

    pub fn to_result(&self) -> anyhow::Result<()> {
        let errs = self.errs();
        if errs.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(errs.join("\n")))
        }
    }

    pub fn errs(&self) -> Vec<String> {
        let mut errs = Vec::new();
        for summary in self.summaries.iter() {
            let cloned_errs: Vec<String> = summary
                .errs
                .iter()
                .map(|e| format!("{:?}", e))
                // e.chain().map(|cause|cause.to_string()).collect::<Vec<_>>().join(" -> "))
                .collect();
            errs.extend(cloned_errs);
        }
        errs
    }
}
