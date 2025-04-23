// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_encoding::tuple::*;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_shared::econ::TokenAmount;
use fvm_shared::{address::Address, ActorID};

use crate::credit::{Credit, TokenCreditRate};

pub mod accounts;
pub mod blobs;
pub mod bytes;
pub mod credit;
pub mod method;
pub mod sdk;

/// The unique identifier for the blob actor in the system.
pub const BLOBS_ACTOR_ID: ActorID = 66;
/// The address of the blob actor, derived from its actor ID.
pub const BLOBS_ACTOR_ADDR: Address = Address::new_id(BLOBS_ACTOR_ID);

/// The stats of the blob actor.
#[derive(Clone, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct GetStatsReturn {
    /// The current token balance earned by the subnet.
    pub balance: TokenAmount,
    /// The total free storage capacity of the subnet.
    pub capacity_free: u64,
    /// The total used storage capacity of the subnet.
    pub capacity_used: u64,
    /// The total number of credits sold in the subnet.
    pub credit_sold: Credit,
    /// The total number of credits committed to active storage in the subnet.
    pub credit_committed: Credit,
    /// The total number of credits debited in the subnet.
    pub credit_debited: Credit,
    /// The token to credit rate.
    pub token_credit_rate: TokenCreditRate,
    /// Total number of debit accounts.
    pub num_accounts: u64,
    /// Total number of actively stored blobs.
    pub num_blobs: u64,
    /// Total number of blobs that are not yet added to the validator's resolve pool.
    pub num_added: u64,
    // Total bytes of all blobs that are not yet added to the validator's resolve pool.
    pub bytes_added: u64,
    /// Total number of currently resolving blobs.
    pub num_resolving: u64,
    /// Total bytes of all currently resolving blobs.
    pub bytes_resolving: u64,
}
