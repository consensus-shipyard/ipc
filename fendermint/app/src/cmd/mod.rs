// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! CLI command implementations.

use crate::{
    options::{Commands, Options},
    settings::{expand_tilde, Settings},
};
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use base64::engine::GeneralPurpose;
use base64::engine::{DecodePaddingMode, GeneralPurposeConfig};
use base64::{alphabet, Engine};

pub mod eth;
pub mod genesis;
pub mod key;
pub mod rpc;
pub mod run;

/// A [`GeneralPurpose`] engine using the [`alphabet::STANDARD`] base64 alphabet
/// padding bytes when writing but requireing no padding when reading.
const B64_ENGINE: GeneralPurpose = GeneralPurpose::new(
    &alphabet::STANDARD,
    GeneralPurposeConfig::new()
        .with_encode_padding(true)
        .with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

/// Encode bytes in a format that the Genesis deserializer can handle.
pub fn to_b64(bz: &[u8]) -> String {
    B64_ENGINE.encode(bz)
}

pub fn from_b64(b64: &str) -> anyhow::Result<Vec<u8>> {
    Ok(B64_ENGINE.decode(b64)?)
}

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
    match &opts.command {
        Commands::Run(args) => args.exec(settings(opts)?).await,
        Commands::Key(args) => args.exec(()).await,
        Commands::Genesis(args) => args.exec(()).await,
        Commands::Rpc(args) => args.exec(()).await,
        Commands::Eth(args) => args.exec(settings(opts)?.eth).await,
    }
}

/// Try to parse the settings in the configuration directory.
fn settings(opts: &Options) -> anyhow::Result<Settings> {
    let config_dir = match expand_tilde(opts.config_dir()) {
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
