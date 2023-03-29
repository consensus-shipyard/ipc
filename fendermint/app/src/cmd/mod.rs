// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! CLI command implementations.

use crate::{
    options::{Commands, GenesisCommands, Options},
    settings::Settings,
};
use anyhow::{anyhow, Context};
use async_trait::async_trait;

pub mod genesis;
pub mod keygen;
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
pub async fn exec(opts: &Options) -> anyhow::Result<()> {
    let fut = match &opts.command {
        Commands::Run(args) => args.exec(settings(opts)?),
        Commands::Keygen(args) => args.exec(()),
        Commands::Genesis(gargs) => {
            let genesis_file = gargs.genesis_file.clone();
            match &gargs.command {
                GenesisCommands::New(args) => args.exec(genesis_file),
                GenesisCommands::AddAccount(args) => args.exec(genesis_file),
                GenesisCommands::AddMultisig(args) => args.exec(genesis_file),
                GenesisCommands::AddValidator(args) => args.exec(genesis_file),
                GenesisCommands::IntoTendermint(args) => args.exec(genesis_file),
            }
        }
    };
    fut.await
}

/// Try to parse the settings in the configuration directory.
fn settings(opts: &Options) -> anyhow::Result<Settings> {
    let config_dir = match opts.config_dir() {
        d if !d.exists() => return Err(anyhow!("'{d:?}' does not exist")),
        d if !d.is_dir() => return Err(anyhow!("'{d:?}' is a not a directory")),
        d => d,
    };

    tracing::info!(
        path = config_dir.to_string_lossy().into_owned(),
        "reading configuration"
    );
    let settings =
        Settings::new(&config_dir, &opts.home_dir, &opts.mode).context("error parsing settings")?;

    Ok(settings)
}
