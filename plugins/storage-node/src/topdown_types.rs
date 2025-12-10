// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Storage-node specific topdown finality types.
//!
//! Moved from fendermint/vm/topdown/src/lib.rs to achieve plugin isolation.
//! These types are used for voting on storage operations (blob resolution, read requests).

use iroh_blobs::Hash;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// The finality view for IPC blob resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IPCBlobFinality {
    pub hash: Hash,
    pub success: bool,
}

impl IPCBlobFinality {
    pub fn new(hash: Hash, success: bool) -> Self {
        Self { hash, success }
    }
}

impl Display for IPCBlobFinality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IPCBlobFinality(hash: {}, success: {})",
            self.hash, self.success
        )
    }
}

/// The finality view for IPC read request resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IPCReadRequestClosed {
    pub hash: Hash,
}

impl IPCReadRequestClosed {
    pub fn new(hash: Hash) -> Self {
        Self { hash }
    }
}

impl Display for IPCReadRequestClosed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IPCReadRequestClosed(hash: {})", self.hash)
    }
}
