// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Subscribing to tracing events and turning them into metrics.

use std::marker::PhantomData;

use tracing::{Event, Subscriber};
use tracing_subscriber::{filter, layer, registry::LookupSpan, Layer};

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
                let mut block_height = visitors::block_height();
                event.record(&mut block_height);
                todo!("increment gauge");
            }
            _ => {}
        }
    }
}

mod visitors {
    use tracing::field::{Field, Visit};

    pub fn block_height() -> FindU64 {
        FindU64::new("block_height")
    }

    pub struct FindU64 {
        name: &'static str,
        value: u64,
    }

    impl FindU64 {
        pub fn new(name: &'static str) -> Self {
            Self { name, value: 0 }
        }
    }

    impl<'a> Visit for FindU64 {
        fn record_u64(&mut self, field: &Field, value: u64) {
            if field.name() == self.name {
                self.value = value;
            }
        }

        fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}
    }
}
