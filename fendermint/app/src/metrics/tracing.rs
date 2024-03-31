// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Subscribing to tracing events and turning them into metrics.

use std::marker::PhantomData;

use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{filter, layer, registry::LookupSpan, Layer};

use super::prometheus::app as am;
use crate::events::*;
use metrics_utils::{event_match, event_name, event_mapping, set_gauge, inc_counter, check_field};
use metrics_utils::tracing::visitors;

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
            NewParentView {
                block_height              => set_gauge   ! &am::TOPDOWN_VIEW_BLOCK_HEIGHT,
                num_msgs                  => inc_counter ! &am::TOPDOWN_VIEW_NUM_MSGS,
                num_validator_changes     => inc_counter ! &am::TOPDOWN_VIEW_NUM_VAL_CHNGS,
            },
            ParentFinalityCommitted {
                block_height              => set_gauge   ! &am::TOPDOWN_FINALIZED_BLOCK_HEIGHT,
            },
            NewBottomUpCheckpoint {
                block_height              => set_gauge   ! &am::BOTTOMUP_CKPT_BLOCK_HEIGHT,
                next_configuration_number => set_gauge   ! &am::BOTTOMUP_CKPT_CONFIG_NUM,
                num_msgs                  => inc_counter ! &am::BOTTOMUP_CKPT_NUM_MSGS,
            },
            NewBlock {
                block_height              => set_gauge   ! &am::ABCI_COMMITTED_BLOCK_HEIGHT
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use tracing_utils::emit;
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
