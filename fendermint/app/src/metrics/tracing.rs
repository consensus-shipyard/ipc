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
                $field: 0,
                ..Default::default()
            };
        }
    }};
}

macro_rules! set_gauge {
    ($event:expr, $event_ty:ident :: $field:ident, $gauge:expr) => {
        check_field!($event_ty::$field);
        let mut fld = visitors::FindU64::new(stringify!($field));
        $event.record(&mut fld);
        $gauge.set(fld.value as i64);
    };
}

macro_rules! inc_counter {
    ($event:expr, $event_ty:ident :: $field:ident, $counter:expr) => {
        check_field!($event_ty::$field);
        let mut fld = visitors::FindU64::new(stringify!($field));
        $event.record(&mut fld);
        $counter.inc_by(fld.value);
    };
}

impl<S: Subscriber> Layer<S> for MetricsLayer<S> {
    fn on_event(&self, event: &Event<'_>, _ctx: layer::Context<'_, S>) {
        match event.metadata().name() {
            "event::NewParentView" => {
                set_gauge!(
                    event,
                    NewParentView::block_height,
                    &am::TOPDOWN_VIEW_BLOCK_HEIGHT
                );
                inc_counter!(event, NewParentView::num_msgs, &am::TOPDOWN_VIEW_NUM_MSGS);
                inc_counter!(
                    event,
                    NewParentView::num_validator_changes,
                    am::TOPDOWN_VIEW_NUM_VAL_CHNGS
                );
            }
            "event::ParentFinalityCommitted" => {
                set_gauge!(
                    event,
                    ParentFinalityCommitted::block_height,
                    &am::TOPDOWN_FINALIZED_BLOCK_HEIGHT
                );
            }
            "event::NewBottomUpCheckpoint" => {
                set_gauge!(
                    event,
                    NewBottomUpCheckpoint::block_height,
                    &am::BOTTOMUP_CKPT_BLOCK_HEIGHT
                );
                set_gauge!(
                    event,
                    NewBottomUpCheckpoint::next_configuration_number,
                    &am::BOTTOMUP_CKPT_CONFIG_NUM
                );
                inc_counter!(
                    event,
                    NewBottomUpCheckpoint::num_msgs,
                    &am::BOTTOMUP_CKPT_NUM_MSGS
                );
            }
            _ => {}
        }
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
