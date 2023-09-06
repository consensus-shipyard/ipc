// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
#![feature(try_blocks)]
use fvm_shared::address::{set_current_network, Network};
use num_traits::FromPrimitive;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    if let Err(e) = ipc_cli::cli().await {
        log::error!("main process failed: {e:#}");
        std::process::exit(1);
    }
}
