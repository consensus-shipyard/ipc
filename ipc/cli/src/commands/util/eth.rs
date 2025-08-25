// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Ethereum utilities for address conversion and key derivation

use async_trait::async_trait;
use clap::Args;
use ethers::types::H160;
use fvm_shared::address::Address;
use ipc_api::evm::payload_to_evm_address;
use ipc_types::{EthAddress, KeyFormat, PrivateKey};
use std::fmt::Debug;
use std::str::FromStr;

use crate::{CommandLineHandler, GlobalArguments};

pub(crate) struct F4ToEthAddr;

#[async_trait]
impl CommandLineHandler for F4ToEthAddr {
    type Arguments = F4ToEthAddrArgs;

    async fn handle(_global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        let addr = Address::from_str(&arguments.addr)?;
        let eth_addr = payload_to_evm_address(addr.payload())?;
        log::info!("eth address: {:?}", eth_addr);
        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Get Ethereum address for an F4")]
pub(crate) struct F4ToEthAddrArgs {
    #[arg(long, help = "F4 address to get the underlying Ethereum addr from")]
    pub addr: String,
}

pub(crate) struct DerivePublicKey;

#[async_trait]
impl CommandLineHandler for DerivePublicKey {
    type Arguments = DerivePublicKeyArgs;

    async fn handle(_global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        // Parse the private key using the new type-safe approach
        let private_key = PrivateKey::from_hex(&arguments.private_key)?;

        // Generate public key in the requested format
        let format = if arguments.compressed {
            KeyFormat::Compressed
        } else {
            KeyFormat::Uncompressed
        };
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
#[command(about = "Derive public key from private key")]
pub(crate) struct DerivePublicKeyArgs {
    #[arg(help = "Private key to derive public key from (hex string, with or without 0x prefix)")]
    pub private_key: String,

    #[arg(
        long,
        help = "Output compressed public key (33 bytes) instead of uncompressed (65 bytes)"
    )]
    pub compressed: bool,

    #[arg(long, help = "Also show the corresponding Ethereum address")]
    pub show_address: bool,
}
