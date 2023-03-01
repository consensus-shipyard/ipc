// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use ipc_agent::cli;

#[tokio::main]
async fn main() {
    cli::cli().await;
}
