// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use clap::{Args, Subcommand};
use fvm_shared::address::Address;
use tendermint_rpc::Url;

use super::parse::{parse_address, parse_cid};

#[derive(Args, Debug)]
pub struct RpcArgs {
    /// The URL of the Tendermint node's RPC endpoint.
    #[arg(
        long,
        short,
        default_value = "http://127.0.0.1:26657",
        env = "TENDERMINT_RPC_URL"
    )]
    pub url: Url,

    /// An optional HTTP/S proxy through which to submit requests to the
    /// Tendermint node's RPC endpoint.
    #[arg(long)]
    pub proxy_url: Option<Url>,

    #[command(subcommand)]
    pub command: RpcCommands,
}

#[derive(Subcommand, Debug)]
pub enum RpcCommands {
    /// Get raw IPLD content; print it as base64 string.
    Query {
        /// Block height to query; 0 means latest.
        #[arg(long, short = 'b', default_value_t = 0)]
        height: u64,
        #[command(subcommand)]
        command: RpcQueryCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum RpcQueryCommands {
    /// Get raw IPLD content; print it as base64 string.
    Ipld {
        /// CID key of the IPLD content to retrieve.
        #[arg(long, short, value_parser = parse_cid)]
        cid: Cid,
    },
    /// Get the state of an actor; print it as JSON.
    ActorState {
        /// Address of the actor to query.
        #[arg(long, short, value_parser = parse_address)]
        address: Address,
    },
}
