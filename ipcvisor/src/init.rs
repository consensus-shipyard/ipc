// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use std::{path::PathBuf, time::Duration};

use anyhow::anyhow;
use clap::Args;

use crate::{
    config::{Config, CONFIG_NAME},
    io::export_json,
};

#[derive(Args, Debug)]
pub struct InitArgs {
    // the root directory of the ipcvisor
    root_dir: PathBuf,
    // the path to the fendermint binary
    fendermint_bin: PathBuf,
    // the parameters we pass to fendermint when running ipcvisor
    fendermint_params: String,
    // the path to the cometbft binary
    cometbft_bin: PathBuf,
    // the parameters we pass to cometbft when running ipcvisor
    cometbft_params: String,
}

pub fn init(args: InitArgs) -> anyhow::Result<()> {
    let config = Config {
        root_dir: args.root_dir,
        fendermint_params: args.fendermint_params,
        cometbft_params: args.cometbft_params,
        restart_after_upgrade: true,
        shutdown_grace: Duration::from_secs(0),
    };

    // validate that the binaries are accessible and executable
    if !args.fendermint_bin.is_file() {
        return Err(anyhow!("fendermint binary is not a file"));
    }
    if !args.cometbft_bin.is_file() {
        return Err(anyhow!("cometbft_bin binary is not a file"));
    }

    // initialize the directory structure and copy the binaries to the genesis directory
    std::fs::create_dir_all(config.root_dir.join("genesis/bin"))?;
    std::fs::copy(
        &args.fendermint_bin,
        config.root_dir.join("genesis/bin/fendermint"),
    )?;
    std::fs::copy(
        &args.cometbft_bin,
        config.root_dir.join("genesis/bin/cometbft"),
    )?;

    // create a current symlink to genesis if it doesn't exist
    if !config.root_dir.join("current").exists() {
        std::os::unix::fs::symlink(
            config.root_dir.join("genesis"),
            config.root_dir.join("current"),
        )?;
    }

    // save the config
    export_json(config.root_dir.join(CONFIG_NAME), &config)?;

    println!("initialized ipcvisor at {:#?}", config.root_dir);

    Ok(())
}
