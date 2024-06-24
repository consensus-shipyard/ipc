// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use clap::{Args, Subcommand};
use tendermint_rpc::Url;

#[derive(Args, Debug)]
pub struct ObjectsArgs {
    #[command(subcommand)]
    pub command: ObjectsCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ObjectsCommands {
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
            default_value = "127.0.0.1:4919",
            env = "IROH_RPC_ADDR"
        )]
        iroh_addr: String,
    },
}
