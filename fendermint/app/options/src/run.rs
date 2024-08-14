// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use clap::Args;

#[derive(Args, Debug)]
pub struct RunArgs {
    #[arg(long, short, default_value = "127.0.0.1:4919", env = "IROH_RPC_ADDR")]
    pub iroh_addr: String,
}
