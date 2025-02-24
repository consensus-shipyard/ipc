// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use super::{
    dataset::{Metrics},
    tps::calc_tps,
    TestResult,
};
use crate::concurrency::config;
use crate::concurrency::config::ExecutionStep;
use anyhow::anyhow;
use ethers::prelude::{Block, H256};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ExecutionSummary {
    pub summaries: Vec<StepSummary>,
}

impl ExecutionSummary {
    pub fn new(
        cfg: config::Execution,
        blocks: HashMap<u64, Block<H256>>,
        results: Vec<Vec<TestResult>>,
    ) -> Self {
        let step_tx_hashes = Self::extract_tx_hashes(&results);
        let step_blocks = Self::map_blocks_to_steps(blocks, step_tx_hashes);

        let mut summaries = Vec::new();
        for (i, results) in results.into_iter().enumerate() {
            let cfg = cfg.steps[i].clone();
            let blocks = step_blocks[i].clone();
            summaries.push(StepSummary::new(cfg, results, blocks));
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
            let cloned_errs: Vec<String> =
                summary.errs.iter().map(|e| format!("{:?}", e)).collect();
            errs.extend(cloned_errs);
        }
        errs
    }

    /// Extracts transaction hashes from test results and groups them by steps.
    fn extract_tx_hashes(results: &[Vec<TestResult>]) -> Vec<Vec<H256>> {
        results
            .iter()
            .map(|step_results| {
                step_results
                    .iter()
                    .filter_map(|result| result.tx_hash)
                    .collect()
            })
            .collect()
    }

    /// Maps blocks to their corresponding steps based on the transactions they contain.
    pub fn map_blocks_to_steps(
        blocks: HashMap<u64, Block<H256>>,
        step_tx_hashes: Vec<Vec<H256>>,
    ) -> Vec<Vec<Block<H256>>> {
        let mut sorted_blocks: Vec<_> = blocks.into_iter().collect();
        sorted_blocks.sort_by_key(|(block_number, _)| *block_number);

        let mut step_mapped_blocks: Vec<Vec<Block<H256>>> = vec![Vec::new(); step_tx_hashes.len()];

        for (_, block) in sorted_blocks {
            // Determine the max step_id based on the transactions in the block
            let latest_step_id = block
                .transactions
                .iter()
                .filter_map(|tx_hash| {
                    step_tx_hashes.iter().enumerate().find_map(|(step_id, tx_hashes)| {
                        if tx_hashes.contains(tx_hash) {
                            Some(step_id)
                        } else {
                            None
                        }
                    })
                })
                .max();

            if let Some(step_id) = latest_step_id {
                // Add the block to the corresponding step_id.
                step_mapped_blocks[step_id].push(block);
            }
        }

        step_mapped_blocks
    }
}

impl Display for ExecutionSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut data = vec![];
        let latencies: HashSet<String> = self
            .summaries
            .iter()
            .flat_map(|summary| summary.latencies.keys().cloned())
            .collect();

        let mut header = vec![
            "max_concurrency".to_string(),
            "duration".to_string(),
            "TPS".to_string(),
        ];
        header.extend(latencies.iter().map(|key| format!("latency ({}) ", key)));
        data.push(header);

        for summary in self.summaries.iter() {
            let mut row = vec![];
            row.push(summary.cfg.max_concurrency.to_string());
            row.push(summary.cfg.duration.as_secs().to_string());
            row.push(format!("median: {:.2}", summary.tps.median));

            for key in &latencies {
                let latency = summary
                    .latencies
                    .get(key)
                    .map_or(String::from("-"), |metrics| format!("median: {:.2}s", metrics.median));
                row.push(latency);
            }

            data.push(row);
        }

        let mut output = Vec::new();
        text_tables::render(&mut output, data).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        write!(f, "{}", output_str)
    }
}

#[derive(Debug)]
pub struct StepSummary {
    pub cfg: ExecutionStep,
    pub latencies: HashMap<String, Metrics>,
    pub tps: Metrics,
    pub errs: Vec<anyhow::Error>,
}

impl StepSummary {
    fn new(cfg: ExecutionStep, results: Vec<TestResult>, blocks: Vec<Block<H256>>) -> Self {
        let mut latencies: HashMap<String, Vec<f64>> = HashMap::new();
        let mut errs = Vec::new();

        for res in results {
            if let Some(err) = res.err {
                errs.push(err);
            }

            let Some(bencher) = res.bencher else { continue };

            for (key, duration) in bencher.latencies.clone() {
                latencies
                    .entry(key.clone())
                    .or_default()
                    .push(duration.as_secs_f64());
            }
        }

        let latencies = latencies
            .into_iter()
            .map(|(key, dataset)| (key, dataset.into()))
            .collect();

        let tps: Metrics = calc_tps(blocks).into();

        Self {
            cfg,
            latencies,
            tps,
            errs,
        }
    }
}
