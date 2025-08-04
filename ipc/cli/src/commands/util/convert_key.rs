// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Key conversion utilities for converting private keys to public keys

use async_trait::async_trait;
use clap::Args;
use ethers::types::H160;
use ipc_types::{PrivateKey, KeyFormat, EthAddress};
use std::fmt::Debug;

use crate::{CommandLineHandler, GlobalArguments};

pub(crate) struct ConvertKey;

#[async_trait]
impl CommandLineHandler for ConvertKey {
    type Arguments = ConvertKeyArgs;

    async fn handle(_global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        // Parse the private key using the new type-safe approach
        let private_key = PrivateKey::from_hex(&arguments.private_key)?;

        // Generate public key in the requested format
        let format = if arguments.compressed { KeyFormat::Compressed } else { KeyFormat::Uncompressed };
        let public_key = private_key.public_key(format);

        // Print the public key
        println!("{}", public_key.to_hex());

        // Optionally show Ethereum address
        if arguments.show_address {
            let eth_address: EthAddress = private_key.into();
            println!("Ethereum Address: {:#x}", H160::from(eth_address.0));
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Convert private key to public key")]
pub(crate) struct ConvertKeyArgs {
    #[arg(help = "Private key to convert (hex string, with or without 0x prefix)")]
    pub private_key: String,

    #[arg(long, help = "Output compressed public key (33 bytes) instead of uncompressed (65 bytes)")]
    pub compressed: bool,

    #[arg(long, help = "Also show the corresponding Ethereum address")]
    pub show_address: bool,
}