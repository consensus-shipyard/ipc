// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::num::NonZeroUsize;
pub use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer};

#[derive(Default)]
pub struct JournalOpts<'a> {
    pub enabled: bool,
    pub directory: Option<&'a str>,
    pub max_log_files: Option<usize>,
    pub rotation: Option<&'a str>,
    pub filters: Option<Vec<&'a str>>,
}

fn appender_from_opts(opts: &JournalOpts<'_>) -> RollingFileAppender {
    let directory = opts.directory.expect("journal directory must be set");
    let mut appender = RollingFileAppender::builder().filename_suffix("journal");

    if let Some(max_log_files) = opts.max_log_files {
        appender = appender.max_log_files(
            NonZeroUsize::new(max_log_files)
                .expect("max_log_files must be greater than 0")
                .into(),
        );
    };

    if let Some(rotation_str) = opts.rotation {
        let rotation = match rotation_str {
            "minutely" => Rotation::DAILY,
            "hourly" => Rotation::HOURLY,
            "daily" => Rotation::DAILY,
            "never" => Rotation::NEVER,
            _ => panic!("invalid rotation: {}", rotation_str),
        };

        appender = appender.rotation(rotation);
    };

    appender
        .build(directory)
        .expect("failed to create journal appender")
}

// register a tracing subscriber with the given options
// returns a guard that must be kept alive for the duration of the program (because it's non-blocking and needs to flush)
pub fn register_tracing_subscriber(
    console_filter: EnvFilter,
    opts: &JournalOpts<'_>,
) -> Option<WorkerGuard> {
    // log all traces to stderr (reserving stdout for any actual output such as from the CLI commands)
    // TODO Karel - do we want to always allow it or should we make it configurable?
    let console_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_filter(console_filter);

    // add a file layer if log_dir is set
    let (file_layer, file_guard) = if opts.enabled {
        let (non_blocking, file_guard) = tracing_appender::non_blocking(appender_from_opts(opts));

        let mut file_filter = EnvFilter::from_default_env();

        if let Some(domain_filter) = &opts.filters {
            for domain in domain_filter {
                file_filter = file_filter.add_directive(domain.parse().unwrap());
            }
        }

        let file_layer = fmt::layer()
            .json()
            .with_writer(non_blocking)
            .with_span_events(fmt::format::FmtSpan::CLOSE)
            .with_target(false)
            .with_file(true)
            .with_line_number(true)
            .with_filter(file_filter);

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
