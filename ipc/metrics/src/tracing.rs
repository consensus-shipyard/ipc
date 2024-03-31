// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Subscribing to tracing events and turning them into metrics.

use std::marker::PhantomData;

use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{filter, layer, registry::LookupSpan, Layer};

use crate::prometheus::app::{
    CHECKPOINT_SUBMIT_COUNT, CHECKPOINT_SUBMIT_FAIL_COUNT, LATEST_ACCEPTED_CHECKPOINT_HEIGHT,
};
use ipc_event::*;
use metrics_utils::tracing::visitors;
use metrics_utils::{check_field, event_mapping, event_match, event_name, inc_counter, set_gauge};

/// Create a layer that handles events by incrementing metrics.
pub fn layer<S>() -> impl Layer<S>
where
    S: Subscriber,
    for<'a> S: LookupSpan<'a>,
{
    MetricsLayer::new().with_filter(filter::filter_fn(|md| {
        md.level() == &Level::INFO && md.name().starts_with("event::")
    }))
}

struct MetricsLayer<S> {
    _subscriber: PhantomData<S>,
}

impl<S> MetricsLayer<S> {
    pub fn new() -> Self {
        Self {
            _subscriber: PhantomData,
        }
    }
}

impl<S: Subscriber> Layer<S> for MetricsLayer<S> {
    fn on_event(&self, event: &Event<'_>, _ctx: layer::Context<'_, S>) {
        event_match!(event {
            GetLatestAcceptedCheckpoint {
                block_height              => set_gauge   ! &LATEST_ACCEPTED_CHECKPOINT_HEIGHT,
            },
            SubmitBottomUpCheckpoint {
                checkpoint_count          => inc_counter  ! &CHECKPOINT_SUBMIT_COUNT,
            },
            SubmitBottomUpCheckpointFail {
                checkpoint_count          => inc_counter  ! &CHECKPOINT_SUBMIT_FAIL_COUNT,
            }
        });
    }
}
