// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::num::NonZeroUsize;
pub use tracing_appender::non_blocking;
pub use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, Layer};

use crate::tracing_layers::DomainEventFilterLayer;

#[derive(Debug)]
pub enum RotationKind {
    Minutely,
    Hourly,
    Daily,
    Never,
}

impl RotationKind {
    fn to_tracing_rotation(&self) -> Rotation {
        match self {
            RotationKind::Minutely => Rotation::DAILY,
            RotationKind::Hourly => Rotation::HOURLY,
            RotationKind::Daily => Rotation::DAILY,
            RotationKind::Never => Rotation::NEVER,
        }
    }
}

impl From<&str> for RotationKind {
    fn from(s: &str) -> Self {
        match s {
            "minutely" => RotationKind::Minutely,
            "hourly" => RotationKind::Hourly,
            "daily" => RotationKind::Daily,
            "never" => RotationKind::Never,
            _ => panic!("invalid rotation kind"),
        }
    }
}

#[derive(Default)]
pub struct FileLayerConfig<'a> {
    pub enabled: bool,
    pub directory: Option<&'a str>,
    pub max_log_files: Option<usize>,
    pub rotation: Option<RotationKind>,
    pub domain_filter: Option<Vec<&'a str>>,
    pub events_filter: Option<Vec<&'a str>>,
}

// Register a tracing subscriber with the given options
// Returns a guard that must be kept alive for the duration of the program (because it's non-blocking and needs to flush)
pub fn register_tracing_subscriber(
    console_level_filter: EnvFilter,
    file_level_filter: EnvFilter,
    file_opts: FileLayerConfig<'_>,
) -> Option<WorkerGuard> {
    // log all traces to stderr (reserving stdout for any actual output such as from the CLI commands)
    let console_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_filter(console_level_filter);

    let (file_layer, file_guard) = if file_opts.enabled {
        let (non_blocking, file_guard) = non_blocking(file_appender_from_opts(&file_opts));

        let file_layer = fmt::layer()
            .json()
            .with_writer(non_blocking)
            .with_span_events(fmt::format::FmtSpan::CLOSE)
            .with_target(false)
            .with_file(true)
            .with_line_number(true)
            .with_filter(file_level_filter);

        let domains = file_opts
            .domain_filter
            .map(|v| v.iter().map(|s| s.to_string()).collect());
        let events = file_opts
            .events_filter
            .map(|v| v.iter().map(|s| s.to_string()).collect());

        let file_layer = DomainEventFilterLayer::new(domains, events, file_layer);

        (Some(file_layer), Some(file_guard))
    } else {
        (None, None)
    };

    let registry = tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer);

    tracing::subscriber::set_global_default(registry)
        .expect("Unable to set a global tracing subscriber");

    file_guard
}

fn file_appender_from_opts(opts: &FileLayerConfig<'_>) -> RollingFileAppender {
    let directory = opts.directory.expect("traces directory must be set");
    let mut appender = RollingFileAppender::builder().filename_suffix("traces");

    if let Some(max_log_files) = opts.max_log_files {
        println!("max log files: {}", max_log_files);

        appender = appender.max_log_files(
            NonZeroUsize::new(max_log_files)
                .expect("max_log_files must be greater than 0")
                .into(),
        );
    };

    if let Some(rotation_kind) = &opts.rotation {
        println!("rotation kind: {:?}", rotation_kind);
        appender = appender.rotation(rotation_kind.to_tracing_rotation());
    };

    appender
        .build(directory)
        .expect("failed to create traces appender")
}
