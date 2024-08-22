// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use serde::{Deserialize, Serialize};

/// Object blake3 hash.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hash(pub [u8; 32]);

/// Iroh node public key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PublicKey(pub [u8; 32]);
