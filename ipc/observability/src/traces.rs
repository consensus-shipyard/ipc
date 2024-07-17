// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::num::NonZeroUsize;
use tracing::Level;
pub use tracing_appender::non_blocking;
pub use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::{fmt, fmt::Subscriber, layer::SubscriberExt, Layer};

use crate::traces_settings::{FileLayerSettings, TracesSettings};
use crate::tracing_layers::DomainEventFilterLayer;
use anyhow::Result;

pub fn create_subscriber() -> Subscriber {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        // builds the subscriber.
        .finish()
}

pub fn set_global_tracing_subscriber(config: &TracesSettings) -> Result<WorkerGuard> {
    // log all traces to stderr (reserving stdout for any actual output such as from the CLI commands)
    let console_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_filter(config.console.level.to_filter()?);

    let (file_layer, file_guard) = if config.file.enabled {
        let (non_blocking, file_guard) = non_blocking(create_file_appender(&config.file));

        let file_layer = fmt::layer()
            .json()
            .with_writer(non_blocking)
            .with_span_events(fmt::format::FmtSpan::CLOSE)
            .with_target(false)
            .with_file(true)
            .with_line_number(true)
            .with_filter(config.file.level.to_filter()?);

        let domains = config
            .file
            .domain_filter
            .as_ref()
            .map(|v| v.iter().map(|s| s.to_string()).collect());

        let events = config
            .file
            .events_filter
            .as_ref()
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

    file_guard.ok_or_else(|| anyhow::anyhow!("file guard not created"))
}

fn create_file_appender(settings: &FileLayerSettings) -> RollingFileAppender {
    let directory = settings
        .directory
        .as_deref()
        .expect("missing file log directory");
    let mut appender = RollingFileAppender::builder().filename_suffix("traces");

    if let Some(max_log_files) = settings.max_log_files {
        println!("max log files: {}", max_log_files);

        appender = appender.max_log_files(
            NonZeroUsize::new(max_log_files)
                .expect("max_log_files must be greater than 0")
                .into(),
        );
    };

    if let Some(rotation_kind) = &settings.rotation {
        println!("rotation kind: {:?}", rotation_kind);
        appender = appender.rotation(rotation_kind.to_tracing_rotation());
    };

    appender
        .build(directory)
        .expect("failed to create traces appender")
}
