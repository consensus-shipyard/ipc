// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use clap::{builder::PossibleValue, ValueEnum};

use lazy_static::lazy_static;
use tracing_subscriber::EnvFilter;

/// Standard log levels, or something we can pass to <https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html>
///
/// To be fair we could pass of everything except "off" as a filter.
#[derive(Debug, Clone)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Filter(String),
}

impl LogLevel {
    pub fn to_filter(&self) -> anyhow::Result<Option<EnvFilter>> {
        let filter = match self {
            LogLevel::Off => return Ok(None),
            LogLevel::Filter(s) => s,
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        };

        // At this point the filter should have been parsed before,
        // but if we created a log level directly, it can fail.
        // We fail if it doesn't parse because presumably we _want_ to see those things.
        let filter = EnvFilter::try_new(filter)?;

        Ok(Some(filter))
    }
}

impl ValueEnum for LogLevel {
    fn value_variants<'a>() -> &'a [Self] {
        lazy_static! {
            static ref VARIANTS: Vec<LogLevel> = vec![
                LogLevel::Off,
                LogLevel::Error,
                LogLevel::Warn,
                LogLevel::Info,
                LogLevel::Debug,
                LogLevel::Trace,
            ];
        }

        &VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        let pv = |name: &str| Some(PossibleValue::new(name.to_lowercase()));
        match self {
            LogLevel::Off => pv("Off"),
            LogLevel::Error => pv("Error"),
            LogLevel::Warn => pv("Warn"),
            LogLevel::Info => pv("Info"),
            LogLevel::Debug => pv("Debug"),
            LogLevel::Trace => pv("Trace"),
            LogLevel::Filter(_) => None,
        }
    }
}

pub fn parse_log_level(s: &str) -> Result<LogLevel, String> {
    if let Ok(lvl) = ValueEnum::from_str(s, true) {
        return Ok(lvl);
    }
    // `EnvFilter` is not `Clone`, so we can't store it, but we can use it to validate early.
    if let Err(e) = EnvFilter::try_new(s) {
        Err(e.to_string())
    } else {
        Ok(LogLevel::Filter(s.to_string()))
    }
}
