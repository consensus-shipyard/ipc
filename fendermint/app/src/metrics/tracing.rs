// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Subscribing to tracing events and turning them into metrics.

use std::marker::PhantomData;

use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{filter, layer, registry::LookupSpan, Layer};

use super::prometheus::app as am;
use crate::events::*;

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

macro_rules! check_field {
    ($event_ty:ident :: $field:ident) => {{
        if false {
            // Check that the field exist; if it doesn't this won't compile.
            let _event = $event_ty {
                $field: Default::default(),
                ..Default::default()
            };
        }
    }};
}

macro_rules! set_gauge {
    ($event:ident, $event_ty:ident :: $field:ident, $gauge:expr) => {
        check_field!($event_ty::$field);
        let mut fld = visitors::FindU64::new(stringify!($field));
        $event.record(&mut fld);
        $gauge.set(fld.value as i64);
    };
}

macro_rules! inc_counter {
    ($event:ident, $event_ty:ident :: $field:ident, $counter:expr) => {
        check_field!($event_ty::$field);
        let mut fld = visitors::FindU64::new(stringify!($field));
        $event.record(&mut fld);
        $counter.inc_by(fld.value);
    };
}

macro_rules! event_name {
    ($event_ty:ident) => {
        concat!("event::", stringify!($event_ty))
    };
}

macro_rules! event_mapping {
    (gauges, $event:ident, $event_ty:ident :: $field:ident, $metric:expr) => {
        set_gauge!($event, $event_ty::$field, $metric);
    };
    (counters, $event:ident, $event_ty:ident :: $field:ident, $metric:expr) => {
        inc_counter!($event, $event_ty::$field, $metric);
    };
}

macro_rules! event_match {
    ($event:ident { $( $event_ty:ident { $( $field:ident => $kind:ident / $metric:expr  ),* $(,)? } ),* } ) => {
        match $event.metadata().name() {
            $(
                event_name!($event_ty) => {
                    $(
                        event_mapping!($kind, $event, $event_ty :: $field, $metric);
                    )*
                }
            )*
            _ => {}
        }
    };
}

impl<S: Subscriber> Layer<S> for MetricsLayer<S> {
    fn on_event(&self, event: &Event<'_>, _ctx: layer::Context<'_, S>) {
        event_match!(event {
            NewParentView {
                block_height              => gauges   / &am::TOPDOWN_VIEW_BLOCK_HEIGHT,
                num_msgs                  => counters / &am::TOPDOWN_VIEW_NUM_MSGS,
                num_validator_changes     => counters / &am::TOPDOWN_VIEW_NUM_VAL_CHNGS,
            },
            ParentFinalityCommitted {
                block_height              => gauges   / &am::TOPDOWN_FINALIZED_BLOCK_HEIGHT,
            },
            NewBottomUpCheckpoint {
                block_height              => gauges   / &am::BOTTOMUP_CKPT_BLOCK_HEIGHT,
                next_configuration_number => gauges   / &am::BOTTOMUP_CKPT_CONFIG_NUM,
                num_msgs                  => counters / &am::BOTTOMUP_CKPT_NUM_MSGS,
            }
        });
    }
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

#[cfg(test)]
mod tests {
    use fendermint_tracing::emit;
    use fendermint_vm_event::ParentFinalityCommitted;
    use prometheus::IntGauge;
    use tracing_subscriber::layer::SubscriberExt;

    #[test]
    fn test_metrics_layer() {
        let gauge: &IntGauge = &super::super::prometheus::app::TOPDOWN_FINALIZED_BLOCK_HEIGHT;

        let v0 = gauge.get();
        gauge.inc();
        let v1 = gauge.get();
        assert!(v1 > v0, "gague should change without being registered");

        let block_height = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let subscriber = tracing_subscriber::registry().with(super::layer());

        tracing::subscriber::with_default(subscriber, || {
            emit! {
                ParentFinalityCommitted { block_height, block_hash: "metrics-test-block" }
            }
        });

        assert_eq!(
            gauge.get() as u64,
            block_height,
            "metrics should be captured"
        );
    }
}
