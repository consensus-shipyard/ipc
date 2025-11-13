// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::anyhow;
use data_encoding::{DecodeError, DecodeKind};
use recall_ipld::hamt::MapKey;
use serde::{Deserialize, Serialize};

/// Container for 256 bits or 32 bytes.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct B256(pub [u8; 32]);

impl AsRef<[u8]> for B256 {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl From<[u8; 32]> for B256 {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<B256> for [u8; 32] {
    fn from(value: B256) -> Self {
        value.0
    }
}

impl From<&[u8; 32]> for B256 {
    fn from(value: &[u8; 32]) -> Self {
        Self(*value)
    }
}

impl TryFrom<&[u8]> for B256 {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() == 32 {
            let mut array = [0u8; 32];
            array.copy_from_slice(slice);
            Ok(Self(array))
        } else {
            Err(anyhow!("hash slice must be exactly 32 bytes"))
        }
    }
}

impl From<u64> for B256 {
    fn from(value: u64) -> Self {
        let mut padded = [0u8; 32];
        padded[24..].copy_from_slice(&value.to_be_bytes());
        Self(padded)
    }
}

impl std::str::FromStr for B256 {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_array::<32>(s)
            .map(Self::from)
            .map_err(|e| anyhow::anyhow!(e))
    }
}

/// Parse from a base32 string into a byte array
fn parse_array<const N: usize>(input: &str) -> Result<[u8; N], DecodeError> {
    data_encoding::BASE32_NOPAD
        .decode(input.to_ascii_uppercase().as_bytes())?
        .try_into()
        .map_err(|_| DecodeError {
            position: N,
            kind: DecodeKind::Length,
        })
}

impl std::fmt::Display for B256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut t = data_encoding::BASE32_NOPAD.encode(self.as_ref());
        t.make_ascii_lowercase();
        f.write_str(&t)
    }
}

impl MapKey for B256 {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        b.try_into().map_err(|e: anyhow::Error| e.to_string())
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.0.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_display_parse_roundtrip() {
        for i in 0..100 {
            let b: B256 = blake3::hash(&[i]).as_bytes().into();
            let text = b.to_string();
            let b1 = text.parse::<B256>().unwrap();
            let b2 = B256::from_str(&text).unwrap();
            assert_eq!(b, b1);
            assert_eq!(b, b2);
        }
    }
}
