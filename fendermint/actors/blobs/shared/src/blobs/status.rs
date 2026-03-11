// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use serde::{Deserialize, Serialize};

/// The status of a blob.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlobStatus {
    /// Blob is added but not resolving.
    #[default]
    Added,
    /// Blob is pending resolve.
    Pending,
    /// Blob was successfully resolved.
    Resolved,
    /// Blob resolution failed.
    Failed,
}

impl std::fmt::Display for BlobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlobStatus::Added => write!(f, "added"),
            BlobStatus::Pending => write!(f, "pending"),
            BlobStatus::Resolved => write!(f, "resolved"),
            BlobStatus::Failed => write!(f, "failed"),
        }
    }
}
