// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use ipc_provider::set_fil_network_from_env;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    set_fil_network_from_env();

    if let Err(e) = ipc_cli::cli().await {
        log::error!("main process failed: {e:#}");
        std::process::exit(1);
    }
}
