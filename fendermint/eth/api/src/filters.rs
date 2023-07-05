// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use ethers_core::types as et;

/// Check whether to keep a log according to the topic filter.
pub fn matches_topics(filter: &et::Filter, log: &et::Log) -> bool {
    for i in 0..4 {
        if let Some(topics) = &filter.topics[i] {
            let topic = log.topics.get(i);
            let matches = match topics {
                et::ValueOrArray::Value(Some(t)) => topic == Some(t),
                et::ValueOrArray::Array(ts) => ts.iter().flatten().any(|t| topic == Some(t)),
                _ => true,
            };
            if !matches {
                return false;
            }
        }
    }
    true
}
