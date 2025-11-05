// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use clap::{Args, Subcommand};
use crate::parse::parse_address;
use fvm_shared::address::Address;

#[derive(Args, Debug)]
pub struct BlobArgs {
    #[command(subcommand)]
    pub command: BlobCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum BlobCommands {
    /// Finalize a blob (mark as resolved/failed) - POC mode
    FinalizeBlob {
        /// The URL of the Tendermint node
        #[arg(long, short, default_value = "http://127.0.0.1:26657")]
        url: tendermint_rpc::Url,

        /// Path to the secret key file
        #[arg(long, short)]
        secret_key: PathBuf,

        /// Subscriber address (owner of the blob)
        #[arg(long, value_parser = parse_address)]
        subscriber: Address,

        /// Blob hash (hex string or CID)
        #[arg(long)]
        hash: String,

        /// Subscription ID
        #[arg(long, default_value = "")]
        id: String,

        /// Blob status: resolved (2) or failed (3)
        #[arg(long, default_value = "2")]
        status: u8,

        /// Gas limit for the transaction
        #[arg(long, default_value = "10000000000")]
        gas_limit: u64,
    },
}
