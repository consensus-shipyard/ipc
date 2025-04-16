// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::PathBuf;

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

        #[arg(long, short, env = "IROH_PATH")]
        iroh_path: PathBuf,
        /// The rpc address of the resolver iroh (blobs) RPC
        #[arg(long, env = "IROH_RESOLVER_RPC_ADDR")]
        iroh_resolver_rpc_addr: SocketAddr,
        /// The ipv4 address iroh will bind ond
        #[arg(long, env = "IROH_V4_ADDR")]
        iroh_v4_addr: Option<SocketAddrV4>,
        /// The ipv6 address iroh will bind ond
        #[arg(long, env = "IROH_V6_ADDR")]
        iroh_v6_addr: Option<SocketAddrV6>,
    },
}
