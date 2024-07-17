// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use serde::Deserialize;
use serde_with::serde_as;
use std::path::PathBuf;
use std::str::FromStr;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::filter::EnvFilter;

#[serde_as]
#[derive(Debug, Deserialize, Clone, Default)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    #[default]
    Trace,
}

impl LogLevel {
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Off => "off",
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }

    pub fn to_filter(&self) -> anyhow::Result<EnvFilter> {
        // At this point the filter should have been parsed before,
        // but if we created a log level directly, it can fail.
        // We fail if it doesn't parse because presumably we _want_ to see those things.
        Ok(EnvFilter::try_new(self.as_str())?)
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub enum RotationKind {
    Minutely,
    Hourly,
    Daily,
    Never,
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

impl FromStr for RotationKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "minutely" => Ok(RotationKind::Minutely),
            "hourly" => Ok(RotationKind::Hourly),
            "daily" => Ok(RotationKind::Daily),
            "never" => Ok(RotationKind::Never),
            _ => Err(format!("invalid rotation kind: {}", s)),
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct TracesSettings {
    pub console: ConsoleLayerSettings,
    pub file: FileLayerSettings,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct ConsoleLayerSettings {
    pub enabled: bool,
    pub level: LogLevel,
}

impl Default for ConsoleLayerSettings {
    fn default() -> Self {
        ConsoleLayerSettings {
            enabled: true,
            level: LogLevel::default(),
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Clone, Default)]

pub struct FileLayerSettings {
    pub enabled: bool,
    pub level: LogLevel,
    pub directory: Option<PathBuf>,
    pub max_log_files: Option<usize>,
    pub rotation: Option<RotationKind>,
    pub domain_filter: Option<Vec<String>>,
    pub events_filter: Option<Vec<String>>,
}
