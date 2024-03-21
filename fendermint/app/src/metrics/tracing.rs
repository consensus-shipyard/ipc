// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Subscribing to tracing events and turning them into metrics.

use std::marker::PhantomData;

use prometheus;
use tracing::{Event, Subscriber};
use tracing_subscriber::{filter, layer, registry::LookupSpan, Layer};

use super::prometheus as pm;

pub fn layer<S>() -> impl Layer<S>
where
    S: Subscriber,
    for<'a> S: LookupSpan<'a>,
{
    MetricsLayer::new().with_filter(filter::filter_fn(|md| md.name().starts_with("event::")))
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
                set_block_height(event, &pm::TOPDOWN_VIEW_BLOCK_HEIGHT);
            }
            "event::ParentFinalityCommitted" => {
                set_block_height(event, &pm::TOPDOWN_FINALIZED_BLOCK_HEIGHT);
            }
            _ => {}
        }
    }
}

fn set_block_height(event: &Event<'_>, gauge: &prometheus::IntGauge) {
    let mut block_height = visitors::block_height();
    event.record(&mut block_height);
    gauge.set(block_height.value as i64);
}

mod visitors {
    use tracing::field::{Field, Visit};

    pub fn block_height() -> FindU64 {
        FindU64::new("block_height")
    }

    pub struct FindU64 {
        pub name: &'static str,
        pub value: u64,
    }

    impl FindU64 {
        pub fn new(name: &'static str) -> Self {
            Self { name, value: 0 }
        }
    }

    impl Visit for FindU64 {
        fn record_u64(&mut self, field: &Field, value: u64) {
            if field.name() == self.name {
                self.value = value;
            }
        }

        fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}
    }
}
