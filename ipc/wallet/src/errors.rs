// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::crypto;

#[derive(Debug, thiserror::Error)]
pub enum WalletErr {
    /// info that corresponds to key does not exist
    #[error("Key info not found in keystore")]
    KeyInfo,
    #[error("Key already exists in keystore")]
    KeyExists,
    #[error("Key does not exist in keystore")]
    KeyNotExists,
    #[error("Key not found in keystore")]
    NoKey,

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),

    #[error("Could not convert from KeyInfo to Key")]
    KeyInfoConversion,

    #[error(transparent)]
    Crypto(#[from] crypto::CryptoError),
}
