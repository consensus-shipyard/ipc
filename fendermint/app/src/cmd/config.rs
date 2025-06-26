// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_app_options::config::{
    ConfigArgs, ConfigCommands, DisplayConfigArgs, InitConfigArgs,
};

use crate::{
    cmd, cmd::load_settings, options::Options, settings::utils::expand_tilde, settings::Settings,
};
use anyhow::Context;
use std::{fs, path::Path, sync::Arc};
use toml;

cmd! {
  ConfigArgs(self, options: Arc<Options>) {
    match &self.command {
        ConfigCommands::Init(args) => args.exec(options).await,
        ConfigCommands::Display(args) => args.exec(load_settings(options)?).await
    }
  }
}

cmd! {
  DisplayConfigArgs(self, settings) {
    print_settings(settings)
  }
}

cmd! {
  InitConfigArgs(self, options: Arc<Options>) {
    write_default_settings(options.config_dir())
  }
}

fn print_settings(settings: Settings) -> anyhow::Result<()> {
    let settings_str = toml::to_string_pretty(&settings)?;
    println!("{}", settings_str);
    Ok(())
}

pub fn write_default_settings(config_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    // make sure the folder exists
    let dir = expand_tilde(config_dir);
    fs::create_dir_all(&dir).with_context(|| {
        format!(
            "failed to create configuration directory `{}`",
            dir.display()
        )
    })?;

    // serialize the defaults
    let settings = Settings::default();
    let toml_str = toml::to_string_pretty(&settings)
        .context("failed to serialize default Settings to TOML")?;

    // write the file
    let file = dir.join("default.toml");
    println!("Writing default config to `{}`", file.display());

    fs::write(&file, toml_str)
        .with_context(|| format!("failed to write default config to `{}`", file.display()))?;

    Ok(())
}
