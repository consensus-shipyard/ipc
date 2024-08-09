// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use serde::Deserialize;
use serde_with::serde_as;
use std::path::PathBuf;
use strum;
use tracing_appender;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::filter::EnvFilter;

#[serde_as]
#[derive(Debug, Deserialize, Clone, Default, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn to_filter(&self) -> anyhow::Result<EnvFilter> {
        // At this point the filter should have been parsed before,
        // but if we created a log level directly, it can fail.
        // We fail if it doesn't parse because presumably we _want_ to see those things.
        Ok(EnvFilter::try_new(self.to_string())?)
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum RotationKind {
    Minutely,
    Hourly,
    Daily,
    Never,
}

impl From<RotationKind> for tracing_appender::rolling::Rotation {
    fn from(kind: RotationKind) -> tracing_appender::rolling::Rotation {
        match kind {
            RotationKind::Minutely => tracing_appender::rolling::Rotation::MINUTELY,
            RotationKind::Hourly => tracing_appender::rolling::Rotation::HOURLY,
            RotationKind::Daily => tracing_appender::rolling::Rotation::DAILY,
            RotationKind::Never => tracing_appender::rolling::Rotation::NEVER,
        }
    }
}

impl RotationKind {
    pub fn to_tracing_rotation(&self) -> Rotation {
        match self {
            RotationKind::Minutely => Rotation::DAILY,
            RotationKind::Hourly => Rotation::HOURLY,
            RotationKind::Daily => Rotation::DAILY,
            RotationKind::Never => Rotation::NEVER,
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Clone, Default)]
pub struct TracingSettings {
    pub console: Option<ConsoleLayerSettings>,
    pub file: Option<FileLayerSettings>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct ConsoleLayerSettings {
    pub level: Option<LogLevel>,
}

impl ConsoleLayerSettings {
    pub fn level_to_filter(&self) -> EnvFilter {
        level_to_filter(&self.level)
    }
}

impl Default for ConsoleLayerSettings {
    fn default() -> Self {
        ConsoleLayerSettings {
            level: Some(LogLevel::default()),
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Clone, Default)]
pub struct FileLayerSettings {
    pub enabled: bool,
    pub level: Option<LogLevel>,
    pub directory: Option<PathBuf>,
    pub max_log_files: Option<usize>,
    pub rotation: Option<RotationKind>,
    pub domain_filter: Option<Vec<String>>,
    pub events_filter: Option<Vec<String>>,
}

impl FileLayerSettings {
    pub fn level_to_filter(&self) -> EnvFilter {
        level_to_filter(&self.level)
    }
}

pub fn level_to_filter(level: &Option<LogLevel>) -> EnvFilter {
    match level {
        Some(level) => level.to_filter().unwrap_or_default(),
        None => LogLevel::Trace.to_filter().unwrap_or_default(),
    }
}
