// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount};

use crate::credit::{Credit, CreditApproval};

/// The external (shared) view of an account.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Account {
    /// Total size of all blobs managed by the account.
    pub capacity_used: u64,
    /// Current free credit in byte-blocks that can be used for new commitments.
    pub credit_free: Credit,
    /// Current committed credit in byte-blocks that will be used for debits.
    pub credit_committed: Credit,
    /// Optional default sponsor account address.
    pub credit_sponsor: Option<Address>,
    /// The chain epoch of the last debit.
    pub last_debit_epoch: ChainEpoch,
    /// Credit approvals to other accounts from this account, keyed by receiver.
    pub approvals_to: HashMap<Address, CreditApproval>,
    /// Credit approvals to this account from other accounts, keyed by sender.
    pub approvals_from: HashMap<Address, CreditApproval>,
    /// The maximum allowed TTL for actor's blobs.
    pub max_ttl: ChainEpoch,
    /// The total token value an account has used to buy credits.
    pub gas_allowance: TokenAmount,
}
