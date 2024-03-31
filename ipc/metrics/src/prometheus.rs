// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Prometheus metrics


/// Metrics emitted by ipc.
pub mod app {
    use lazy_static::lazy_static;
    use paste::paste;
    use prometheus::{IntCounter, IntGauge, Registry};

    metrics_utils::metrics! {
        LATEST_ACCEPTED_CHECKPOINT_HEIGHT: IntGauge = "Block height of latest accepted bottom-up checkpoint";
        CHECKPOINT_SUBMIT_COUNT: IntCounter = "Number of checkpoint submission";
        CHECKPOINT_SUBMIT_FAIL_COUNT: IntCounter = "Number of checkpoint submission failures";
    }
}
