// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Subscribing to tracing events and turning them into metrics.

use std::marker::PhantomData;

use prometheus;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{filter, layer, registry::LookupSpan, Layer};

use super::prometheus::app as am;

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
        match event.metadata().name() {
            "event::NewParentView" => {
                set_block_height(event, &am::TOPDOWN_VIEW_BLOCK_HEIGHT);
                inc_num_msgs(event, &am::TOPDOWN_VIEW_NUM_MSGS);
                inc_counter(
                    event,
                    &am::TOPDOWN_VIEW_NUM_VAL_CHNGS,
                    "num_validator_changes",
                );
            }
            "event::ParentFinalityCommitted" => {
                set_block_height(event, &am::TOPDOWN_FINALIZED_BLOCK_HEIGHT);
            }
            "event::NewBottomUpCheckpoint" => {
                set_block_height(event, &am::BOTTOMUP_CKPT_BLOCK_HEIGHT);
                inc_num_msgs(event, &am::BOTTOMUP_CKPT_NUM_MSGS);
                set_gauge(
                    event,
                    &am::BOTTOMUP_CKPT_CONFIG_NUM,
                    "next_configuration_number",
                );
            }
            _ => {}
        }
    }
}

fn set_block_height(event: &Event<'_>, gauge: &prometheus::IntGauge) {
    set_gauge(event, gauge, "block_height")
}

fn set_gauge(event: &Event<'_>, gauge: &prometheus::IntGauge, name: &str) {
    let mut block_height = visitors::FindU64::new(name);
    event.record(&mut block_height);
    gauge.set(block_height.value as i64);
}

fn inc_num_msgs(event: &Event<'_>, counter: &prometheus::IntCounter) {
    inc_counter(event, counter, "num_msgs")
}

fn inc_counter(event: &Event<'_>, counter: &prometheus::IntCounter, name: &str) {
    let mut num_msgs = visitors::FindU64::new(name);
    event.record(&mut num_msgs);
    counter.inc_by(num_msgs.value);
}

mod visitors {
    use tracing::field::{Field, Visit};

    pub struct FindU64<'a> {
        pub name: &'a str,
        pub value: u64,
    }

    impl<'a> FindU64<'a> {
        pub fn new(name: &'a str) -> Self {
            Self { name, value: 0 }
        }
    }

    impl<'a> Visit for FindU64<'a> {
        fn record_u64(&mut self, field: &Field, value: u64) {
            if field.name() == self.name {
                self.value = value;
            }
        }

        fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}
    }
}
