// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Subscribing to tracing events and turning them into metrics.

pub mod visitors {
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

    // Looking for multiple values because the callsite might be passed as a literal which turns into an i64 for example.
    impl<'a> Visit for FindU64<'a> {
        fn record_u64(&mut self, field: &Field, value: u64) {
            if field.name() == self.name {
                self.value = value;
            }
        }

        fn record_i64(&mut self, field: &Field, value: i64) {
            if field.name() == self.name {
                self.value = value as u64;
            }
        }

        fn record_i128(&mut self, field: &Field, value: i128) {
            if field.name() == self.name {
                self.value = value as u64;
            }
        }

        fn record_u128(&mut self, field: &Field, value: u128) {
            if field.name() == self.name {
                self.value = value as u64;
            }
        }

        fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}
    }
}

/// Check that the field exist on a type; if it doesn't this won't compile.
/// This ensures that we're mapping fields with the correct name.
#[macro_export]
macro_rules! check_field {
    ($event_ty:ident :: $field:ident) => {{
        if false {
            #[allow(clippy::needless_update)]
            let _event = $event_ty {
                $field: Default::default(),
                ..Default::default()
            };
        }
    }};
}

/// Set a gague to an absolute value based on a field in an event.
#[macro_export]
macro_rules! set_gauge {
    ($event:ident, $event_ty:ident :: $field:ident, $gauge:expr) => {
        check_field!($event_ty::$field);
        let mut fld = visitors::FindU64::new(stringify!($field));
        $event.record(&mut fld);
        $gauge.set(fld.value as i64);
    };
}

/// Increment a counter by the value of a field in the event.
#[macro_export]
macro_rules! inc_counter {
    ($event:ident, $event_ty:ident :: $field:ident, $counter:expr) => {
        check_field!($event_ty::$field);
        let mut fld = visitors::FindU64::new(stringify!($field));
        $event.record(&mut fld);
        $counter.inc_by(fld.value);
    };
}

/// Produce the prefixed event name from the type name.
#[macro_export]
macro_rules! event_name {
    ($event_ty:ident) => {
        concat!("event::", stringify!($event_ty))
    };
}

/// Call one of the macros that set values on a metric.
#[macro_export]
macro_rules! event_mapping {
    ($op:ident, $event:ident, $event_ty:ident :: $field:ident, $metric:expr) => {
        $op!($event, $event_ty::$field, $metric);
    };
}

/// Match the event name to event DTO types and within the map fields to metrics.
#[macro_export]
macro_rules! event_match {
    ($event:ident { $( $event_ty:ident { $( $field:ident => $op:ident ! $metric:expr  ),* $(,)? } ),* } ) => {
        println!("jiejie: Got a event with name {}", $event.metadata().name());
        match $event.metadata().name() {
            $(
                event_name!($event_ty) => {
                    println!("jiejie: Matched event!");
                    $(
                        event_mapping!($op, $event, $event_ty :: $field, $metric);
                    )*
                }
            )*
            _ => {}
        }
    };
}

/// This test shows how the user should use this metrics library in their
/// own applications.
#[cfg(test)]
mod tests {
    use crate::metrics;
    use crate::tracing::visitors;
    use lazy_static::lazy_static;
    use paste::paste;
    use prometheus::IntGauge;
    use prometheus::Registry;
    use std::marker::PhantomData;
    use tracing::{Event, Level, Subscriber};
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::registry::LookupSpan;
    use tracing_subscriber::{filter, layer, Layer};
    use tracing_utils::emit;

    struct MetricsLayer<S> {
        _subscriber: PhantomData<S>,
    }

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

    impl<S> MetricsLayer<S> {
        pub fn new() -> Self {
            Self {
                _subscriber: PhantomData,
            }
        }
    }

    #[allow(dead_code)]
    #[derive(Default)]
    struct MyBlockEvent {
        pub block_height: u64,
    }

    metrics! {
        TEST_GUAGE: IntGauge = "A test gauge";
    }

    impl<S: Subscriber> Layer<S> for MetricsLayer<S> {
        fn on_event(&self, event: &Event<'_>, _ctx: layer::Context<'_, S>) {
            event_match!(event {
                MyBlockEvent {
                    block_height => set_gauge ! &TEST_GUAGE
                }
            });
        }
    }

    #[test]
    fn test_metrics_layer() {
        let gauge: &IntGauge = &TEST_GUAGE;

        let v0 = gauge.get();
        gauge.inc();
        let v1 = gauge.get();
        assert!(v1 > v0, "gauge should change without being registered");

        let block_height = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let subscriber = tracing_subscriber::registry().with(layer());

        tracing::subscriber::with_default(subscriber, || {
            emit! {
                MyBlockEvent { block_height: block_height }
            }
        });

        assert_eq!(
            gauge.get() as u64,
            block_height,
            "metrics should be captured"
        );
    }

    #[test]
    fn can_register_metrics() {
        let r = prometheus::Registry::new();
        register_metrics(&r).unwrap();
    }
}
