// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Cryptographic utilities for key conversion and address derivation
//!
//! **DEPRECATED**: This module contains legacy utility functions.
//! New code should use the key types directly from `ipc_types`:
//! - `ipc_types::PrivateKey`
//! - `ipc_types::PublicKey`
//! - `ipc_types::EthAddress`

use ethers::core::utils::keccak256;
use ethers::types::H160;
use ipc_types::{PrivateKey, PublicKey, KeyFormat, EthAddress};

/// Convert a private key (hex string) to a public key (compressed or uncompressed)
///
/// **DEPRECATED**: Use `PrivateKey::from_hex()` and `PrivateKey::public_key()` instead.
///
/// # Arguments
/// * `private_key_hex` - Private key as hex string (with or without 0x prefix)
/// * `compressed` - Whether to return compressed (33 bytes) or uncompressed (65 bytes) public key
///
/// # Returns
/// * Public key as hex string with 0x prefix
#[deprecated(note = "Use ipc_types::PrivateKey::from_hex() and PrivateKey::public_key() instead")]
pub fn private_key_to_public_key(private_key_hex: &str, compressed: bool) -> anyhow::Result<String> {
    let private_key = PrivateKey::from_hex(private_key_hex)?;
    let format = if compressed { KeyFormat::Compressed } else { KeyFormat::Uncompressed };
    Ok(private_key.public_key(format).to_hex())
}

/// Convert an uncompressed secp256k1 public key (hex string) into an Ethereum address
///
/// **DEPRECATED**: Use `PublicKey::from_hex()` and convert to `EthAddress` instead.
///
/// This function provides robust validation and error handling.
///
/// # Arguments
/// * `public_key_hex` - Public key as hex string (with or without 0x prefix)
///
/// # Returns
/// * Ethereum address as H160
#[deprecated(note = "Use ipc_types::PublicKey::from_hex() and convert to EthAddress instead")]
pub fn public_key_to_ethereum_address(public_key_hex: &str) -> anyhow::Result<H160> {
    let public_key = PublicKey::from_hex(public_key_hex)?;
    let eth_address: EthAddress = public_key.into();
    Ok(H160::from(eth_address.0))
}

/// Convert a private key (hex string) to an Ethereum address
///
/// **DEPRECATED**: Use `PrivateKey::from_hex()` and convert to `EthAddress` instead.
///
/// # Arguments
/// * `private_key_hex` - Private key as hex string (with or without 0x prefix)
///
/// # Returns
/// * Ethereum address as H160
#[deprecated(note = "Use ipc_types::PrivateKey::from_hex() and convert to EthAddress instead")]
pub fn private_key_to_ethereum_address(private_key_hex: &str) -> anyhow::Result<H160> {
    let private_key = PrivateKey::from_hex(private_key_hex)?;
    let eth_address: EthAddress = private_key.into();
    Ok(H160::from(eth_address.0))
}

/// Check if a public key hex string represents a compressed key
///
/// **DEPRECATED**: Use `PublicKey::from_hex()` and `PublicKey::is_compressed()` instead.
///
/// # Arguments
/// * `public_key_hex` - Public key as hex string (with or without 0x prefix)
///
/// # Returns
/// * true if compressed, false if uncompressed
#[deprecated(note = "Use ipc_types::PublicKey::from_hex() and PublicKey::is_compressed() instead")]
pub fn is_compressed_public_key(public_key_hex: &str) -> anyhow::Result<bool> {
    let public_key = PublicKey::from_hex(public_key_hex)?;
    Ok(public_key.is_compressed())
}

/// Low-level function: Convert raw public key bytes to Ethereum address
///
/// **DEPRECATED**: Use the type-safe alternatives instead.
///
/// # Arguments
/// * `public_key_bytes` - Uncompressed public key bytes (should start with 0x04)
///
/// # Returns
/// * Ethereum address as 20-byte array
#[deprecated(note = "Use ipc_types key types for type-safe operations")]
pub fn ethereum_address_from_public_key(public_key_bytes: &[u8]) -> [u8; 20] {
    let hash = keccak256(public_key_bytes);
    let mut address = [0u8; 20];
    address.copy_from_slice(&hash[12..]);
    address
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    #[test]
    #[allow(deprecated)]
    fn test_backward_compatibility() {
        // Test that deprecated functions still work
        let public_key_uncompressed = private_key_to_public_key(TEST_PRIVATE_KEY, false).unwrap();
        let public_key_compressed = private_key_to_public_key(TEST_PRIVATE_KEY, true).unwrap();

        assert!(public_key_uncompressed.len() > public_key_compressed.len());

        let eth_address1 = private_key_to_ethereum_address(TEST_PRIVATE_KEY).unwrap();
        let eth_address2 = public_key_to_ethereum_address(&public_key_uncompressed).unwrap();

        assert_eq!(eth_address1, eth_address2);

        assert!(!is_compressed_public_key(&public_key_uncompressed).unwrap());
        assert!(is_compressed_public_key(&public_key_compressed).unwrap());
    }

    #[test]
    fn test_new_types_consistency() {
        // Test that new types give same results as old functions
        let private_key = PrivateKey::from_hex(TEST_PRIVATE_KEY).unwrap();

        let new_compressed = private_key.public_key(KeyFormat::Compressed).to_hex();
        let new_uncompressed = private_key.public_key(KeyFormat::Uncompressed).to_hex();

        #[allow(deprecated)]
        {
            let old_compressed = private_key_to_public_key(TEST_PRIVATE_KEY, true).unwrap();
            let old_uncompressed = private_key_to_public_key(TEST_PRIVATE_KEY, false).unwrap();

            assert_eq!(new_compressed, old_compressed);
            assert_eq!(new_uncompressed, old_uncompressed);
        }

        let new_eth_addr: EthAddress = private_key.into();

        #[allow(deprecated)]
        {
            let old_eth_addr = private_key_to_ethereum_address(TEST_PRIVATE_KEY).unwrap();
            assert_eq!(H160::from(new_eth_addr.0), old_eth_addr);
        }
    }
}