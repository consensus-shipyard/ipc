// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Cryptographic key types and operations

use anyhow::{anyhow, ensure};
use ethers::core::k256::ecdsa::{SigningKey, VerifyingKey};
use ethers::core::utils::keccak256;
use ethers::utils::hex;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Key format for serialization/parsing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyFormat {
    /// Compressed public key (33 bytes, starts with 0x02 or 0x03)
    Compressed,
    /// Uncompressed public key (65 bytes, starts with 0x04)
    Uncompressed,
}

/// A secp256k1 private key for cryptographic operations
#[derive(Clone, PartialEq, Eq)]
pub struct PrivateKey {
    // Store the raw key bytes for proper zeroization
    key_bytes: [u8; 32],
    inner: SigningKey,
}

/// A secp256k1 public key (compressed or uncompressed)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PublicKey {
    inner: VerifyingKey,
    format: KeyFormat,
}

impl Zeroize for PrivateKey {
    fn zeroize(&mut self) {
        // Zeroize the raw key bytes
        self.key_bytes.zeroize();
        // Create a new default SigningKey to replace the current one
        // This follows the pattern from fendermint/crypto
        if let Ok(default_key) = SigningKey::from_slice(&[1u8; 32]) {
            let _ = std::mem::replace(&mut self.inner, default_key);
        }
    }
}

impl Drop for PrivateKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

impl ZeroizeOnDrop for PrivateKey {}

impl PrivateKey {
    /// Create a private key from hex string (with or without 0x prefix)
    pub fn from_hex(hex: &str) -> anyhow::Result<Self> {
        let key_hex = hex
            .strip_prefix("0x")
            .or_else(|| hex.strip_prefix("0X"))
            .unwrap_or(hex);

        let private_key_bytes = hex::decode(key_hex)
            .map_err(|e| anyhow!("Failed to decode private key: {}", e))?;

        Self::from_bytes(&private_key_bytes)
    }

    /// Create a private key from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        ensure!(bytes.len() == 32, "Private key must be exactly 32 bytes");

        let signing_key = SigningKey::from_slice(bytes)
            .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(bytes);

        Ok(Self {
            key_bytes,
            inner: signing_key
        })
    }

    /// Generate a random private key
    pub fn generate() -> Self {
        let signing_key = SigningKey::random(&mut rand::thread_rng());
        let key_bytes = signing_key.to_bytes().into();
        Self {
            key_bytes,
            inner: signing_key
        }
    }

    /// Get the public key with specified format
    pub fn public_key(&self, format: KeyFormat) -> PublicKey {
        PublicKey {
            inner: *self.inner.verifying_key(),
            format,
        }
    }

    /// Export as hex string (with 0x prefix)
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.to_bytes()))
    }

    /// Export as 32-byte array
    pub fn to_bytes(&self) -> [u8; 32] {
        self.key_bytes
    }
}

impl PublicKey {
    /// Create a public key from hex string with automatic format detection
    pub fn from_hex(hex: &str) -> anyhow::Result<Self> {
        let key_hex = hex
            .strip_prefix("0x")
            .or_else(|| hex.strip_prefix("0X"))
            .unwrap_or(hex);

        let bytes = hex::decode(key_hex)
            .map_err(|e| anyhow!("Failed to decode public key: {}", e))?;

        Self::from_bytes(&bytes)
    }

    /// Create a public key from bytes with automatic format detection
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let format = match bytes.len() {
            33 => {
                ensure!(
                    bytes[0] == 0x02 || bytes[0] == 0x03,
                    "Invalid compressed public key prefix: expected 0x02 or 0x03, got 0x{:02x}",
                    bytes[0]
                );
                KeyFormat::Compressed
            }
            65 => {
                ensure!(
                    bytes[0] == 0x04,
                    "Invalid uncompressed public key prefix: expected 0x04, got 0x{:02x}",
                    bytes[0]
                );
                KeyFormat::Uncompressed
            }
            _ => anyhow::bail!(
                "Invalid public key length: expected 33 (compressed) or 65 (uncompressed) bytes, got {}",
                bytes.len()
            ),
        };

        // Parse the key using k256
        let verifying_key = VerifyingKey::from_sec1_bytes(bytes)
            .map_err(|e| anyhow!("Failed to parse public key: {}", e))?;

        Ok(Self {
            inner: verifying_key,
            format,
        })
    }

    /// Convert to different format (compressed <-> uncompressed)
    pub fn with_format(&self, format: KeyFormat) -> Self {
        Self {
            inner: self.inner,
            format,
        }
    }

    /// Export as hex string (with 0x prefix)
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.to_bytes()))
    }

    /// Export as bytes in the current format
    pub fn to_bytes(&self) -> Vec<u8> {
        let compressed = matches!(self.format, KeyFormat::Compressed);
        let encoded_point = self.inner.to_encoded_point(compressed);
        encoded_point.as_bytes().to_vec()
    }

    /// Check if this public key is in compressed format
    pub fn is_compressed(&self) -> bool {
        matches!(self.format, KeyFormat::Compressed)
    }

    /// Get the format of this public key
    pub fn format(&self) -> KeyFormat {
        self.format
    }
}

// String conversions
impl FromStr for PrivateKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl FromStr for PublicKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// Type conversions between keys
impl From<&PrivateKey> for PublicKey {
    fn from(private_key: &PrivateKey) -> Self {
        private_key.public_key(KeyFormat::Uncompressed)
    }
}

impl From<PrivateKey> for PublicKey {
    fn from(private_key: PrivateKey) -> Self {
        private_key.public_key(KeyFormat::Uncompressed)
    }
}

// Conversions to EthAddress
impl From<PrivateKey> for crate::EthAddress {
    fn from(private_key: PrivateKey) -> Self {
        let public_key = private_key.public_key(KeyFormat::Uncompressed);
        public_key.into()
    }
}

impl From<&PrivateKey> for crate::EthAddress {
    fn from(private_key: &PrivateKey) -> Self {
        let public_key = private_key.public_key(KeyFormat::Uncompressed);
        public_key.into()
    }
}

impl From<PublicKey> for crate::EthAddress {
    fn from(public_key: PublicKey) -> Self {
        // Convert to uncompressed format if needed
        let uncompressed = public_key.with_format(KeyFormat::Uncompressed);
        let public_key_bytes = uncompressed.to_bytes();

        // Skip the 0x04 prefix and hash the remaining 64 bytes
        let hash = keccak256(&public_key_bytes[1..]);
        let mut address = [0u8; 20];
        address.copy_from_slice(&hash[12..]);
        crate::EthAddress(address)
    }
}

impl From<&PublicKey> for crate::EthAddress {
    fn from(public_key: &PublicKey) -> Self {
        // Convert to uncompressed format if needed
        let uncompressed = public_key.with_format(KeyFormat::Uncompressed);
        let public_key_bytes = uncompressed.to_bytes();

        // Skip the 0x04 prefix and hash the remaining 64 bytes
        let hash = keccak256(&public_key_bytes[1..]);
        let mut address = [0u8; 20];
        address.copy_from_slice(&hash[12..]);
        crate::EthAddress(address)
    }
}

// Debug impl that doesn't expose private key
impl std::fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrivateKey")
            .field("public_key", &PublicKey::from(self))
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_key_from_hex() {
        let private_key = PrivateKey::from_hex("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80").unwrap();
        assert_eq!(private_key.to_hex(), "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80");
    }

    #[test]
    fn test_private_key_from_hex_without_prefix() {
        let hex_without_prefix = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let private_key = PrivateKey::from_hex(hex_without_prefix).unwrap();
        assert_eq!(private_key.to_hex(), "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80");
    }

    #[test]
    fn test_private_key_generation() {
        let key1 = PrivateKey::generate();
        let key2 = PrivateKey::generate();
        assert_ne!(key1.to_hex(), key2.to_hex());
    }

    #[test]
    fn test_public_key_formats() {
        let private_key = PrivateKey::from_hex("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80").unwrap();

        let compressed = private_key.public_key(KeyFormat::Compressed);
        let uncompressed = private_key.public_key(KeyFormat::Uncompressed);

        assert!(compressed.is_compressed());
        assert!(!uncompressed.is_compressed());
        assert_eq!(compressed.to_bytes().len(), 33);
        assert_eq!(uncompressed.to_bytes().len(), 65);
    }

    #[test]
    fn test_public_key_format_conversion() {
        let private_key = PrivateKey::from_hex("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80").unwrap();
        let compressed = private_key.public_key(KeyFormat::Compressed);
        let uncompressed = compressed.with_format(KeyFormat::Uncompressed);

        assert!(compressed.is_compressed());
        assert!(!uncompressed.is_compressed());

        // Converting back should give the same result
        let compressed_again = uncompressed.with_format(KeyFormat::Compressed);
        assert_eq!(compressed.to_hex(), compressed_again.to_hex());
    }

    #[test]
    fn test_public_key_from_hex_detection() {
        let private_key = PrivateKey::from_hex("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80").unwrap();

        // Generate actual corresponding public keys
        let compressed_key = private_key.public_key(KeyFormat::Compressed);
        let uncompressed_key = private_key.public_key(KeyFormat::Uncompressed);

        // Test parsing them back
        let parsed_compressed = PublicKey::from_hex(&compressed_key.to_hex()).unwrap();
        let parsed_uncompressed = PublicKey::from_hex(&uncompressed_key.to_hex()).unwrap();

        assert_eq!(parsed_compressed.format(), KeyFormat::Compressed);
        assert_eq!(parsed_uncompressed.format(), KeyFormat::Uncompressed);
    }

    #[test]
    fn test_string_conversions() {
        let private_key: PrivateKey = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".parse().unwrap();
        assert_eq!(private_key.to_string(), "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80");

        let public_key = private_key.public_key(KeyFormat::Compressed);
        let public_key_parsed: PublicKey = public_key.to_string().parse().unwrap();
        assert_eq!(public_key.to_string(), public_key_parsed.to_string());
    }

    #[test]
    fn test_private_to_public_conversion() {
        let private_key = PrivateKey::from_hex("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80").unwrap();
        let public_key: PublicKey = private_key.into();

        // Should default to uncompressed
        assert!(!public_key.is_compressed());
        assert_eq!(public_key.to_bytes().len(), 65);
    }

    #[test]
    fn test_private_key_to_eth_address() {
        let private_key = PrivateKey::from_hex("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80").unwrap();
        let eth_address: crate::EthAddress = private_key.clone().into();

        // The address should be deterministic for the same private key
        let eth_address2: crate::EthAddress = (&private_key).into();
        assert_eq!(eth_address, eth_address2);
    }

    #[test]
    fn test_public_key_to_eth_address() {
        let private_key = PrivateKey::from_hex("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80").unwrap();
        let public_key_compressed = private_key.public_key(KeyFormat::Compressed);
        let public_key_uncompressed = private_key.public_key(KeyFormat::Uncompressed);

        // Both compressed and uncompressed should give the same address
        let addr1: crate::EthAddress = public_key_compressed.clone().into();
        let addr2: crate::EthAddress = public_key_uncompressed.into();
        let addr3: crate::EthAddress = (&public_key_compressed).into();

        assert_eq!(addr1, addr2);
        assert_eq!(addr1, addr3);
    }

    #[test]
    fn test_consistent_address_derivation() {
        let private_key = PrivateKey::from_hex("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80").unwrap();

        // All these should give the same address
        let addr_from_private: crate::EthAddress = (&private_key).into();
        let public_key = PublicKey::from(&private_key);
        let addr_from_public: crate::EthAddress = public_key.into();

        assert_eq!(addr_from_private, addr_from_public);
    }

    #[test]
    fn test_zeroization() {
        // Create a private key
        let private_key = PrivateKey::from_hex("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80").unwrap();

        // Extract the raw bytes before zeroization
        let original_bytes = private_key.to_bytes();
        assert_ne!(original_bytes, [0u8; 32], "Original key should not be all zeros");

        // Clone the key to test zeroization
        let mut key_to_zeroize = private_key.clone();

        // Manually call zeroize
        key_to_zeroize.zeroize();

        // Check that the key bytes have been zeroized
        let zeroized_bytes = key_to_zeroize.to_bytes();
        assert_eq!(zeroized_bytes, [0u8; 32], "Key bytes should be zeroed after zeroization");

        // Original key should still be intact
        assert_eq!(private_key.to_bytes(), original_bytes, "Original key should remain unchanged");
    }

    #[test]
    fn test_drop_calls_zeroize() {
        let original_bytes = {
            let private_key = PrivateKey::from_hex("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80").unwrap();
            let bytes = private_key.to_bytes();
            assert_ne!(bytes, [0u8; 32], "Key should not be all zeros");
            // private_key is dropped here, which should call zeroize
            bytes
        };

        // This test mainly ensures that Drop is implemented and doesn't panic
        // The actual memory zeroization is hard to test directly due to memory reuse
        assert_ne!(original_bytes, [0u8; 32], "Original bytes should not be all zeros");
    }
}