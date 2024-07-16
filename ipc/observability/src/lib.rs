// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod macros;
pub mod traces;
mod tracing_layers;
pub use lazy_static::lazy_static;

use std::fmt::Debug;
use tracing::{debug, error, info, trace, warn};

pub trait Recordable {
    fn record_metrics(&self);
}

pub trait Traceable {
    fn trace_level(&self) -> TraceLevel;
    fn domain(&self) -> &'static str;
}

pub enum TraceLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

pub fn emit<T>(trace: T)
where
    T: Recordable + Traceable + Debug,
{
    match trace.trace_level() {
        TraceLevel::Trace => trace!(domain=trace.domain(), event = ?trace),
        TraceLevel::Debug => debug!(domain=trace.domain(), event = ?trace),
        TraceLevel::Info => info!(domain=trace.domain(), event = ?trace),
        TraceLevel::Warn => warn!(domain=trace.domain(), event = ?trace),
        TraceLevel::Error => error!(domain=trace.domain(), event = ?trace),
    }

    trace.record_metrics();
}