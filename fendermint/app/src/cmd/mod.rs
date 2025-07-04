// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! CLI command implementations.

use std::sync::Arc;

use crate::{
    options::{Commands, Options},
    settings::{utils::expand_tilde, Settings},
};
use anyhow::{anyhow, Context};
use async_trait::async_trait;

use ipc_observability::config::TracingSettings;
use ipc_observability::traces::create_temporary_subscriber;
use ipc_observability::traces::set_global_tracing_subscriber;
use tracing::subscriber;

pub mod config;
pub mod debug;
pub mod eth;
pub mod genesis;
pub mod key;
pub mod materializer;
pub mod rpc;
pub mod run;

#[async_trait]
pub trait Cmd {
    type Settings;
    async fn exec(&self, settings: Self::Settings) -> anyhow::Result<()>;
}

/// Convenience macro to simplify declaring commands that either need or don't need settings.
///
/// ```text
/// cmd! {
///   <arg-type>(self, settings: <settings-type>) {
///     <exec-body>
///   }
/// }
/// ```
#[macro_export]
macro_rules! cmd {
    // A command which needs access to some settings.
    ($name:ident($self:ident, $settings_name:ident : $settings_type:ty) $exec:expr) => {
        #[async_trait::async_trait]
        impl $crate::cmd::Cmd for $name {
            type Settings = $settings_type;

            async fn exec(&$self, $settings_name: Self::Settings) -> anyhow::Result<()> {
                $exec
            }
        }
    };

    // A command which works on the full `Settings`.
    ($name:ident($self:ident, $settings:ident) $exec:expr) => {
        cmd!($name($self, $settings: $crate::settings::Settings) $exec);
    };

    // A command which is self-contained and doesn't need any settings.
    ($name:ident($self:ident) $exec:expr) => {
        cmd!($name($self, _settings: ()) $exec);
    };
}

/// Execute the command specified in the options.
pub async fn exec(opts: Arc<Options>) -> anyhow::Result<()> {
    match &opts.command {
        Commands::Config(args) => args.exec(opts.clone()).await,
        Commands::Debug(args) => {
            let _trace_file_guard = set_global_tracing_subscriber(&TracingSettings::default());
            args.exec(()).await
        }
        Commands::Run(args) => {
            let settings = load_settings(opts.clone())?;
            let _trace_file_guard = set_global_tracing_subscriber(&settings.tracing);
            args.exec(settings).await
        }
        Commands::Key(args) => {
            let _trace_file_guard = set_global_tracing_subscriber(&TracingSettings::default());
            args.exec(()).await
        }
        Commands::Genesis(args) => {
            let _trace_file_guard = set_global_tracing_subscriber(&TracingSettings::default());
            args.exec(()).await
        }
        Commands::Rpc(args) => {
            let _trace_file_guard = set_global_tracing_subscriber(&TracingSettings::default());
            args.exec(()).await
        }
        Commands::Eth(args) => {
            let settings = load_settings(opts.clone())?.eth;
            let _trace_file_guard = set_global_tracing_subscriber(&settings.tracing);
            args.exec(settings).await
        }
        Commands::Materializer(args) => {
            let _trace_file_guard = set_global_tracing_subscriber(&TracingSettings::default());
            args.exec(()).await
        }
    }
}

/// Try to parse the settings in the configuration directory.
pub fn load_settings(opts: Arc<Options>) -> anyhow::Result<Settings> {
    let config_dir = match expand_tilde(opts.config_dir()) {
        d if !d.exists() => return Err(anyhow!("'{d:?}' does not exist")),
        d if !d.is_dir() => return Err(anyhow!("'{d:?}' is a not a directory")),
        d => d,
    };

    subscriber::with_default(create_temporary_subscriber(), || {
        tracing::info!(
            path = config_dir.to_string_lossy().into_owned(),
            "reading configuration"
        )
    });

    let settings =
        Settings::new(&config_dir, &opts.home_dir, &opts.mode).context("error parsing settings")?;

    Ok(settings)
}
