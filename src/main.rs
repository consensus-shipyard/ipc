// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
#![feature(try_blocks)]
use fvm_shared::address::{set_current_network, Network};
use ipc_agent::cli;
use num_traits::FromPrimitive;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let network_raw: u8 = std::env::var("LOTUS_NETWORK")
        // default to testnet
        .unwrap_or_else(|_| String::from("1"))
        .parse()
        .unwrap();
    let network = Network::from_u8(network_raw).unwrap();
    log::debug!("using network: {network:?}");
    set_current_network(network);

    cli::cli().await;
}
