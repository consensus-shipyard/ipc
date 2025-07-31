// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Key conversion utilities for converting private keys to public keys

use async_trait::async_trait;
use clap::Args;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::core::utils::keccak256;
use hex;
use std::fmt::Debug;

use crate::{CommandLineHandler, GlobalArguments};

pub(crate) struct ConvertKey;

#[async_trait]
impl CommandLineHandler for ConvertKey {
    type Arguments = ConvertKeyArgs;

        async fn handle(_global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        // Remove 0x prefix if present
        let key_hex = arguments.private_key.strip_prefix("0x").unwrap_or(&arguments.private_key);

        // Decode hex to bytes
        let private_key_bytes = hex::decode(key_hex)
            .map_err(|e| anyhow::anyhow!("Failed to decode private key: {}", e))?;

        // Parse as secp256k1 secret key using ethers k256 for better public key handling
        let signing_key = SigningKey::from_slice(&private_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;

        let verifying_key = signing_key.verifying_key();

        if arguments.compressed {
            // Output compressed public key (33 bytes)
            let compressed_point = verifying_key.to_encoded_point(true);
            let public_key_hex = hex::encode(compressed_point.as_bytes());
            println!("0x{}", public_key_hex);
        } else {
            // Output uncompressed public key (65 bytes) - default
            let uncompressed_point = verifying_key.to_encoded_point(false);
            let public_key_hex = hex::encode(uncompressed_point.as_bytes());
            println!("0x{}", public_key_hex);
        }

        // Optionally show Ethereum address
        if arguments.show_address {
            let uncompressed_point = verifying_key.to_encoded_point(false);
            let public_key_bytes = uncompressed_point.as_bytes();
            let eth_address = ethereum_address_from_public_key(&public_key_bytes[1..]);
            println!("Ethereum Address: 0x{}", hex::encode(eth_address));
        }

        Ok(())
    }
}

/// Calculate Ethereum address from uncompressed public key (without 0x04 prefix)
fn ethereum_address_from_public_key(public_key_bytes: &[u8]) -> [u8; 20] {
    let hash = keccak256(public_key_bytes);
    let mut address = [0u8; 20];
    address.copy_from_slice(&hash[12..]);
    address
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