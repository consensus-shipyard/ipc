// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use statrs::statistics::{Data, Distribution, Max, Min, OrderStatistics};
use std::fmt::{Display, Formatter};

#[derive(Debug, Default)]
pub struct Metrics {
    pub mean: f64,
    pub median: f64,
    pub max: f64,
    pub min: f64,
    pub percentile_90: f64,
}

impl From<Vec<f64>> for Metrics {
    fn from(mut data: Vec<f64>) -> Self {
        if data.is_empty() {
            return Metrics::default();
        }

        data.sort_by(|a, b| a.partial_cmp(b).unwrap()); // Sort once before using Data
        let mut data = Data::new(data);

        Metrics {
            mean: data.mean().unwrap(),
            median: data.median(),
            max: data.max(),
            min: data.min(),
            percentile_90: data.percentile(90),
        }
    }
}

impl Display for Metrics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "mean: {:.2}, median: {:.2}, max: {:.2}, min: {:.2}, 90th: {:.2}",
            self.mean, self.median, self.max, self.min, self.percentile_90
        )
    }
}
