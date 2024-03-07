// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};

pub const CONFIG_NAME: &str = "config.json";

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // the root directory of the ipcvisor where we store all configs, binaries, and data
    pub root_dir: PathBuf,
    // the parameters we pass to fendermint when running ipcvisor
    pub fendermint_params: String,
    // the parameters we pass to cometbft when running ipcvisor
    pub cometbft_params: String,

    pub restart_after_upgrade: bool,
    #[serde_as(as = "DurationSeconds<u64>")]
    pub shutdown_grace: Duration,
}
