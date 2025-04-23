// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

/// The status of an account.
/// This controls the max TTL that the user is allowed to set on their blobs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AccountStatus {
    // Default TTL.
    #[default]
    Default,
    /// Reduced TTL.
    Reduced,
    /// Extended TTL.
    Extended,
}

impl AccountStatus {
    /// Returns the max allowed TTL.
    pub fn get_max_ttl(&self, default_max_ttl: ChainEpoch) -> ChainEpoch {
        match self {
            AccountStatus::Default => default_max_ttl,
            AccountStatus::Reduced => 0,
            AccountStatus::Extended => ChainEpoch::MAX,
        }
    }
}

impl std::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountStatus::Default => write!(f, "default"),
            AccountStatus::Reduced => write!(f, "reduced"),
            AccountStatus::Extended => write!(f, "extended"),
        }
    }
}
