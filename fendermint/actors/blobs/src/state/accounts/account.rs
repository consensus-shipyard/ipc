// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use fendermint_actor_blobs_shared::{self as shared, credit::Credit};
use fil_actors_runtime::{runtime::Runtime, ActorError};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount};
use recall_actor_sdk::util::to_delegated_address;
use recall_ipld::hamt::{self, map::TrackedFlushResult, BytesKey};

use crate::state::credit::Approvals;

/// The stored representation of an account.
#[derive(Clone, PartialEq, Serialize_tuple, Deserialize_tuple)]
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
    pub approvals_to: Approvals,
    /// Credit approvals to this account from other accounts, keyed by sender.
    pub approvals_from: Approvals,
    /// The maximum allowed TTL for actor's blobs.
    pub max_ttl: ChainEpoch,
    /// The total token value an account has used to buy credits.
    pub gas_allowance: TokenAmount,
}

impl Account {
    /// Returns a new [`Account`].
    pub fn new<BS: Blockstore>(
        store: &BS,
        current_epoch: ChainEpoch,
        max_ttl: ChainEpoch,
    ) -> Result<Self, ActorError> {
        Ok(Self {
            capacity_used: 0,
            credit_free: Credit::default(),
            credit_committed: Credit::default(),
            credit_sponsor: None,
            last_debit_epoch: current_epoch,
            approvals_to: Approvals::new(store)?,
            approvals_from: Approvals::new(store)?,
            max_ttl,
            gas_allowance: TokenAmount::default(),
        })
    }
}

impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account")
            .field("capacity_used", &self.capacity_used)
            .field("credit_free", &self.credit_free)
            .field("credit_committed", &self.credit_committed)
            .field("credit_sponsor", &self.credit_sponsor)
            .field("last_debit_epoch", &self.last_debit_epoch)
            .field("max_ttl", &self.max_ttl)
            .field("gas_allowance", &self.gas_allowance)
            .finish()
    }
}

impl Account {
    /// Returns [`shared::accounts::Account`] that is safe to return from actor methods.
    pub fn to_shared(&self, rt: &impl Runtime) -> Result<shared::accounts::Account, ActorError> {
        let store = rt.store();
        let mut approvals_to = HashMap::new();
        self.approvals_to
            .hamt(store)?
            .for_each(|address, approval| {
                let external_account_address = to_delegated_address(rt, address)?;
                approvals_to.insert(external_account_address, approval.clone());
                Ok(())
            })?;

        let mut approvals_from = HashMap::new();
        self.approvals_from
            .hamt(store)?
            .for_each(|address, approval| {
                let external_account_address = to_delegated_address(rt, address)?;
                approvals_from.insert(external_account_address, approval.clone());
                Ok(())
            })?;

        Ok(shared::accounts::Account {
            capacity_used: self.capacity_used,
            credit_free: self.credit_free.clone(),
            credit_committed: self.credit_committed.clone(),
            credit_sponsor: self.credit_sponsor,
            last_debit_epoch: self.last_debit_epoch,
            approvals_to,
            approvals_from,
            max_ttl: self.max_ttl,
            gas_allowance: self.gas_allowance.clone(),
        })
    }
}

/// HAMT wrapper for accounts state.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct Accounts {
    /// The HAMT root.
    pub root: hamt::Root<Address, Account>,
    /// The size of the collection.
    size: u64,
    /// The next account to debit in the current debit cycle.
    /// If this is None, we have finished the debit cycle.
    next_debit_address: Option<Address>,
}

impl Accounts {
    /// Returns a new account collection.
    pub fn new<BS: Blockstore>(store: &BS) -> Result<Self, ActorError> {
        let root = hamt::Root::<Address, Account>::new(store, "accounts")?;
        Ok(Self {
            root,
            size: 0,
            next_debit_address: None,
        })
    }

    /// Returns the underlying [`hamt::map::Hamt`].
    pub fn hamt<'a, BS: Blockstore>(
        &self,
        store: BS,
    ) -> Result<hamt::map::Hamt<'a, BS, Address, Account>, ActorError> {
        self.root.hamt(store, self.size)
    }

    /// Saves the state from the [`TrackedFlushResult`].
    pub fn save_tracked(&mut self, tracked_flush_result: TrackedFlushResult<Address, Account>) {
        self.root = tracked_flush_result.root;
        self.size = tracked_flush_result.size
    }

    /// Saves the start address to be used by the next debit round.  
    pub fn save_debit_progress(&mut self, next_address: Option<Address>) {
        self.next_debit_address = next_address;
    }

    /// Returns the start address to be used by the next debit round.
    pub fn get_debit_start_address(&self) -> Option<BytesKey> {
        self.next_debit_address
            .map(|address| BytesKey::from(address.to_bytes()))
    }

    /// The size of the collection.
    pub fn len(&self) -> u64 {
        self.size
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}
