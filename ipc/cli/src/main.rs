// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {

        tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    if let Err(e) = ipc_cli::cli().await {
        log::error!("main process failed: {e:#}");
        std::process::exit(1);
    }
}
