// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Blob blake3 hash.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hash(pub [u8; 32]);

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

/// Iroh node public key.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct PublicKey(pub [u8; 32]);

/// The stored representation of a credit account.
#[derive(Clone, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Account {
    /// Total size of all blobs managed by the account.
    pub capacity_used: BigInt,
    /// Current free credit in byte-blocks that can be used for new commitments.
    pub credit_free: BigInt,
    /// Current committed credit in byte-blocks that will be used for debits.
    pub credit_committed: BigInt,
    /// The chain epoch of the last debit.
    pub last_debit_epoch: ChainEpoch,
    /// Credit approvals to other accounts, keyed by receiver, keyed by caller,
    /// which could be the receiver or a specific contract, like an object store.
    /// This allows for limiting approvals to interactions from a specific contract.
    /// For example, an approval for Alice might be valid for any contract caller, so long as
    /// the origin is Alice.
    /// An approval for Bob might be valid from only one contract caller, so long as
    /// the origin is Bob.
    pub approvals: HashMap<Address, HashMap<Address, CreditApproval>>,
}

impl Account {
    pub fn new(credit_free: BigInt, current_epoch: ChainEpoch) -> Self {
        Self {
            capacity_used: Default::default(),
            credit_free,
            credit_committed: Default::default(),
            last_debit_epoch: current_epoch,
            approvals: Default::default(),
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
    /// Counter for how much credit has been committed via this approval.
    pub committed: BigInt,
}

/// The stored representation of a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Blob {
    /// The size of the content.
    pub size: u64,
    /// Active subscribers (accounts) that are paying for the blob.
    pub subs: HashMap<Address, Subscription>,
    /// Blob status.
    pub status: BlobStatus,
}

/// The status of a blob.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BlobStatus {
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
            BlobStatus::Pending => write!(f, "pending"),
            BlobStatus::Resolved => write!(f, "resolved"),
            BlobStatus::Failed => write!(f, "failed"),
        }
    }
}

/// An object used to determine what [`Account`](s) are accountable for a blob, and for how long.
/// Subscriptions allow us to distribute the cost of a blob across multiple accounts that
/// have added the same blob.   
#[derive(Clone, Debug, Default, Serialize_tuple, Deserialize_tuple)]
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
}
