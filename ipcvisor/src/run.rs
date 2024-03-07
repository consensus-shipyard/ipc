// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::thread;

use anyhow::anyhow;
use anyhow::Context;
use clap::Args;

use crate::config::CONFIG_NAME;
use crate::fendermint::Fendermint;
use crate::{config::Config, io::import_json};

#[derive(Args, Debug)]
pub struct RunArgs {
    // the root directory of the ipcvisor
    root_dir: PathBuf,
}

pub fn run(args: RunArgs) -> anyhow::Result<()> {
    // read the config from root_dir
    let config_path = args.root_dir.join(CONFIG_NAME);
    let config = import_json::<Config>(config_path)
        .context("failed to read {CONFIG_NAME}")?
        .ok_or_else(|| anyhow!("missing {CONFIG_NAME}"))?;
    println!("config: {:#?}", config);

    let fendermint = Fendermint::new(&config);

    let mut should_restart = fendermint.run()?;
    if config.restart_after_upgrade && should_restart {
        while should_restart {
            println!("restarting fendermint");
            should_restart = fendermint.run()?;
        }
    }

    Ok(())
}
