// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::ActorError;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, clock::ChainEpoch};
use recall_ipld::hamt::MapKey;
use serde::{Deserialize, Serialize};

use crate::bytes::B256;

/// An object used to determine what [`Account`](s) are accountable for a blob, and for how long.
/// Subscriptions allow us to distribute the cost of a blob across multiple accounts that
/// have added the same blob.   
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Subscription {
    /// Added block.
    pub added: ChainEpoch,
    /// Expiry block.
    pub expiry: ChainEpoch,
    /// Source Iroh node ID used for ingestion.
    /// This might be unique to each instance of the same blob.
    /// It's included here for record keeping.
    pub source: B256,
    /// The delegate origin that may have created the subscription via a credit approval.
    pub delegate: Option<Address>,
    /// Whether the subscription failed due to an issue resolving the target blob.
    pub failed: bool,
}

/// User-defined identifier used to differentiate blob subscriptions for the same subscriber.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId {
    inner: String,
}

impl SubscriptionId {
    /// Max ID length.
    pub const MAX_LEN: usize = 64;

    /// Returns a new [`SubscriptionId`].
    pub fn new(value: &str) -> Result<Self, ActorError> {
        if value.len() > Self::MAX_LEN {
            return Err(ActorError::illegal_argument(format!(
                "subscription ID length is {} but must not exceed the maximum of {} characters",
                value.len(),
                Self::MAX_LEN
            )));
        }
        Ok(Self {
            inner: value.to_string(),
        })
    }
}

impl From<SubscriptionId> for String {
    fn from(id: SubscriptionId) -> String {
        id.inner
    }
}

impl TryFrom<String> for SubscriptionId {
    type Error = ActorError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.is_empty() {
            write!(f, "default")
        } else {
            write!(f, "{}", self.inner)
        }
    }
}

impl MapKey for SubscriptionId {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        let inner = String::from_utf8(b.to_vec()).map_err(|e| e.to_string())?;
        Self::new(&inner).map_err(|e| e.to_string())
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.inner.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_id_length() {
        let id_str = |len: usize| "a".repeat(len);
        let id = SubscriptionId::new(&id_str(SubscriptionId::MAX_LEN)).unwrap();
        assert_eq!(id.inner, id_str(SubscriptionId::MAX_LEN));

        let id = SubscriptionId::new(&id_str(SubscriptionId::MAX_LEN + 1));
        assert!(id.is_err());
    }
}
