// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::anyhow;
use fendermint_actor_blobs_shared::params::GetStatsReturn;
use fendermint_actor_blobs_shared::state::{
    Account, Blob, BlobStatus, Hash, PublicKey, Subscription,
};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use num_traits::{ToPrimitive, Zero};

/// The state represents all accounts and stored blobs.
/// TODO: use raw HAMTs
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// The total free storage capacity of the subnet.
    pub capacity_free: BigInt,
    /// The total used storage capacity of the subnet.
    pub capacity_used: BigInt,
    /// The total number of credits sold in the subnet.
    pub credit_sold: BigInt,
    /// The total number of credits committed to active storage in the subnet.
    pub credit_committed: BigInt,
    /// The total number of credits debited in the subnet.
    pub credit_debited: BigInt,
    /// The byte-blocks per atto token rate set at genesis.
    pub credit_debit_rate: u64,
    /// Map containing all accounts by robust (non-ID) actor address.
    /// TODO: add list of blobs to account
    pub accounts: HashMap<Address, Account>,
    /// Map containing all blobs.
    pub blobs: HashMap<Hash, Blob>,
    /// Map of currently resolving blob hashes to source Iroh node IDs.
    pub resolving: BTreeMap<Hash, HashSet<PublicKey>>,
}

impl State {
    pub fn new(capacity: u64, credit_debit_rate: u64) -> anyhow::Result<Self> {
        Ok(Self {
            capacity_free: BigInt::from(capacity),
            capacity_used: BigInt::zero(),
            credit_sold: BigInt::zero(),
            credit_committed: BigInt::zero(),
            credit_debited: BigInt::zero(),
            credit_debit_rate,
            accounts: HashMap::new(),
            blobs: HashMap::new(),
            resolving: BTreeMap::new(),
        })
    }

    pub fn get_stats(&self, balance: TokenAmount) -> anyhow::Result<GetStatsReturn> {
        Ok(GetStatsReturn {
            balance,
            capacity_free: self.capacity_free.clone(),
            capacity_used: self.capacity_used.clone(),
            credit_sold: self.credit_sold.clone(),
            credit_committed: self.credit_committed.clone(),
            credit_debited: self.credit_debited.clone(),
            credit_debit_rate: self.credit_debit_rate,
            num_accounts: self.accounts.len() as u64,
            num_blobs: self.blobs.len() as u64,
            num_resolving: self.resolving.len() as u64,
        })
    }

    pub fn buy_credit(
        &mut self,
        address: Address,
        amount: TokenAmount,
        current_epoch: ChainEpoch,
    ) -> anyhow::Result<Account> {
        let credits = self.credit_debit_rate * amount.atto();

        // Don't sell credits if we're at storage capacity
        // TODO: This should be more nuanced, i.e., pick some min block duration and storage amount
        // at which to stop selling credits. Say there's only 1 byte of capcity left,
        // we don't want to sell a bunch of credits even though they could be used if the account
        // wants to store 1 byte at a time, which is unlikely :)
        if self.capacity_used == self.capacity_free {
            return Err(anyhow!("credits not available (subnet has reach capacity)"));
        }
        self.credit_sold += &credits;

        match self.accounts.get_mut(&address) {
            Some(account) => {
                account.credit_free += &credits;
                Ok(account.clone())
            }
            None => {
                let account = Account {
                    capacity_used: BigInt::zero(),
                    credit_free: credits.clone(),
                    credit_committed: BigInt::zero(),
                    last_debit_epoch: current_epoch,
                };
                self.accounts.insert(address, account.clone());
                Ok(account)
            }
        }
    }

    pub fn get_account(&self, address: Address) -> anyhow::Result<Option<Account>> {
        let account = self.accounts.get(&address).cloned();
        Ok(account)
    }

    // TODO: expiry should be optional, ie, pay for as long as there's credit
    pub fn add_blob(
        &mut self,
        sender: Address,
        current_epoch: ChainEpoch,
        hash: Hash,
        size: u64,
        expiry: ChainEpoch,
        source: PublicKey,
    ) -> anyhow::Result<Account> {
        if expiry <= current_epoch {
            return Err(anyhow!("expiry must be in the future"));
        }

        match self.accounts.get_mut(&sender) {
            Some(account) => {
                let size = BigInt::from(size);
                // Capacity updates and required credit depend on whether the sender is already
                // subcribing to this blob
                let mut new_capacity = BigInt::zero();
                let mut new_account_capacity = BigInt::zero();
                let required_credit: BigInt;
                let blob = if let Some(blob) = self.blobs.get(&hash) {
                    // We could get_mut to begin with, but the logic below is simpler if we
                    // have transactional control
                    let mut blob = blob.clone();
                    if let Some(sub) = blob.subs.get(&sender) {
                        let mut sub = sub.clone();
                        // Required credit can be negative if sender is reducing expiry
                        required_credit = (sub.expiry - expiry) as u64 * &size;
                        sub.expiry = expiry;
                        // Overwrite source allows sender to retry resolving
                        sub.source = source;
                    } else {
                        // One or more accounts have already committed credit.
                        // However, we still need to reserve the full required credit from the new
                        // subscriber, as the existing account(s) may decide to change the
                        // expiry or cancel.
                        required_credit = expiry as u64 * &size;
                        new_account_capacity = size.clone();
                        // Add new subscription
                        blob.subs.insert(sender, Subscription { expiry, source });
                    }
                    match blob.status {
                        BlobStatus::Added(_) | BlobStatus::Failed => {
                            // It's pending or failed, reset with current epoch
                            blob.status = BlobStatus::Added(current_epoch)
                        }
                        BlobStatus::Resolved => {
                            // No-op, already resolved
                        }
                    }
                    blob
                } else {
                    required_credit = expiry as u64 * &size;
                    new_capacity = size.clone();
                    new_account_capacity = size.clone();

                    // Create new blob
                    Blob {
                        size: size.to_u64().unwrap(),
                        subs: HashMap::from([(sender, Subscription { expiry, source })]),
                        status: BlobStatus::Added(current_epoch),
                    }
                };

                if account.credit_free < required_credit {
                    return Err(anyhow!(
                        "account {} has insufficient credit (available: {}; required: {})",
                        sender,
                        account.credit_free,
                        required_credit
                    ));
                }

                // Debit for existing usage
                let debit_blocks = current_epoch - account.last_debit_epoch;
                let debit = debit_blocks as u64 * &account.capacity_used;
                self.credit_debited += &debit;
                self.credit_committed -= &debit;
                account.credit_committed -= &debit;
                account.last_debit_epoch = current_epoch;

                // Account for new size and move free credit to committed credit
                self.capacity_used += &new_capacity;
                account.capacity_used += &new_account_capacity;
                self.credit_committed += &required_credit;
                account.credit_committed += &required_credit;
                account.credit_free -= &required_credit;

                // Add/update hash and its source to resolving
                self.resolving
                    .entry(hash)
                    .and_modify(|sources| {
                        sources.insert(source);
                    })
                    .or_insert(HashSet::from([source]));

                // Add/update blob
                self.blobs.insert(hash, blob);

                Ok(account.clone())
            }
            None => Err(anyhow!("account {} not found", sender)),
        }
    }

    pub fn get_blob(&self, hash: Hash) -> anyhow::Result<Option<Blob>> {
        let blob = self.blobs.get(&hash).cloned();
        Ok(blob)
    }

    pub fn resolve_blob(&mut self, hash: Hash) -> anyhow::Result<()> {
        self.resolving.remove(&hash);
        match self.blobs.get_mut(&hash) {
            Some(blob) => {
                blob.status = BlobStatus::Resolved;
                Ok(())
            }
            // Don't error here in case the key was already deleted
            None => Ok(()),
        }
    }

    pub fn get_resolving_blobs(&self) -> anyhow::Result<BTreeMap<Hash, HashSet<PublicKey>>> {
        Ok(self.resolving.clone())
    }

    // TODO: give back credit and capacity
    pub fn fail_blob(&mut self, hash: Hash) -> anyhow::Result<()> {
        self.resolving.remove(&hash);
        match self.blobs.get_mut(&hash) {
            Some(blob) => {
                blob.status = BlobStatus::Failed;
                Ok(())
            }
            // Don't error here in case the key was already deleted
            None => Ok(()),
        }
    }

    // TODO: Reverse accounting in add and return Account.
    pub fn delete_blob(&mut self, hash: Hash) -> anyhow::Result<()> {
        self.blobs.remove(&hash);
        Ok(())
    }
}
