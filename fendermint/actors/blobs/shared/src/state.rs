// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::ActorError;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use hoku_ipld::hamt::MapKey;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
/// The stored representation of a credit account.
#[derive(Clone, Debug, Default, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Account {
    /// Total size of all blobs managed by the account.
    pub capacity_used: u64,
    /// Current free credit in byte-blocks that can be used for new commitments.
    pub credit_free: BigInt,
    /// Current committed credit in byte-blocks that will be used for debits.
    pub credit_committed: BigInt,
    /// Optional default sponsor account address.
    pub credit_sponsor: Option<Address>,
    /// The chain epoch of the last debit.
    pub last_debit_epoch: ChainEpoch,
    /// Credit approvals to other accounts, keyed by receiver, keyed by caller,
    /// which could be the receiver or a specific contract, like a bucket.
    /// This allows for limiting approvals to interactions from a specific contract.
    /// For example, an approval for Alice might be valid for any contract caller, so long as
    /// the origin is Alice.
    /// An approval for Bob might be valid from only one contract caller, so long as
    /// the origin is Bob.
    pub approvals: HashMap<String, CreditApproval>,
    /// The maximum allowed TTL for actor's blobs.
    pub max_ttl: ChainEpoch,
}

impl Account {
    pub fn new(current_epoch: ChainEpoch) -> Self {
        Self {
            last_debit_epoch: current_epoch,
            max_ttl: TtlStatus::DEFAULT_MAX_TTL,
            ..Default::default()
        }
    }
}

/// A credit approval from one account to another.
#[derive(Debug, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct CreditApproval {
    /// Optional credit approval limit.
    pub limit: Option<BigInt>,
    /// Optional credit approval expiry epoch.
    pub expiry: Option<ChainEpoch>,
    /// Counter for how much credit has been used via this approval.
    pub used: BigInt,
    /// Optional caller allowlist.
    /// If not present, any caller is allowed.
    pub caller_allowlist: Option<HashSet<Address>>,
}

impl CreditApproval {
    pub fn remove_caller(&mut self, caller: &Address) -> bool {
        if let Some(allowlist) = self.caller_allowlist.as_mut() {
            allowlist.remove(caller)
        } else {
            false
        }
    }

    pub fn has_allowlist(&self) -> bool {
        if let Some(allowlist) = self.caller_allowlist.as_ref() {
            !allowlist.is_empty()
        } else {
            false
        }
    }

    pub fn is_caller_allowed(&self, caller: &Address) -> bool {
        if let Some(allowlist) = self.caller_allowlist.as_ref() {
            allowlist.contains(caller)
        } else {
            true
        }
    }
}

/// Credit allowance for an account.
#[derive(Debug, Default, Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct CreditAllowance {
    /// The amount from the account.
    pub amount: TokenAmount,
    /// The account's default sponsor.
    pub sponsor: Option<Address>,
    /// The amount from the account's default sponsor.
    pub sponsored_amount: TokenAmount,
}

impl CreditAllowance {
    /// Returns the total allowance from self and default sponsor.
    pub fn total(&self) -> TokenAmount {
        &self.amount + &self.sponsored_amount
    }
}

/// Blob blake3 hash.
#[derive(
    Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Hash(pub [u8; 32]);

impl TryInto<Hash> for &[u8] {
    type Error = String;

    fn try_into(self) -> Result<Hash, Self::Error> {
        if self.len() == 32 {
            let mut array = [0u8; 32];
            array.copy_from_slice(self);
            Ok(Hash(array))
        } else {
            Err("hash slice must be exactly 32 bytes".into())
        }
    }
}

impl MapKey for Hash {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        b.try_into()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.0.to_vec())
    }
}

/// Source https://github.com/n0-computer/iroh/blob/main/iroh-base/src/hash.rs
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // the result will be 52 bytes
        let mut res = [b'b'; 52];
        // write the encoded bytes
        data_encoding::BASE32_NOPAD.encode_mut(self.0.as_slice(), &mut res);
        // convert to string, this is guaranteed to succeed
        let t = std::str::from_utf8_mut(res.as_mut()).unwrap();
        // hack since data_encoding doesn't have BASE32LOWER_NOPAD as a const
        t.make_ascii_lowercase();
        // write the str, no allocations
        f.write_str(t)
    }
}

impl TryFrom<&str> for Hash {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut res = [0u8; 32];
        data_encoding::BASE32_NOPAD
            .decode_mut(value.as_bytes(), &mut res)
            .map_err(|_| anyhow::anyhow!("invalid hash"))?;
        Ok(Self(res))
    }
}

/// Iroh node public key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PublicKey(pub [u8; 32]);

/// The stored representation of a blob.
#[derive(Clone, PartialEq, Debug, Default, Serialize_tuple, Deserialize_tuple)]
pub struct Blob {
    /// The size of the content.
    pub size: u64,
    /// Blob metadata that contains information for blob recovery.
    pub metadata_hash: Hash,
    /// Active subscribers (accounts) that are paying for the blob.
    pub subscribers: HashMap<String, SubscriptionGroup>,
    /// Blob status.
    pub status: BlobStatus,
}

/// An object used to determine what [`Account`](s) are accountable for a blob, and for how long.
/// Subscriptions allow us to distribute the cost of a blob across multiple accounts that
/// have added the same blob.   
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Subscription {
    /// Added block.
    pub added: ChainEpoch,
    /// Expiry block.
    pub expiry: ChainEpoch,
    /// Whether to automatically renew the subscription.
    pub auto_renew: bool,
    /// Source Iroh node ID used for ingestion.
    /// This might be unique to each instance of the same blob.
    /// It's included here for record keeping.
    pub source: PublicKey,
    /// The delegate origin and caller that may have created the subscription via a credit approval.
    pub delegate: Option<(Address, Address)>,
    /// Whether the subscription failed due to an issue resolving the target blob.
    pub failed: bool,
}

/// User-defined identifier used to differentiate blob subscriptions for the same subscriber.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId {
    inner: String,
}

impl SubscriptionId {
    pub const MAX_LEN: usize = 64;

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

impl TryFrom<String> for SubscriptionId {
    type Error = ActorError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

/// A group of subscriptions for the same subscriber.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubscriptionGroup {
    /// Subscription group keys.
    pub subscriptions: HashMap<String, Subscription>,
}

impl SubscriptionGroup {
    /// Returns the current group max expiry and the group max expiry after adding the provided ID
    /// and new value.
    pub fn max_expiries(
        &self,
        target_id: &SubscriptionId,
        new_value: Option<ChainEpoch>,
    ) -> (Option<ChainEpoch>, Option<ChainEpoch>) {
        let mut max = None;
        let mut new_max = None;
        for (id, sub) in self.subscriptions.iter() {
            if sub.failed {
                continue;
            }
            if sub.expiry > max.unwrap_or(0) {
                max = Some(sub.expiry);
            }
            let new_value = if *id == target_id.to_string() {
                new_value.unwrap_or_default()
            } else {
                sub.expiry
            };
            if new_value > new_max.unwrap_or(0) {
                new_max = Some(new_value);
            }
        }
        // Target ID may not be in the current group
        if let Some(new_value) = new_value {
            if new_value > new_max.unwrap_or(0) {
                new_max = Some(new_value);
            }
        }
        (max, new_max)
    }

    /// Returns whether the provided ID corresponds to a subscription that has the minimum
    /// added epoch and the next minimum added epoch in the group.
    pub fn is_min_added(
        &self,
        trim_id: &SubscriptionId,
    ) -> anyhow::Result<(bool, Option<ChainEpoch>), ActorError> {
        let tid = trim_id.to_string();
        let trim = self
            .subscriptions
            .get(&tid)
            .ok_or(ActorError::not_found(format!(
                "subscription id {} not found",
                trim_id
            )))?;
        let mut next_min = None;
        for (id, sub) in self.subscriptions.iter() {
            if sub.failed || *id == tid {
                continue;
            }
            if sub.added < trim.added {
                return Ok((false, None));
            }
            if sub.added < next_min.unwrap_or(ChainEpoch::MAX) {
                next_min = Some(sub.added);
            }
        }
        Ok((true, next_min))
    }
}

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

impl fmt::Display for BlobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlobStatus::Added => write!(f, "added"),
            BlobStatus::Pending => write!(f, "pending"),
            BlobStatus::Resolved => write!(f, "resolved"),
            BlobStatus::Failed => write!(f, "failed"),
        }
    }
}

/// The TTL status of an account. This controls the max TTL that the user is allowed to set on their blobs.  
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum TtlStatus {
    // Default TTL.
    #[default]
    Default,
    /// Reduced TTL.
    Reduced,
    /// Extended TTL.
    Extended,
}

impl TtlStatus {
    pub const DEFAULT_MAX_TTL: ChainEpoch = 60 * 60 * 24; // 1 day
}

impl fmt::Display for TtlStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TtlStatus::Default => write!(f, "default"),
            TtlStatus::Reduced => write!(f, "reduced"),
            TtlStatus::Extended => write!(f, "extended"),
        }
    }
}

impl From<TtlStatus> for ChainEpoch {
    fn from(status: TtlStatus) -> Self {
        match status {
            TtlStatus::Default => TtlStatus::DEFAULT_MAX_TTL,
            TtlStatus::Reduced => 0,
            TtlStatus::Extended => ChainEpoch::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subsciption_id_length() {
        let id_str = |len: usize| "a".repeat(len);
        let id = SubscriptionId::new(&id_str(SubscriptionId::MAX_LEN)).unwrap();
        assert_eq!(id.inner, id_str(SubscriptionId::MAX_LEN));

        let id = SubscriptionId::new(&id_str(SubscriptionId::MAX_LEN + 1));
        assert!(id.is_err());
    }
}
