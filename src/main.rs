// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use ipc_agent::cli;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    cli::cli().await;
}
