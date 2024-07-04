// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod register;

use std::fmt::Display;
use tracing::{debug, error, info, trace, warn};

pub trait Recordable {
    fn record_metrics(&self);
}

pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

pub trait Traceable {
    fn level(&self) -> Level;
}

pub fn emit<T>(trace: T)
where
    T: Recordable + Traceable + Display,
{
    match trace.level() {
        Level::Trace => trace!(event = %trace),
        Level::Debug => debug!(event = %trace),
        Level::Info => info!(event = %trace),
        Level::Warn => warn!(event = %trace),
        Level::Error => error!(event = %trace),
    }

    trace.record_metrics();
}
