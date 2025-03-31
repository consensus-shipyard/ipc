// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use serde::Deserialize;
use serde_with::serde_as;
use std::path::PathBuf;
use strum;
use tracing_appender;
use tracing_appender::rolling::Rotation;

#[serde_as]
#[derive(Debug, Deserialize, Clone, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "lowercase")]
pub enum RotationKind {
    Minutely,
    Hourly,
    Daily,
    Never,
}

impl From<&RotationKind> for tracing_appender::rolling::Rotation {
    fn from(kind: &RotationKind) -> tracing_appender::rolling::Rotation {
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
#[derive(Debug, Deserialize, Clone)]
pub struct TracingSettings {
    #[serde(default = "default_console_settings")]
    pub console: Option<ConsoleLayerSettings>,

    #[serde(default)] // still optional
    pub file: Option<FileLayerSettings>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct ConsoleLayerSettings {
    #[serde(default = "default_log_level")]
    pub level: Option<String>,
}

pub fn default_log_level() -> Option<String> {
    Some("info".to_string())
}

fn default_console_settings() -> Option<ConsoleLayerSettings> {
    Some(ConsoleLayerSettings {
        level: default_log_level(),
    })
}

// If you still want to `#[derive(Default)]`:
impl Default for ConsoleLayerSettings {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

impl Default for TracingSettings {
    fn default() -> Self {
        Self {
            console: default_console_settings(),
            file: None,
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Clone, Default)]
pub struct FileLayerSettings {
    pub enabled: bool,
    pub level: Option<String>,
    pub directory: Option<PathBuf>,
    pub max_log_files: Option<usize>,
    pub rotation: Option<RotationKind>,
    pub domain_filter: Option<Vec<String>>,
    pub events_filter: Option<Vec<String>>,
}
