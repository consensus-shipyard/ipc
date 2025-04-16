// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Storage path for the iroh node
    #[arg(long, env = "IROH_PATH")]
    pub iroh_path: PathBuf,
    /// The address to bind the iroh (blobs) RPC to
    #[arg(long, env = "IROH_RPC_ADDR")]
    pub iroh_rpc_addr: SocketAddr,
    /// The ipv4 address iroh will bind on
    #[arg(long, env = "IROH_V4_ADDR")]
    pub iroh_v4_addr: Option<SocketAddrV4>,
    /// The ipv6 address iroh will bind on
    #[arg(long, env = "IROH_V6_ADDR")]
    pub iroh_v6_addr: Option<SocketAddrV6>,
}
