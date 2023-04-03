// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use cid::Cid;
use clap::{Args, Subcommand, ValueEnum};
use fvm_shared::{address::Address, econ::TokenAmount};
use tendermint_rpc::Url;

use crate::options::parse::{parse_address, parse_cid, parse_token_amount};

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
    /// Send an ABCI query.
    Query {
        /// Block height to query; 0 means latest.
        #[arg(long, short = 'b', default_value_t = 0)]
        height: u64,
        #[command(subcommand)]
        command: RpcQueryCommands,
    },
    /// Transfer tokens between accounts.
    Transfer {
        #[command(flatten)]
        args: TransArgs,
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

/// Arguments common to transactions and transfers.
#[derive(Args, Debug)]
pub struct TransArgs {
    /// Path to the secret key of the sender to sign the transaction.
    #[arg(long, short)]
    pub secret_key: PathBuf,
    /// Address of the actor to send the message to.
    #[arg(long, short, value_parser = parse_address)]
    pub to: Address,
    /// Amount of tokens to send, in atto.
    #[arg(long, short, value_parser = parse_token_amount, default_value = "0")]
    pub value: TokenAmount,
    /// Sender account nonce.
    #[arg(long, short = 'n')]
    pub sequence: u64,
    /// Maximum amount of gas that can be charged.
    #[arg(long, default_value_t = 10_000_000_000)] // Default from ref-fvm testkit.
    pub gas_limit: u64,
    /// Price of gas.
    ///
    /// Any discrepancy between this and the base fee is paid for
    /// by the validator who puts the transaction into the block.
    #[arg(long, value_parser = parse_token_amount, default_value = "0")]
    pub gas_fee_cap: TokenAmount,
    /// Gas premium.
    #[arg(long, value_parser = parse_token_amount, default_value = "0")]
    pub gas_premium: TokenAmount,
    /// Whether to wait for the results from Tendermint or not.
    #[arg(long, short, default_value = "commit")]
    pub broadcast_mode: BroadcastMode,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum BroadcastMode {
    /// Do no wait for the results.
    Async,
    /// Wait for the result of `check_tx`.
    Sync,
    /// Wait for the result of `deliver_tx`.
    Commit,
}
