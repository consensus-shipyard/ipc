// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use clap::Args;

#[derive(Args, Debug)]
pub struct RunArgs {
    #[arg(
        long,
        short,
        default_value = "/ip4/127.0.0.1/tcp/5001",
        env = "IPFS_RPC_ADDR"
    )]
    pub ipfs_addr: String,
}
