// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use clap::{Args, Subcommand};
use tendermint_rpc::Url;

#[derive(Args, Debug)]
pub struct ObjectAPIArgs {
    #[command(subcommand)]
    pub command: ObjectAPICommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ObjectAPICommands {
    Run {
        /// The URL of the Tendermint node's RPC endpoint.
        #[arg(
            long,
            short,
            default_value = "http://127.0.0.1:26657",
            env = "TENDERMINT_RPC_URL"
        )]
        tendermint_url: Url,

        #[arg(
            long,
            short,
            default_value = "/ip4/127.0.0.1/tcp/5001",
            env = "IPFS_RPC_ADDR"
        )]
        ipfs_addr: String,

        #[command(flatten)]
        args: super::rpc::TransArgs,
    },
}
