// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let err = ipc_cli::cli().await;

    if let Err(err) = err {
        println!();
        println!("Error: {err}");
        println!();
        for cause in err.chain() {
            println!("\tCaused by: {cause}");
        }
        println!();
        std::process::exit(1);
    } else {
        Ok(())
    }
}
