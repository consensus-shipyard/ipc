// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

pub mod comet_runner;

#[tokio::main]
async fn main() {
    if let Err(e) = ipc_cli::cli().await {
        log::error!("main process failed: {e:#}");
        std::process::exit(1);
    }
}
