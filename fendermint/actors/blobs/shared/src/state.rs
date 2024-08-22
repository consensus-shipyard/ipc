// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

/// Blob blake3 hash.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hash(pub [u8; 32]);

/// Iroh node public key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
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
}

/// The stored representation of a blob.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Blob {
    /// The size of the content.
    pub size: u64,
    /// Expiry block.
    pub expiry: ChainEpoch,
    /// TODO: add subs
    //pub subs: HashMap<Address, Subscription>,
    /// Source Iroh node ID used for ingestion.
    pub source: PublicKey,
    /// Whether the blob has been resolved.
    /// TODO: change to enum: resolving, resolved, failed
    pub resolved: bool,
}

#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Subscription {
    /// Expiry block.
    pub expiry: ChainEpoch,
}
