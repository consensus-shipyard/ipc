// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::num::NonZeroUsize;
use tracing::Level;
pub use tracing_appender::non_blocking;
pub use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{fmt, fmt::Subscriber, layer::SubscriberExt, Layer};

use crate::config::{level_to_filter, FileLayerSettings, LogLevel, TracingSettings};
use crate::tracing_layers::DomainEventFilterLayer;

pub const TRACING_TARGET: &str = "tracing_event";

// Creates a temporary subscriber that logs all traces to stderr. Useful when global tracing is not set yet.
pub fn create_temporary_subscriber() -> Subscriber {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .finish()
}

// Sets a global tracing subscriber with the given configuration. Returns a guard that can be used to drop the subscriber.
pub fn set_global_tracing_subscriber(
    config: &TracingSettings,
) -> (Option<WorkerGuard>, Option<WorkerGuard>) {
    let (logs_file_layer, traces_file_layer, logs_file_guard, traces_file_guard) = match &config
        .file
    {
        Some(file_settings) if file_settings.enabled => {
            // setup logs file layer first - logs are traces that does not have the target set to TRACING_TARGET
            let (logs_non_blocking, logs_file_guard) =
                non_blocking(create_file_appender(file_settings, "app.logs"));

            let log_filter = file_settings.level_to_filter().add_directive(
                format!("{TRACING_TARGET}=off")
                    .parse()
                    .expect("invalid logs level"),
            );

            let logs_file_layer = fmt::layer()
                .json()
                .with_writer(logs_non_blocking)
                .with_target(false)
                .with_file(true)
                .with_line_number(true)
                .with_filter(log_filter);

            let (traces_non_blocking, traces_file_guard) =
                non_blocking(create_file_appender(file_settings, "app.traces"));

            // setup traces file layer - traces are logs that have the target set to TRACING_TARGET
            let traces_file_layer = fmt::layer()
                .json()
                .with_writer(traces_non_blocking)
                .with_target(false)
                .with_file(false)
                .with_line_number(false);

            let domains = file_settings
                .domain_filter
                .as_ref()
                .map(|v| v.iter().map(|s| s.to_string()).collect());

            let events = file_settings
                .events_filter
                .as_ref()
                .map(|v| v.iter().map(|s| s.to_string()).collect());

            let traces_level = match &file_settings.level {
                Some(level) => level,
                None => &LogLevel::default(),
            };

            let traces_filter = EnvFilter::try_new(format!("{TRACING_TARGET}={}", traces_level))
                .expect("invalid traces level");

            let traces_file_layer = DomainEventFilterLayer::new(domains, events, traces_file_layer)
                .with_filter(traces_filter);

            (
                Some(logs_file_layer),
                Some(traces_file_layer),
                Some(logs_file_guard),
                Some(traces_file_guard),
            )
        }
        _ => (None, None, None, None),
    };

    let console_filter = match &config.console {
        Some(console_settings) => console_settings.level_to_filter(),
        None => level_to_filter(&None),
    };

    // log all traces to stderr (reserving stdout for any actual output such as from the CLI commands)
    let console_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_filter(console_filter);

    let global_registry = tracing_subscriber::registry()
        .with(logs_file_layer)
        .with(traces_file_layer)
        .with(console_layer);

    tracing::subscriber::set_global_default(global_registry)
        .expect("Unable to set a global tracing subscriber");

    (logs_file_guard, traces_file_guard)
}

fn create_file_appender(settings: &FileLayerSettings, suffix: &str) -> RollingFileAppender {
    let directory = settings
        .directory
        .as_deref()
        .expect("missing file log directory");
    let mut appender = RollingFileAppender::builder().filename_suffix(suffix);

    if let Some(max_log_files) = settings.max_log_files {
        appender = appender.max_log_files(
            NonZeroUsize::new(max_log_files)
                .expect("max_log_files must be greater than 0")
                .into(),
        );
    };

    if let Some(rotation_kind) = &settings.rotation {
        let rotation: tracing_appender::rolling::Rotation = rotation_kind.clone().into();
        appender = appender.rotation(rotation);
    } else {
        appender = appender.rotation(tracing_appender::rolling::Rotation::DAILY);
    }

    appender
        .build(directory)
        .expect("failed to create traces appender")
}
